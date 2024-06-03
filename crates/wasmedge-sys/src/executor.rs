//! Defines WasmEdge Executor.

use super::ffi;
#[cfg(all(feature = "async", target_os = "linux"))]
use crate::r#async::fiber::{AsyncState, FiberFuture};

#[cfg(all(feature = "async", target_os = "linux", not(target_env = "musl")))]
use crate::r#async::fiber::TimeoutFiberFuture;

use crate::{
    instance::{function::AsFunc, module::InnerInstance},
    store::Store,
    types::WasmEdgeString,
    utils::check,
    AsInstance, Config, Function, Instance, Module, Statistics, WasmEdgeResult, WasmValue,
};
#[cfg(all(target_os = "linux", not(target_env = "musl")))]
use std::os::raw::c_void;
use wasmedge_types::error::WasmEdgeError;

#[cfg(all(target_os = "linux", not(target_env = "musl")))]
pub(crate) struct JmpState {
    pub(crate) sigjmp_buf: *mut setjmp::sigjmp_buf,
}

#[cfg(all(target_os = "linux", not(target_env = "musl")))]
scoped_tls::scoped_thread_local!(pub(crate) static JMP_BUF: JmpState);

#[cfg(all(target_os = "linux", not(target_env = "musl")))]
unsafe extern "C" fn sync_timeout(sig: i32, info: *mut libc::siginfo_t) {
    if let Some(info) = info.as_mut() {
        let si_value = info.si_value();
        let value: *mut libc::pthread_t = si_value.sival_ptr.cast();
        let dist_pthread = *value;
        let self_pthread = libc::pthread_self();
        if self_pthread == dist_pthread {
            if JMP_BUF.is_set() {
                let env = JMP_BUF.with(|s| s.sigjmp_buf);
                setjmp::siglongjmp(env, 1);
            }
        } else {
            libc::pthread_sigqueue(dist_pthread, sig, si_value);
        }
    }
}
#[cfg(all(target_os = "linux", not(target_env = "musl")))]
unsafe extern "C" fn pre_host_func(_: *mut c_void) {
    use libc::SIG_BLOCK;

    let mut set = std::mem::zeroed();
    libc::sigemptyset(&mut set);
    libc::sigaddset(&mut set, timeout_signo());
    libc::pthread_sigmask(SIG_BLOCK, &set, std::ptr::null_mut());
}
#[cfg(all(target_os = "linux", not(target_env = "musl")))]
unsafe extern "C" fn post_host_func(_: *mut c_void) {
    use libc::SIG_UNBLOCK;

    let mut set = std::mem::zeroed();
    libc::sigemptyset(&mut set);
    libc::sigaddset(&mut set, timeout_signo());
    libc::pthread_sigmask(SIG_UNBLOCK, &set, std::ptr::null_mut());
}

#[inline]
#[cfg(all(target_os = "linux", not(target_env = "musl")))]
pub(crate) fn timeout_signo() -> i32 {
    option_env!("SIG_OFFSET")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
        + libc::SIGRTMIN()
}

#[cfg(all(target_os = "linux", not(target_env = "musl")))]
static INIT_SIGNAL_LISTEN: std::sync::Once = std::sync::Once::new();

#[inline(always)]
#[cfg(all(target_os = "linux", not(target_env = "musl")))]
pub(crate) unsafe fn init_signal_listen() {
    INIT_SIGNAL_LISTEN.call_once(|| {
        let mut new_act: libc::sigaction = std::mem::zeroed();
        new_act.sa_sigaction = sync_timeout as usize;
        new_act.sa_flags = libc::SA_RESTART | libc::SA_SIGINFO;
        libc::sigaction(timeout_signo(), &new_act, std::ptr::null_mut());
    });
}

/// Defines an execution environment for both pure WASM and compiled WASM.
#[derive(Debug)]
pub struct Executor {
    pub(crate) inner: InnerExecutor,
}

impl Drop for Executor {
    fn drop(&mut self) {
        unsafe { ffi::WasmEdge_ExecutorDelete(self.inner.0) }
    }
}

