//! Defines data structure for WasmEdge async mechanism.

use fiber_for_wasmedge::{Fiber, FiberStack, Suspend};
use std::{
    future::Future,
    pin::Pin,
    ptr,
    task::{Context, Poll},
};

/// Defines a FiberFuture.
pub(crate) struct FiberFuture<'a> {
    fiber: Fiber<'a, Result<(), ()>, (), Result<(), ()>>,
    current_suspend: *mut *const Suspend<Result<(), ()>, (), Result<(), ()>>,
    current_poll_cx: *mut *mut Context<'static>,
}
impl<'a> FiberFuture<'a> {
    /// Create a fiber to execute the given function.
    ///
    /// # Arguments
    ///
    /// * `func` - The function to execute.
    ///
    /// # Error
    ///
    /// If fail to create the fiber stack or the fiber fail to resume, then an error is returned.
    pub(crate) async fn on_fiber<R>(
        async_state: &AsyncState,
        func: impl FnOnce() -> R + Send,
    ) -> Result<R, ()> {
        let mut slot = None;
        let future = {
            let current_poll_cx = async_state.current_poll_cx.get();
            let current_suspend = async_state.current_suspend.get();

            let stack = FiberStack::new(2 << 20).map_err(|_e| ())?;
            let slot = &mut slot;
            let fiber = Fiber::new(stack, move |keep_going, suspend| {
                keep_going?;

                unsafe {
                    let _reset = Reset(current_suspend, *current_suspend);
                    *current_suspend = suspend;
                    *slot = Some(func());
                    Ok(())
                }
            })
            .map_err(|_e| ())?;

            FiberFuture {
                fiber,
                current_suspend,
                current_poll_cx,
            }
        };
        future.await?;

        Ok(slot.unwrap())
    }
}
impl<'a> Future for FiberFuture<'a> {
    type Output = Result<(), ()>;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        unsafe {
            let _reset = Reset(self.current_poll_cx, *self.current_poll_cx);
            *self.current_poll_cx =
                std::mem::transmute::<&mut Context<'_>, *mut Context<'static>>(cx);

            let async_cx = AsyncCx {
                current_suspend: self.current_suspend,
                current_poll_cx: self.current_poll_cx,
            };
            ASYNC_CX.set(&async_cx, || match self.as_ref().fiber.resume(Ok(())) {
                Ok(ret) => Poll::Ready(ret),
                Err(_) => Poll::Pending,
            })
        }
    }
}
unsafe impl Send for FiberFuture<'_> {}

type FiberSuspend = Suspend<Result<(), ()>, (), Result<(), ()>>;

// jmp_buf, in_host, timeout
scoped_tls::scoped_thread_local!(static ASYNC_JMP_BUF: (*mut setjmp::sigjmp_buf,*mut bool,*mut bool));
unsafe extern "C" fn async_timeout(sig: i32, info: *mut libc::siginfo_t) {
    if let Some(info) = info.as_mut() {
        let si_value = info.si_value();
        let value: *mut libc::pthread_t = si_value.sival_ptr.cast();
        let dist_pthread = *value;
        let self_pthread = libc::pthread_self();
        if self_pthread == dist_pthread {
            if ASYNC_JMP_BUF.is_set() {
                let (env, in_host, timeout) = ASYNC_JMP_BUF.with(|f| *f);
                if *in_host {
                    *timeout = true;
                } else {
                    setjmp::siglongjmp(env, 1);
                }
            }
        } else {
            libc::pthread_sigqueue(dist_pthread, sig, si_value);
        }
    }
}

static INIT_SIGNAL_LISTEN: std::sync::Once = std::sync::Once::new();

/// Defines a TimeoutFiberFuture.
pub(crate) struct TimeoutFiberFuture<'a> {
    fiber: Fiber<'a, Result<(), ()>, (), Result<(), ()>>,
    current_suspend: *mut *const Suspend<Result<(), ()>, (), Result<(), ()>>,
    current_poll_cx: *mut *mut Context<'static>,
    timeout_sec: u64,
}

impl<'a> TimeoutFiberFuture<'a> {
    /// Create a fiber to execute the given function.
    ///
    /// # Arguments
    ///
    /// * `func` - The function to execute.
    ///
    /// * `timeout_sec` - The maximum execution time in seconds for the function instance.
    ///
    /// # Error
    ///
    /// If fail to create the fiber stack or the fiber fail to resume, then an error is returned.
    pub(crate) async fn on_fiber<R>(
        async_state: &AsyncState,
        func: impl FnOnce() -> R + Send,
        timeout_sec: u64,
    ) -> Result<R, ()> {
        let mut slot = None;

        let future = {
            let current_poll_cx = async_state.current_poll_cx.get();
            let current_suspend = async_state.current_suspend.get();

            let stack = FiberStack::new(2 << 20).map_err(|_e| ())?;
            let slot = &mut slot;
            let fiber = Fiber::new(stack, move |keep_going, suspend| {
                keep_going?;

                unsafe {
                    let _reset = Reset(current_suspend, *current_suspend);
                    *current_suspend = suspend;
                    *slot = Some(func());
                    Ok(())
                }
            })
            .map_err(|_e| ())?;

            TimeoutFiberFuture {
                fiber,
                current_suspend,
                current_poll_cx,
                timeout_sec,
            }
        };

        future.await?;

        Ok(slot.unwrap())
    }
}