impl Executor {
    /// Creates a new [executor](crate::Executor) to be associated with the given [config](crate::Config) and [statistics](crate::Statistics).
    ///
    /// # Arguments
    ///
    /// * `config` - The configuration of the new [executor](crate::Executor).
    ///
    /// * `stat` - The [statistics](crate::Statistics) needed by the new [executor](crate::Executor).
    ///
    /// # Error
    ///
    /// If fail to create a [executor](crate::Executor), then an error is returned.
    pub fn create(config: Option<&Config>, stat: Option<Statistics>) -> WasmEdgeResult<Self> {
        let conf_ctx = config
            .map(|cfg| cfg.inner.0)
            .unwrap_or(std::ptr::null_mut());
        let stat_ctx = stat
            .map(|stat| stat.inner.0)
            .unwrap_or(std::ptr::null_mut());

        let ctx = unsafe { ffi::WasmEdge_ExecutorCreate(conf_ctx, stat_ctx) };

        if ctx.is_null() {
            Err(Box::new(WasmEdgeError::ExecutorCreate))
        } else {
            #[cfg(all(target_os = "linux", not(target_env = "musl")))]
            unsafe {
                ffi::WasmEdge_ExecutorExperimentalRegisterPreHostFunction(
                    ctx,
                    std::ptr::null_mut(),
                    Some(pre_host_func),
                );
                ffi::WasmEdge_ExecutorExperimentalRegisterPostHostFunction(
                    ctx,
                    std::ptr::null_mut(),
                    Some(post_host_func),
                );
            }

            Ok(Executor {
                inner: InnerExecutor(ctx),
            })
        }
    }
}

impl Executor {
    /// Runs a host function instance and returns the results.
    ///
    /// # Arguments
    ///
    /// * `func` - The function instance to run.
    ///
    /// * `params` - The arguments to pass to the function.
    ///
    /// # Errors
    ///
    /// If fail to run the host function, then an error is returned.
    pub fn call_func(
        &mut self,
        func: &mut Function,
        params: impl IntoIterator<Item = WasmValue>,
    ) -> WasmEdgeResult<Vec<WasmValue>> {
        let raw_params = params.into_iter().map(|x| x.as_raw()).collect::<Vec<_>>();

        // get the length of the function's returns
        let func_ty = func
            .ty()
            .ok_or(WasmEdgeError::Func(wasmedge_types::error::FuncError::Type))?;
        let returns_len = func_ty.returns_len();
        let mut returns = Vec::with_capacity(returns_len);

        unsafe {
            check(ffi::WasmEdge_ExecutorInvoke(
                self.inner.0,
                func.get_func_raw(),
                raw_params.as_ptr(),
                raw_params.len() as u32,
                returns.as_mut_ptr(),
                returns_len as u32,
            ))?;

            returns.set_len(returns_len);
        }

        Ok(returns.into_iter().map(Into::into).collect::<Vec<_>>())
    }

    /// Run a host function instance and return the results or timeout.
    ///
    /// # Arguments
    ///
    /// * `func` - The function instance to run.
    ///
    /// * `params` - The arguments to pass to the function.
    ///
    /// * `timeout` - The maximum execution time of the function to be run.
    ///
    /// # Errors
    ///
    /// If fail to run the host function, then an error is returned.
    #[cfg(all(target_os = "linux", not(target_env = "musl")))]
    #[cfg_attr(docsrs, doc(cfg(all(target_os = "linux", not(target_env = "musl")))))]
    pub fn call_func_with_timeout(
        &self,
        func: &mut Function,
        params: impl IntoIterator<Item = WasmValue>,
        timeout: std::time::Duration,
    ) -> WasmEdgeResult<Vec<WasmValue>> {
        use wasmedge_types::error;

        let raw_params = params.into_iter().map(|x| x.as_raw()).collect::<Vec<_>>();
        // get the length of the function's returns
        let func_ty = func
            .ty()
            .ok_or(WasmEdgeError::Func(wasmedge_types::error::FuncError::Type))?;
        let returns_len = func_ty.returns_len();
        let mut returns = Vec::with_capacity(returns_len);

        unsafe {
            init_signal_listen();
            let mut self_thread = libc::pthread_self();
            let mut sigjmp_buf: setjmp::sigjmp_buf = std::mem::zeroed();
            let env = &mut sigjmp_buf as *mut _;

            let mut timerid: libc::timer_t = std::mem::zeroed();
            let mut sev: libc::sigevent = std::mem::zeroed();
            sev.sigev_notify = libc::SIGEV_SIGNAL;
            sev.sigev_signo = timeout_signo();
            sev.sigev_value.sival_ptr = &mut self_thread as *mut _ as *mut libc::c_void;

            if libc::timer_create(libc::CLOCK_REALTIME, &mut sev, &mut timerid) < 0 {
                return Err(Box::new(error::WasmEdgeError::Operation(
                    "timer_create error".into(),
                )));
            }
            let mut value: libc::itimerspec = std::mem::zeroed();
            value.it_value.tv_sec = timeout.as_secs() as _;
            value.it_value.tv_nsec = timeout.subsec_nanos() as _;
            if libc::timer_settime(timerid, 0, &value, std::ptr::null_mut()) < 0 {
                libc::timer_delete(timerid);
                return Err(Box::new(error::WasmEdgeError::Operation(
                    "timer_settime error".into(),
                )));
            }
            let jmp_state = JmpState { sigjmp_buf: env };

            JMP_BUF.set(&jmp_state, || {
                if setjmp::sigsetjmp(env, 1) == 0 {
                    let r = check(ffi::WasmEdge_ExecutorInvoke(
                        self.inner.0,
                        func.get_func_raw(),
                        raw_params.as_ptr(),
                        raw_params.len() as u32,
                        returns.as_mut_ptr(),
                        returns_len as u32,
                    ));
                    libc::timer_delete(timerid);
                    r
                } else {
                    libc::timer_delete(timerid);
                    Err(Box::new(error::WasmEdgeError::ExecuteTimeout))
                }
            })?;

            returns.set_len(returns_len);
            Ok(returns.into_iter().map(Into::into).collect::<Vec<_>>())
        }
    }

    /// Asynchronously runs a host function instance and returns the results.
    ///
    /// # Arguments
    ///
    /// * `async_state` - Used to store asynchronous state at run time.
    ///
    /// * `func` - The function instance to run.
    ///
    /// * `params` - The arguments to pass to the function.
    ///
    /// # Errors
    ///
    /// If fail to run the host function, then an error is returned.
    #[cfg(all(feature = "async", target_os = "linux"))]
    #[cfg_attr(docsrs, doc(cfg(all(feature = "async", target_os = "linux"))))]
    pub async fn call_func_async(
        &mut self,
        async_state: &AsyncState,
        func: &mut Function,
        params: impl IntoIterator<Item = WasmValue> + Send,
    ) -> WasmEdgeResult<Vec<WasmValue>> {
        FiberFuture::on_fiber(async_state, || self.call_func(func, params))
            .await
            .unwrap()
    }

    /// Asynchronously runs a host function instance with a timeout setting
    ///
    /// # Arguments
    ///
    /// * `async_state` - Used to store asynchronous state at run time.
    ///
    /// * `func` - The function instance to run.
    ///
    /// * `params` - The arguments to pass to the function.
    ///
    /// * `timeout` - The maximum execution time of the function to be run.
    ///
    /// # Errors
    ///
    /// If fail to run the host function, then an error is returned.
    #[cfg(all(feature = "async", target_os = "linux", not(target_env = "musl")))]
    #[cfg_attr(
        docsrs,
        doc(cfg(all(feature = "async", target_os = "linux", not(target_env = "musl"))))
    )]
    pub async fn call_func_async_with_timeout(
        &mut self,
        async_state: &AsyncState,
        func: &mut Function,
        params: impl IntoIterator<Item = WasmValue> + Send,
        timeout: std::time::Duration,
    ) -> WasmEdgeResult<Vec<WasmValue>> {
        use wasmedge_types::error;
        let ldd = std::time::SystemTime::now() + timeout;
        TimeoutFiberFuture::on_fiber(async_state, || self.call_func(func, params), ldd)
            .await
            .map_err(|_| Box::new(error::WasmEdgeError::ExecuteTimeout))?
    }

    /// Runs a host function reference instance and returns the results.
    ///
    /// # Arguments
    ///
    /// * `func_ref` - The function reference instance to run.
    ///
    /// * `params` - The arguments to pass to the function.
    ///
    /// # Errors
    ///
    /// If fail to run the host function reference instance, then an error is returned.
    pub fn call_func_ref<FuncRef: AsFunc>(
        &mut self,
        func_ref: &mut FuncRef,
        params: impl IntoIterator<Item = WasmValue>,
    ) -> WasmEdgeResult<Vec<WasmValue>> {
        let raw_params = params.into_iter().map(|x| x.as_raw()).collect::<Vec<_>>();

        // get the length of the function's returns
        let func_ty = func_ref.ty().unwrap();
        let returns_len = func_ty.returns_len();
        let mut returns = Vec::with_capacity(returns_len);

        unsafe {
            check(ffi::WasmEdge_ExecutorInvoke(
                self.inner.0,
                func_ref.get_func_raw(),
                raw_params.as_ptr(),
                raw_params.len() as u32,
                returns.as_mut_ptr(),
                returns_len as u32,
            ))?;
            returns.set_len(returns_len);
        }

        Ok(returns.into_iter().map(Into::into).collect::<Vec<_>>())
    }

    /// Asynchronously runs a host function reference instance and returns the results.
    ///
    /// # Arguments
    ///
    /// * `async_state` - Used to store asynchronous state at run time.
    ///
    /// * `func_ref` - The function reference instance to run.
    ///
    /// * `params` - The arguments to pass to the function.
    ///
    /// # Errors
    ///
    /// If fail to run the host function reference instance, then an error is returned.
    #[cfg(all(feature = "async", target_os = "linux"))]
    #[cfg_attr(docsrs, doc(cfg(all(feature = "async", target_os = "linux"))))]
    pub async fn call_func_ref_async<FuncRef: AsFunc + Send>(
        &mut self,
        async_state: &AsyncState,
        func_ref: &mut FuncRef,
        params: impl IntoIterator<Item = WasmValue> + Send,
    ) -> WasmEdgeResult<Vec<WasmValue>> {
        FiberFuture::on_fiber(async_state, || self.call_func_ref(func_ref, params))
            .await
            .unwrap()
    }
}