impl<'a> Future for TimeoutFiberFuture<'a> {
    type Output = Result<(), ()>;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        unsafe {
            INIT_SIGNAL_LISTEN.call_once(|| {
                let mut new_act: libc::sigaction = std::mem::zeroed();
                new_act.sa_sigaction = async_timeout as usize;
                new_act.sa_flags = libc::SA_RESTART | libc::SA_SIGINFO;
                libc::sigaction(libc::SIGUSR2, &new_act, std::ptr::null_mut());
            });

            let _reset = Reset(self.current_poll_cx, *self.current_poll_cx);
            *self.current_poll_cx =
                std::mem::transmute::<&mut Context<'_>, *mut Context<'static>>(cx);
            let async_cx = AsyncCx {
                current_suspend: self.current_suspend,
                current_poll_cx: self.current_poll_cx,
            };

            ASYNC_CX.set(&async_cx, || {
                let mut self_thread = libc::pthread_self();
                let mut timerid: libc::timer_t = std::mem::zeroed();
                let mut sev: libc::sigevent = std::mem::zeroed();
                sev.sigev_notify = libc::SIGEV_SIGNAL;
                sev.sigev_signo = libc::SIGUSR2;
                sev.sigev_value.sival_ptr = &mut self_thread as *mut _ as *mut libc::c_void;

                if libc::timer_create(libc::CLOCK_REALTIME, &mut sev, &mut timerid) < 0 {
                    return Poll::Ready(Err(()));
                }
                let mut value: libc::itimerspec = std::mem::zeroed();
                value.it_value.tv_sec = self.timeout_sec as i64;
                if libc::timer_settime(timerid, 0, &value, std::ptr::null_mut()) < 0 {
                    libc::timer_delete(timerid);
                    return Poll::Ready(Err(()));
                }

                let mut env: setjmp::sigjmp_buf = std::mem::zeroed();
                let mut in_host = false;
                let mut timeout = false;
                let r = if setjmp::sigsetjmp(&mut env, 1) == 0 {
                    ASYNC_JMP_BUF.set(&(&mut env, &mut in_host, &mut timeout), || {
                        match self.as_ref().fiber.resume(Ok(())) {
                            Ok(ret) => Poll::Ready(ret),
                            Err(_) => Poll::Pending,
                        }
                    })
                } else {
                    Poll::Ready(Err(()))
                };
                libc::timer_delete(timerid);
                r
            })
        }
    }
}
unsafe impl Send for TimeoutFiberFuture<'_> {}

scoped_tls::scoped_thread_local!(static ASYNC_CX: AsyncCx);

/// Defines a async state that contains the pointer to current poll context and current suspend.
#[derive(Debug)]
pub struct AsyncState {
    current_suspend: std::cell::UnsafeCell<*const FiberSuspend>,
    current_poll_cx: std::cell::UnsafeCell<*mut Context<'static>>,
}
impl Default for AsyncState {
    fn default() -> Self {
        Self::new()
    }
}
impl AsyncState {
    /// Creates a new async state.
    pub fn new() -> Self {
        AsyncState {
            current_suspend: std::cell::UnsafeCell::new(std::ptr::null()),
            current_poll_cx: std::cell::UnsafeCell::new(std::ptr::null_mut()),
        }
    }

    /// Returns an async execution context.
    ///
    /// If the pointer of poll context is null, then None is returned.
    pub fn async_cx(&self) -> Option<AsyncCx> {
        let poll_cx_box_ptr = self.current_poll_cx.get();
        if poll_cx_box_ptr.is_null() {
            return None;
        }
        let poll_cx_inner_ptr = unsafe { *poll_cx_box_ptr };
        if poll_cx_inner_ptr.is_null() {
            return None;
        }

        Some(AsyncCx {
            current_suspend: self.current_suspend.get(),
            current_poll_cx: poll_cx_box_ptr,
        })
    }
}
unsafe impl Send for AsyncState {}
unsafe impl Sync for AsyncState {}

/// Defines an async execution context.
#[derive(Debug, Clone, Copy)]
pub struct AsyncCx {
    current_suspend: *mut *const Suspend<Result<(), ()>, (), Result<(), ()>>,
    current_poll_cx: *mut *mut Context<'static>,
}
impl Default for AsyncCx {
    fn default() -> Self {
        Self::new()
    }
}
impl AsyncCx {
    /// Creates a new async execution context.
    pub fn new() -> Self {
        ASYNC_CX.with(|async_cx| *async_cx)
    }

    /// Runs a future to completion.
    ///
    /// # Arguments
    ///
    /// * `future` - The future to run.
    ///
    /// # Error
    ///
    /// If fail to run, then an error is returned.
    pub(crate) unsafe fn block_on<U>(
        &self,
        mut future: Pin<&mut (dyn Future<Output = U> + Send)>,
    ) -> Result<U, ()> {
        let suspend = *self.current_suspend;
        let _reset = Reset(self.current_suspend, suspend);
        *self.current_suspend = ptr::null();
        assert!(!suspend.is_null());

        loop {
            let future_result = {
                let poll_cx = *self.current_poll_cx;
                let _reset = Reset(self.current_poll_cx, poll_cx);
                *self.current_poll_cx = ptr::null_mut();
                assert!(!poll_cx.is_null());
                future.as_mut().poll(&mut *poll_cx)
            };

            match future_result {
                Poll::Ready(t) => break Ok(t),
                Poll::Pending => {}
            }
            let res = (*suspend).suspend(());
            res?;
        }
    }
}

struct Reset<T: Copy>(*mut T, T);
impl<T: Copy> Drop for Reset<T> {
    fn drop(&mut self) {
        unsafe {
            *self.0 = self.1;
        }
    }
}