impl Executor {
    /// Registers and instantiates a [import module](crate::ImportModule) into a [store](crate::Store).
    ///
    /// # Arguments
    ///
    /// * `store` - The target [store](crate::Store), into which the given [import module](crate::ImportModule) is registered.
    ///
    /// * `import` - The WasmEdge [import module](crate::ImportModule) to be registered.
    ///
    /// # Error
    ///
    /// If fail to register the given [import module](crate::ImportModule), then an error is returned.
    pub fn register_import_module<T: AsInstance + ?Sized>(
        &mut self,
        store: &mut Store,
        import: &T,
    ) -> WasmEdgeResult<()> {
        unsafe {
            check(ffi::WasmEdge_ExecutorRegisterImport(
                self.inner.0,
                store.inner.0,
                import.as_ptr(),
            ))?;
        }

        Ok(())
    }

    /// Registers and instantiates a WasmEdge [module](crate::Module) into a store.
    ///
    /// Instantiates the given WasmEdge [module](crate::Module), including the [functions](crate::Function), [memories](crate::Memory), [tables](crate::Table), and [globals](crate::Global) it hosts; and then, registers the module [instance](crate::Instance) into the [store](crate::Store) with the given name.
    ///
    /// # Arguments
    ///
    /// * `store` - The target [store](crate::Store), into which the given [module](crate::Module) is registered.
    ///
    /// * `module` - A validated [module](crate::Module) to be registered.
    ///
    /// * `name` - The exported name of the registered [module](crate::Module).
    ///
    /// # Error
    ///
    /// If fail to register the given [module](crate::Module), then an error is returned.
    pub fn register_named_module(
        &mut self,
        store: &mut Store,
        module: &Module,
        name: impl AsRef<str>,
    ) -> WasmEdgeResult<Instance> {
        let mut instance_ctx = std::ptr::null_mut();
        let mod_name: WasmEdgeString = name.as_ref().into();
        unsafe {
            check(ffi::WasmEdge_ExecutorRegister(
                self.inner.0,
                &mut instance_ctx,
                store.inner.0,
                module.inner.0 as *const _,
                mod_name.as_raw(),
            ))?;

            let inst = Instance {
                inner: InnerInstance(instance_ctx),
            };

            Ok(inst)
        }
    }

    /// Registers and instantiates a WasmEdge [module](crate::Module) into a [store](crate::Store) as an anonymous module.
    ///
    /// Notice that when a new module is instantiated into the [store](crate::Store), the old instantiated module is removed; in addition, ensure that the [imports](crate::ImportModule) the module depends on are already registered into the [store](crate::Store).
    ///
    ///
    /// # Arguments
    ///
    /// * `store` - The [store](crate::Store), in which the [module](crate::Module) to be instantiated is stored.
    ///
    /// * `ast_mod` - The target [module](crate::Module) to be instantiated.
    ///
    /// # Error
    ///
    /// If fail to instantiate the given [module](crate::Module), then an error is returned.
    pub fn register_active_module(
        &mut self,
        store: &mut Store,
        module: &Module,
    ) -> WasmEdgeResult<Instance> {
        let mut instance_ctx = std::ptr::null_mut();
        unsafe {
            check(ffi::WasmEdge_ExecutorInstantiate(
                self.inner.0,
                &mut instance_ctx,
                store.inner.0,
                module.inner.0 as *const _,
            ))?;

            let inst = Instance {
                inner: InnerInstance(instance_ctx),
            };

            Ok(inst)
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct InnerExecutor(pub(crate) *mut ffi::WasmEdge_ExecutorContext);
unsafe impl Send for InnerExecutor {}
unsafe impl Sync for InnerExecutor {}
