use crate::{
    instance::function::SyncFn, AsInstance, CallingFrame, FuncType, Function, ImportModule,
    Instance, Memory, WasmEdgeResult, WasmValue,
};
use async_wasi::snapshots::{
    common::{
        error::Errno,
        memory::WasmPtr,
        types::{__wasi_ciovec_t, __wasi_size_t},
    },
    preview_1 as p, WasiCtx,
};
use std::{
    future::Future,
    ops::{Deref, DerefMut},
};
use wasmedge_types::{
    error::{CoreCommonError, CoreError, CoreExecutionError},
    ValType,
};

use super::function::{AsyncFn, AsyncFunction};

#[derive(Debug)]
pub struct AsyncInstance(pub(crate) Instance);

impl AsRef<Instance> for AsyncInstance {
    fn as_ref(&self) -> &Instance {
        &self.0
    }
}

impl AsMut<Instance> for AsyncInstance {
    fn as_mut(&mut self) -> &mut Instance {
        &mut self.0
    }
}

#[derive(Debug)]
pub struct AsyncImportObject<T: Send>(ImportModule<T>);
impl<T: Send> Deref for AsyncImportObject<T> {
    type Target = ImportModule<T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T: Send> DerefMut for AsyncImportObject<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: Send> AsyncImportObject<T> {
    pub fn create(name: impl AsRef<str>, data: Box<T>) -> WasmEdgeResult<Self> {
        let inner = ImportModule::create(name, data)?;
        Ok(AsyncImportObject(inner))
    }

    pub fn add_async_func(&mut self, name: impl AsRef<str>, func: AsyncFunction) {
        self.0.add_func(name, func.0);
    }
}
impl<T: Send> AsRef<ImportModule<T>> for AsyncImportObject<T> {
    fn as_ref(&self) -> &ImportModule<T> {
        &self.0
    }
}
impl<T: Send> AsMut<ImportModule<T>> for AsyncImportObject<T> {
    fn as_mut(&mut self) -> &mut ImportModule<T> {
        &mut self.0
    }
}

impl<T: Send> AsInstance for AsyncImportObject<T> {
    unsafe fn as_ptr(&self) -> *const crate::ffi::WasmEdge_ModuleInstanceContext {
        self.0.as_ptr()
    }
}

/// A [AsyncWasiModule] is a module instance for the WASI specification and used in the `async` scenario.
#[derive(Debug)]
pub struct AsyncWasiModule(AsyncImportObject<WasiCtx>);

impl AsRef<AsyncImportObject<WasiCtx>> for AsyncWasiModule {
    fn as_ref(&self) -> &AsyncImportObject<WasiCtx> {
        &self.0
    }
}

impl AsMut<AsyncImportObject<WasiCtx>> for AsyncWasiModule {
    fn as_mut(&mut self) -> &mut AsyncImportObject<WasiCtx> {
        &mut self.0
    }
}

impl AsyncWasiModule {
    /// Creates a [AsyncWasiModule] instance.
    ///
    /// # Arguments
    ///
    /// * `args` - The commandline arguments. The first argument is the program name.
    ///
    /// * `envs` - The environment variables.
    ///
    /// # Error
    ///
    /// If fail to create a [AsyncWasiModule] instance, then an error is returned.
    pub fn create(
        args: Option<Vec<impl AsRef<str>>>,
        envs: Option<Vec<(impl AsRef<str>, impl AsRef<str>)>>,
    ) -> WasmEdgeResult<Self> {
        // create wasi context
        let mut wasi_ctx = WasiCtx::new();

        // push args, envs and preopens
        if let Some(args) = args {
            wasi_ctx.push_args(args.iter().map(|x| x.as_ref().to_string()).collect());
        }
        if let Some(envs) = envs {
            wasi_ctx.push_envs(
                envs.iter()
                    .map(|(k, v)| format!("{}={}", k.as_ref(), v.as_ref()))
                    .collect(),
            );
        }

        Self::create_from_wasi_context(wasi_ctx)
    }

    /// Creates a [AsyncWasiModule] instance with the given wasi context.
    ///
    /// # Arguments
    ///
    /// * `wasi_ctx` - The [WasiCtx](async_wasi::snapshots::WasiCtx) instance.
    ///
    /// # Error
    ///
    /// If fail to create [AsyncWasiModule] instance, then an error is returned.
    pub fn create_from_wasi_context(wasi_ctx: WasiCtx) -> WasmEdgeResult<Self> {
        // create wasi module
        let name = "wasi_snapshot_preview1";

        let mut async_wasi_module = Self(AsyncImportObject::create(name, Box::new(wasi_ctx))?);

        // add sync/async host functions to the module
        for wasi_func in wasi_impls() {
            match wasi_func {
                WasiFunc::SyncFn(name, (ty_args, ty_rets), real_fn) => {
                    let func_ty = FuncType::new(ty_args, ty_rets);

                    let func = unsafe {
                        Function::create_sync_func(
                            &func_ty,
                            real_fn,
                            async_wasi_module.0.get_host_data_mut(),
                            0,
                        )
                    }?;

                    async_wasi_module.0.add_func(&name, func);
                }
                WasiFunc::AsyncFn(name, (ty_args, ty_rets), real_async_fn) => {
                    let func_ty = FuncType::new(ty_args, ty_rets);

                    let func = AsyncFunction::create_async_func(
                        &func_ty,
                        real_async_fn,
                        async_wasi_module.0.get_host_data_mut(),
                        0,
                    )?;

                    async_wasi_module.0.add_async_func(&name, func);
                }
            }
        }

        Ok(async_wasi_module)
    }

    /// Returns the name of the module instance.
    pub fn name(&self) -> &str {
        "wasi_snapshot_preview1"
    }

    /// Returns the WASI exit code.
    ///
    /// The WASI exit code can be accessed after running the "_start" function of a `wasm32-wasi` program.
    pub fn exit_code(&self) -> u32 {
        self.0.get_host_data().exit_code
    }
}

// ============== wasi host functions ==============

fn args_get(
    data: &mut WasiCtx,
    _inst: &mut Instance,
    frame: &mut CallingFrame,
    args: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, CoreError> {
    let mut mem = frame
        .memory_mut(0)
        .ok_or(CoreError::Execution(CoreExecutionError::MemoryOutOfBounds))?;

    if let Some([argv, argv_buf]) = args.get(0..2) {
        let argv = argv.to_i32() as usize;
        let argv_buf = argv_buf.to_i32() as usize;
        Ok(to_wasm_return(p::args_get(
            data,
            &mut *mem,
            WasmPtr::from(argv),
            WasmPtr::from(argv_buf),
        )))
    } else {
        Err(CoreError::Execution(CoreExecutionError::FuncSigMismatch))
    }
}

fn args_sizes_get(
    data: &mut WasiCtx,
    _inst: &mut Instance,
    frame: &mut CallingFrame,
    args: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, CoreError> {
    let mut mem = frame
        .memory_mut(0)
        .ok_or(CoreError::Execution(CoreExecutionError::MemoryOutOfBounds))?;

    if let Some([argc, argv_buf_size]) = args.get(0..2) {
        let argc = argc.to_i32() as usize;
        let argv_buf_size = argv_buf_size.to_i32() as usize;
        Ok(to_wasm_return(p::args_sizes_get(
            data,
            &mut mem as &mut Memory,
            WasmPtr::from(argc),
            WasmPtr::from(argv_buf_size),
        )))
    } else {
        Err(CoreError::Execution(CoreExecutionError::FuncSigMismatch))
    }
}

fn environ_get(
    data: &mut WasiCtx,
    _inst: &mut Instance,
    frame: &mut CallingFrame,
    args: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, CoreError> {
    let mut mem = frame
        .memory_mut(0)
        .ok_or(CoreError::Execution(CoreExecutionError::MemoryOutOfBounds))?;

    if let Some([p1, p2]) = args.get(0..2) {
        let environ = p1.to_i32() as usize;
        let environ_buf = p2.to_i32() as usize;
        Ok(to_wasm_return(p::environ_get(
            data,
            &mut mem as &mut Memory,
            WasmPtr::from(environ),
            WasmPtr::from(environ_buf),
        )))
    } else {
        Err(CoreError::Execution(CoreExecutionError::FuncSigMismatch))
    }
}

fn environ_sizes_get(
    data: &mut WasiCtx,
    _inst: &mut Instance,
    frame: &mut CallingFrame,
    args: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, CoreError> {
    let mut mem = frame
        .memory_mut(0)
        .ok_or(CoreError::Execution(CoreExecutionError::MemoryOutOfBounds))?;

    if let Some([p1, p2]) = args.get(0..2) {
        let environ_count = p1.to_i32() as usize;
        let environ_buf_size = p2.to_i32() as usize;
        Ok(to_wasm_return(p::environ_sizes_get(
            data,
            &mut mem as &mut Memory,
            WasmPtr::from(environ_count),
            WasmPtr::from(environ_buf_size),
        )))
    } else {
        Err(CoreError::Execution(CoreExecutionError::FuncSigMismatch))
    }
}

fn clock_res_get(
    data: &mut WasiCtx,
    _inst: &mut Instance,
    frame: &mut CallingFrame,
    args: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, CoreError> {
    let mut mem = frame
        .memory_mut(0)
        .ok_or(CoreError::Execution(CoreExecutionError::MemoryOutOfBounds))?;

    if let Some([p1, p2]) = args.get(0..2) {
        let clock_id = p1.to_i32() as u32;
        let resolution_ptr = p2.to_i32() as usize;
        Ok(to_wasm_return(p::clock_res_get(
            data,
            &mut mem as &mut Memory,
            clock_id,
            WasmPtr::from(resolution_ptr),
        )))
    } else {
        Err(CoreError::Execution(CoreExecutionError::FuncSigMismatch))
    }
}

fn clock_time_get(
    data: &mut WasiCtx,
    _inst: &mut Instance,
    frame: &mut CallingFrame,
    args: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, CoreError> {
    let mut mem = frame
        .memory_mut(0)
        .ok_or(CoreError::Execution(CoreExecutionError::MemoryOutOfBounds))?;

    if let Some([p1, p2, p3]) = args.get(0..3) {
        let clock_id = p1.to_i32() as u32;
        let precision = p2.to_i64() as u64;
        let time_ptr = p3.to_i32() as usize;

        Ok(to_wasm_return(p::clock_time_get(
            data,
            &mut mem as &mut Memory,
            clock_id,
            precision,
            WasmPtr::from(time_ptr),
        )))
    } else {
        Err(CoreError::Execution(CoreExecutionError::FuncSigMismatch))
    }
}

fn random_get(
    data: &mut WasiCtx,
    _inst: &mut Instance,
    frame: &mut CallingFrame,
    args: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, CoreError> {
    let mut mem = frame
        .memory_mut(0)
        .ok_or(CoreError::Execution(CoreExecutionError::MemoryOutOfBounds))?;

    if let Some([p1, p2]) = args.get(0..2) {
        let buf = p1.to_i32() as usize;
        let buf_len = p2.to_i32() as u32;

        Ok(to_wasm_return(p::random_get(
            data,
            &mut mem as &mut Memory,
            WasmPtr::from(buf),
            buf_len,
        )))
    } else {
        Err(CoreError::Execution(CoreExecutionError::FuncSigMismatch))
    }
}

fn fd_prestat_get(
    data: &mut WasiCtx,
    _inst: &mut Instance,
    frame: &mut CallingFrame,
    args: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, CoreError> {
    let mut mem = frame
        .memory_mut(0)
        .ok_or(CoreError::Execution(CoreExecutionError::MemoryOutOfBounds))?;

    if let Some([p1, p2]) = args.get(0..2) {
        let fd = p1.to_i32();
        let prestat_ptr = p2.to_i32() as usize;

        Ok(to_wasm_return(p::fd_prestat_get(
            data,
            &mut mem as &mut Memory,
            fd,
            WasmPtr::from(prestat_ptr),
        )))
    } else {
        Err(CoreError::Execution(CoreExecutionError::FuncSigMismatch))
    }
}

fn fd_prestat_dir_name(
    data: &mut WasiCtx,
    _inst: &mut Instance,
    frame: &mut CallingFrame,
    args: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, CoreError> {
    let mut mem = frame
        .memory_mut(0)
        .ok_or(CoreError::Execution(CoreExecutionError::MemoryOutOfBounds))?;

    if let Some([p1, p2, p3]) = args.get(0..3) {
        let fd = p1.to_i32();
        let path_buf_ptr = p2.to_i32() as usize;
        let path_max_len = p3.to_i32() as u32;

        Ok(to_wasm_return(p::fd_prestat_dir_name(
            data,
            &mut mem as &mut Memory,
            fd,
            WasmPtr::from(path_buf_ptr),
            path_max_len,
        )))
    } else {
        Err(CoreError::Execution(CoreExecutionError::FuncSigMismatch))
    }
}

fn fd_renumber(
    data: &mut WasiCtx,
    _inst: &mut Instance,
    frame: &mut CallingFrame,
    args: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, CoreError> {
    let mut mem = frame
        .memory_mut(0)
        .ok_or(CoreError::Execution(CoreExecutionError::MemoryOutOfBounds))?;

    if let Some([p1, p2]) = args.get(0..2) {
        let from = p1.to_i32();
        let to = p2.to_i32();

        Ok(to_wasm_return(p::fd_renumber(
            data,
            &mut mem as &mut Memory,
            from,
            to,
        )))
    } else {
        Err(CoreError::Execution(CoreExecutionError::FuncSigMismatch))
    }
}

fn fd_advise(
    data: &mut WasiCtx,
    _inst: &mut Instance,
    frame: &mut CallingFrame,
    args: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, CoreError> {
    let mut mem = frame
        .memory_mut(0)
        .ok_or(CoreError::Execution(CoreExecutionError::MemoryOutOfBounds))?;

    if let Some([p1, p2, p3, p4]) = args.get(0..4) {
        let fd = p1.to_i32();
        let offset = p2.to_i64() as u64;
        let len = p3.to_i64() as u64;
        let advice = p4.to_i32() as u8;

        Ok(to_wasm_return(p::fd_advise(
            data,
            &mut mem as &mut Memory,
            fd,
            offset,
            len,
            advice,
        )))
    } else {
        Err(CoreError::Execution(CoreExecutionError::FuncSigMismatch))
    }
}

fn fd_allocate(
    data: &mut WasiCtx,
    _inst: &mut Instance,
    frame: &mut CallingFrame,
    args: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, CoreError> {
    let mut mem = frame
        .memory_mut(0)
        .ok_or(CoreError::Execution(CoreExecutionError::MemoryOutOfBounds))?;

    if let Some([p1, p2, p3]) = args.get(0..3) {
        let fd = p1.to_i32();
        let offset = p2.to_i64() as u64;
        let len = p3.to_i64() as u64;

        Ok(to_wasm_return(p::fd_allocate(
            data,
            &mut mem as &mut Memory,
            fd,
            offset,
            len,
        )))
    } else {
        Err(CoreError::Execution(CoreExecutionError::FuncSigMismatch))
    }
}

fn fd_close(
    data: &mut WasiCtx,
    _inst: &mut Instance,
    frame: &mut CallingFrame,
    args: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, CoreError> {
    let mut mem = frame
        .memory_mut(0)
        .ok_or(CoreError::Execution(CoreExecutionError::MemoryOutOfBounds))?;

    if let Some([p1]) = args.get(0..1) {
        let fd = p1.to_i32();

        Ok(to_wasm_return(p::fd_close(
            data,
            &mut mem as &mut Memory,
            fd,
        )))
    } else {
        Err(CoreError::Execution(CoreExecutionError::FuncSigMismatch))
    }
}

fn fd_seek(
    data: &mut WasiCtx,
    _inst: &mut Instance,
    frame: &mut CallingFrame,
    args: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, CoreError> {
    let mut mem = frame
        .memory_mut(0)
        .ok_or(CoreError::Execution(CoreExecutionError::MemoryOutOfBounds))?;

    if let Some([p1, p2, p3, p4]) = args.get(0..4) {
        let fd = p1.to_i32();
        let offset = p2.to_i64();
        let whence = p3.to_i32() as u8;
        let newoffset_ptr = p4.to_i32() as usize;

        Ok(to_wasm_return(p::fd_seek(
            data,
            &mut mem as &mut Memory,
            fd,
            offset,
            whence,
            WasmPtr::from(newoffset_ptr),
        )))
    } else {
        Err(CoreError::Execution(CoreExecutionError::FuncSigMismatch))
    }
}

fn fd_sync(
    data: &mut WasiCtx,
    _inst: &mut Instance,
    frame: &mut CallingFrame,
    args: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, CoreError> {
    let mut mem = frame
        .memory_mut(0)
        .ok_or(CoreError::Execution(CoreExecutionError::MemoryOutOfBounds))?;

    if let Some([p1]) = args.get(0..1) {
        let fd = p1.to_i32();

        Ok(to_wasm_return(p::fd_sync(
            data,
            &mut mem as &mut Memory,
            fd,
        )))
    } else {
        Err(CoreError::Execution(CoreExecutionError::FuncSigMismatch))
    }
}

fn fd_datasync(
    data: &mut WasiCtx,
    _inst: &mut Instance,
    frame: &mut CallingFrame,
    args: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, CoreError> {
    let mut mem = frame
        .memory_mut(0)
        .ok_or(CoreError::Execution(CoreExecutionError::MemoryOutOfBounds))?;

    if let Some([p1]) = args.get(0..1) {
        let fd = p1.to_i32();

        Ok(to_wasm_return(p::fd_datasync(
            data,
            &mut mem as &mut Memory,
            fd,
        )))
    } else {
        Err(CoreError::Execution(CoreExecutionError::FuncSigMismatch))
    }
}

fn fd_tell(
    data: &mut WasiCtx,
    _inst: &mut Instance,
    frame: &mut CallingFrame,
    args: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, CoreError> {
    let mut mem = frame
        .memory_mut(0)
        .ok_or(CoreError::Execution(CoreExecutionError::MemoryOutOfBounds))?;

    if let Some([p1, p2]) = args.get(0..2) {
        let fd = p1.to_i32();
        let offset = p2.to_i32() as usize;

        Ok(to_wasm_return(p::fd_tell(
            data,
            &mut mem as &mut Memory,
            fd,
            WasmPtr::from(offset),
        )))
    } else {
        Err(CoreError::Execution(CoreExecutionError::FuncSigMismatch))
    }
}

fn fd_fdstat_get(
    data: &mut WasiCtx,
    _inst: &mut Instance,
    frame: &mut CallingFrame,
    args: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, CoreError> {
    let mut mem = frame
        .memory_mut(0)
        .ok_or(CoreError::Execution(CoreExecutionError::MemoryOutOfBounds))?;

    if let Some([p1, p2]) = args.get(0..2) {
        let fd = p1.to_i32();
        let buf_ptr = p2.to_i32() as usize;

        Ok(to_wasm_return(p::fd_fdstat_get(
            data,
            &mut mem as &mut Memory,
            fd,
            WasmPtr::from(buf_ptr),
        )))
    } else {
        Err(CoreError::Execution(CoreExecutionError::FuncSigMismatch))
    }
}

fn fd_fdstat_set_flags(
    data: &mut WasiCtx,
    _inst: &mut Instance,
    frame: &mut CallingFrame,
    args: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, CoreError> {
    let mut mem = frame
        .memory_mut(0)
        .ok_or(CoreError::Execution(CoreExecutionError::MemoryOutOfBounds))?;

    if let Some([p1, p2]) = args.get(0..2) {
        let fd = p1.to_i32();
        let flags = p2.to_i32() as u16;

        Ok(to_wasm_return(p::fd_fdstat_set_flags(
            data,
            &mut mem as &mut Memory,
            fd,
            flags,
        )))
    } else {
        Err(CoreError::Execution(CoreExecutionError::FuncSigMismatch))
    }
}

fn fd_fdstat_set_rights(
    data: &mut WasiCtx,
    _inst: &mut Instance,
    frame: &mut CallingFrame,
    args: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, CoreError> {
    let mut mem = frame
        .memory_mut(0)
        .ok_or(CoreError::Execution(CoreExecutionError::MemoryOutOfBounds))?;

    if let Some([p1, p2, p3]) = args.get(0..3) {
        let fd = p1.to_i32();
        let fs_rights_base = p2.to_i64() as u64;
        let fs_rights_inheriting = p3.to_i64() as u64;

        Ok(to_wasm_return(p::fd_fdstat_set_rights(
            data,
            &mut mem as &mut Memory,
            fd,
            fs_rights_base,
            fs_rights_inheriting,
        )))
    } else {
        Err(CoreError::Execution(CoreExecutionError::FuncSigMismatch))
    }
}

fn fd_filestat_get(
    data: &mut WasiCtx,
    _inst: &mut Instance,
    frame: &mut CallingFrame,
    args: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, CoreError> {
    let mut mem = frame
        .memory_mut(0)
        .ok_or(CoreError::Execution(CoreExecutionError::MemoryOutOfBounds))?;

    if let Some([p1, p2]) = args.get(0..2) {
        let fd = p1.to_i32();
        let buf = p2.to_i32() as usize;

        Ok(to_wasm_return(p::fd_filestat_get(
            data,
            &mut mem as &mut Memory,
            fd,
            WasmPtr::from(buf),
        )))
    } else {
        Err(CoreError::Execution(CoreExecutionError::FuncSigMismatch))
    }
}

fn fd_filestat_set_size(
    data: &mut WasiCtx,
    _inst: &mut Instance,
    frame: &mut CallingFrame,
    args: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, CoreError> {
    let mut mem = frame
        .memory_mut(0)
        .ok_or(CoreError::Execution(CoreExecutionError::MemoryOutOfBounds))?;

    if let Some([p1, p2]) = args.get(0..2) {
        let fd = p1.to_i32();
        let buf = p2.to_i32() as usize;

        Ok(to_wasm_return(p::fd_filestat_get(
            data,
            &mut mem as &mut Memory,
            fd,
            WasmPtr::from(buf),
        )))
    } else {
        Err(CoreError::Execution(CoreExecutionError::FuncSigMismatch))
    }
}

fn fd_filestat_set_times(
    data: &mut WasiCtx,
    _inst: &mut Instance,
    frame: &mut CallingFrame,
    args: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, CoreError> {
    let mut mem = frame
        .memory_mut(0)
        .ok_or(CoreError::Execution(CoreExecutionError::MemoryOutOfBounds))?;

    if let Some([p1, p2, p3, p4]) = args.get(0..4) {
        let fd = p1.to_i32();
        let st_atim = p2.to_i64() as u64;
        let st_mtim = p3.to_i64() as u64;
        let fst_flags = p4.to_i32() as u16;

        Ok(to_wasm_return(p::fd_filestat_set_times(
            data,
            &mut mem as &mut Memory,
            fd,
            st_atim,
            st_mtim,
            fst_flags,
        )))
    } else {
        Err(CoreError::Execution(CoreExecutionError::FuncSigMismatch))
    }
}

fn fd_read(
    data: &mut WasiCtx,
    _inst: &mut Instance,
    frame: &mut CallingFrame,
    args: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, CoreError> {
    let mut mem = frame
        .memory_mut(0)
        .ok_or(CoreError::Execution(CoreExecutionError::MemoryOutOfBounds))?;

    if let Some([p1, p2, p3, p4]) = args.get(0..4) {
        let fd = p1.to_i32();
        let iovs = p2.to_i32() as usize;
        let iovs_len = p3.to_i32() as u32;
        let nread = p4.to_i32() as usize;

        Ok(to_wasm_return(p::fd_read(
            data,
            &mut mem as &mut Memory,
            fd,
            WasmPtr::from(iovs),
            iovs_len,
            WasmPtr::from(nread),
        )))
    } else {
        Err(CoreError::Execution(CoreExecutionError::FuncSigMismatch))
    }
}

fn fd_pread(
    data: &mut WasiCtx,
    _inst: &mut Instance,
    frame: &mut CallingFrame,
    args: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, CoreError> {
    let mut mem = frame
        .memory_mut(0)
        .ok_or(CoreError::Execution(CoreExecutionError::MemoryOutOfBounds))?;

    if let Some([p1, p2, p3, p4, p5]) = args.get(0..5) {
        let fd = p1.to_i32();
        let iovs = p2.to_i32() as usize;
        let iovs_len = p3.to_i32() as u32;
        let offset = p4.to_i64() as u64;
        let nread = p5.to_i32() as usize;

        Ok(to_wasm_return(p::fd_pread(
            data,
            &mut mem as &mut Memory,
            fd,
            WasmPtr::from(iovs),
            iovs_len,
            offset,
            WasmPtr::from(nread),
        )))
    } else {
        Err(CoreError::Execution(CoreExecutionError::FuncSigMismatch))
    }
}

fn fd_write(
    data: &mut WasiCtx,
    _inst: &mut Instance,
    frame: &mut CallingFrame,
    args: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, CoreError> {
    let mut mem = frame
        .memory_mut(0)
        .ok_or(CoreError::Execution(CoreExecutionError::MemoryOutOfBounds))?;

    if let Some([p1, p2, p3, p4]) = args.get(0..4) {
        let fd = p1.to_i32();
        let iovs = p2.to_i32() as usize;
        let iovs_len = p3.to_i32() as u32;
        let nwritten = p4.to_i32() as usize;

        Ok(to_wasm_return(p::fd_write(
            data,
            &mut mem as &mut Memory,
            fd,
            WasmPtr::from(iovs),
            iovs_len,
            WasmPtr::from(nwritten),
        )))
    } else {
        Err(CoreError::Execution(CoreExecutionError::FuncSigMismatch))
    }
}

fn fd_pwrite(
    data: &mut WasiCtx,
    _inst: &mut Instance,
    frame: &mut CallingFrame,
    args: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, CoreError> {
    let mut mem = frame
        .memory_mut(0)
        .ok_or(CoreError::Execution(CoreExecutionError::MemoryOutOfBounds))?;

    if let Some([p1, p2, p3, p4, p5]) = args.get(0..5) {
        let fd = p1.to_i32();
        let iovs = p2.to_i32() as usize;
        let iovs_len = p3.to_i32() as u32;
        let offset = p4.to_i64() as u64;
        let nwritten = p5.to_i32() as usize;

        Ok(to_wasm_return(p::fd_pwrite(
            data,
            &mut mem as &mut Memory,
            fd,
            WasmPtr::from(iovs),
            iovs_len,
            offset,
            WasmPtr::from(nwritten),
        )))
    } else {
        Err(CoreError::Execution(CoreExecutionError::FuncSigMismatch))
    }
}

fn fd_readdir(
    data: &mut WasiCtx,
    _inst: &mut Instance,
    frame: &mut CallingFrame,
    args: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, CoreError> {
    let mut mem = frame
        .memory_mut(0)
        .ok_or(CoreError::Execution(CoreExecutionError::MemoryOutOfBounds))?;

    if let Some([p1, p2, p3, p4, p5]) = args.get(0..5) {
        let fd = p1.to_i32();
        let buf = p2.to_i32() as usize;
        let buf_len = p3.to_i32() as u32;
        let cookie = p4.to_i64() as u64;
        let bufused_ptr = p5.to_i32() as usize;

        Ok(to_wasm_return(p::fd_readdir(
            data,
            &mut mem as &mut Memory,
            fd,
            WasmPtr::from(buf),
            buf_len,
            cookie,
            WasmPtr::from(bufused_ptr),
        )))
    } else {
        Err(CoreError::Execution(CoreExecutionError::FuncSigMismatch))
    }
}

fn path_create_directory(
    data: &mut WasiCtx,
    _inst: &mut Instance,
    frame: &mut CallingFrame,
    args: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, CoreError> {
    let mem = frame
        .memory_mut(0)
        .ok_or(CoreError::Execution(CoreExecutionError::MemoryOutOfBounds))?;

    if let Some([p1, p2, p3]) = args.get(0..3) {
        let dirfd = p1.to_i32();
        let path_ptr = p2.to_i32() as usize;
        let path_len = p3.to_i32() as u32;

        Ok(to_wasm_return(p::path_create_directory(
            data,
            &mem as &Memory,
            dirfd,
            WasmPtr::from(path_ptr),
            path_len,
        )))
    } else {
        Err(CoreError::Execution(CoreExecutionError::FuncSigMismatch))
    }
}

fn path_filestat_get(
    data: &mut WasiCtx,
    _inst: &mut Instance,
    frame: &mut CallingFrame,
    args: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, CoreError> {
    let mut mem = frame
        .memory_mut(0)
        .ok_or(CoreError::Execution(CoreExecutionError::MemoryOutOfBounds))?;

    if let Some([p1, p2, p3, p4, p5]) = args.get(0..5) {
        let fd = p1.to_i32();
        let flags = p2.to_i32() as u32;
        let path_ptr = p3.to_i32() as usize;
        let path_len = p4.to_i32() as u32;
        let file_stat_ptr = p5.to_i32() as usize;

        Ok(to_wasm_return(p::path_filestat_get(
            data,
            &mut mem as &mut Memory,
            fd,
            flags,
            WasmPtr::from(path_ptr),
            path_len,
            WasmPtr::from(file_stat_ptr),
        )))
    } else {
        Err(CoreError::Execution(CoreExecutionError::FuncSigMismatch))
    }
}

fn path_filestat_set_times(
    _data: &mut WasiCtx,
    _inst: &mut Instance,
    _frame: &mut CallingFrame,
    _args: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, CoreError> {
    Ok(vec![WasmValue::from_i32(
        Errno::__WASI_ERRNO_NOSYS.0 as i32,
    )])
}

fn path_link(
    _data: &mut WasiCtx,
    _inst: &mut Instance,
    _frame: &mut CallingFrame,
    _args: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, CoreError> {
    Ok(vec![WasmValue::from_i32(
        Errno::__WASI_ERRNO_NOSYS.0 as i32,
    )])
}

fn path_open(
    data: &mut WasiCtx,
    _inst: &mut Instance,
    frame: &mut CallingFrame,
    args: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, CoreError> {
    let mut mem = frame
        .memory_mut(0)
        .ok_or(CoreError::Execution(CoreExecutionError::MemoryOutOfBounds))?;

    if let Some([p1, p2, p3, p4, p5, p6, p7, p8, p9]) = args.get(0..9) {
        let dirfd = p1.to_i32();
        let dirflags = p2.to_i32() as u32;
        let path = p3.to_i32() as usize;
        let path_len = p4.to_i32() as u32;
        let o_flags = p5.to_i32() as u16;
        let fs_rights_base = p6.to_i64() as u64;
        let fs_rights_inheriting = p7.to_i64() as u64;
        let fs_flags = p8.to_i32() as u16;
        let fd_ptr = p9.to_i32() as usize;

        Ok(to_wasm_return(p::path_open(
            data,
            &mut mem as &mut Memory,
            dirfd,
            dirflags,
            WasmPtr::from(path),
            path_len,
            o_flags,
            fs_rights_base,
            fs_rights_inheriting,
            fs_flags,
            WasmPtr::from(fd_ptr),
        )))
    } else {
        Err(CoreError::Execution(CoreExecutionError::FuncSigMismatch))
    }
}

fn path_readlink(
    _data: &mut WasiCtx,
    _inst: &mut Instance,
    _frame: &mut CallingFrame,
    _args: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, CoreError> {
    Ok(vec![WasmValue::from_i32(
        Errno::__WASI_ERRNO_NOSYS.0 as i32,
    )])
}

fn path_remove_directory(
    data: &mut WasiCtx,
    _inst: &mut Instance,
    frame: &mut CallingFrame,
    args: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, CoreError> {
    let mem = frame
        .memory_mut(0)
        .ok_or(CoreError::Execution(CoreExecutionError::MemoryOutOfBounds))?;

    if let Some([p1, p2, p3]) = args.get(0..3) {
        let fd = p1.to_i32();
        let path_ptr = p2.to_i32() as usize;
        let path_len = p3.to_i32() as u32;

        Ok(to_wasm_return(p::path_remove_directory(
            data,
            &mem as &Memory,
            fd,
            WasmPtr::from(path_ptr),
            path_len,
        )))
    } else {
        Err(CoreError::Execution(CoreExecutionError::FuncSigMismatch))
    }
}

fn path_rename(
    data: &mut WasiCtx,
    _inst: &mut Instance,
    frame: &mut CallingFrame,
    args: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, CoreError> {
    let mem = frame
        .memory_mut(0)
        .ok_or(CoreError::Execution(CoreExecutionError::MemoryOutOfBounds))?;

    if let Some([p1, p2, p3, p4, p5, p6]) = args.get(0..6) {
        let old_fd = p1.to_i32();
        let old_path = p2.to_i32() as usize;
        let old_path_len = p3.to_i32() as u32;
        let new_fd = p4.to_i32();
        let new_path = p5.to_i32() as usize;
        let new_path_len = p6.to_i32() as u32;

        Ok(to_wasm_return(p::path_rename(
            data,
            &mem as &Memory,
            old_fd,
            WasmPtr::from(old_path),
            old_path_len,
            new_fd,
            WasmPtr::from(new_path),
            new_path_len,
        )))
    } else {
        Err(CoreError::Execution(CoreExecutionError::FuncSigMismatch))
    }
}

fn path_symlink(
    _data: &mut WasiCtx,
    _inst: &mut Instance,
    _frame: &mut CallingFrame,
    _args: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, CoreError> {
    Ok(vec![WasmValue::from_i32(
        Errno::__WASI_ERRNO_NOSYS.0 as i32,
    )])
}

fn path_unlink_file(
    data: &mut WasiCtx,
    _inst: &mut Instance,
    frame: &mut CallingFrame,
    args: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, CoreError> {
    let mem = frame
        .memory_mut(0)
        .ok_or(CoreError::Execution(CoreExecutionError::MemoryOutOfBounds))?;

    if let Some([p1, p2, p3]) = args.get(0..3) {
        let fd = p1.to_i32();
        let path_ptr = p2.to_i32() as usize;
        let path_len = p3.to_i32() as u32;

        Ok(to_wasm_return(p::path_unlink_file(
            data,
            &mem as &Memory,
            fd,
            WasmPtr::from(path_ptr),
            path_len,
        )))
    } else {
        Err(CoreError::Execution(CoreExecutionError::FuncSigMismatch))
    }
}

fn proc_exit(
    data: &mut WasiCtx,
    _inst: &mut Instance,
    frame: &mut CallingFrame,
    args: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, CoreError> {
    let mut mem = frame
        .memory_mut(0)
        .ok_or(CoreError::Execution(CoreExecutionError::MemoryOutOfBounds))?;

    if let Some([p1]) = args.get(0..1) {
        let code = p1.to_i32() as u32;
        p::proc_exit(data, &mut mem as &mut Memory, code);
        Err(CoreError::Common(CoreCommonError::Terminated))
    } else {
        Err(CoreError::Execution(CoreExecutionError::FuncSigMismatch))
    }
}

fn proc_raise(
    _data: &mut WasiCtx,
    _inst: &mut Instance,
    _frame: &mut CallingFrame,
    _args: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, CoreError> {
    Ok(vec![WasmValue::from_i32(
        Errno::__WASI_ERRNO_NOSYS.0 as i32,
    )])
}

// todo: ld asyncify yield

fn sched_yield(
    _data: &mut WasiCtx,
    _inst: &mut Instance,
    _frame: &mut CallingFrame,
    _args: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, CoreError> {
    Ok(vec![WasmValue::from_i32(
        Errno::__WASI_ERRNO_NOSYS.0 as i32,
    )])
}

//socket

fn sock_open(
    data: &mut WasiCtx,
    _inst: &mut Instance,
    frame: &mut CallingFrame,
    args: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, CoreError> {
    let mut mem = frame
        .memory_mut(0)
        .ok_or(CoreError::Execution(CoreExecutionError::MemoryOutOfBounds))?;

    if let Some([p1, p2, p3]) = args.get(0..3) {
        let af = p1.to_i32() as u8;
        let ty = p2.to_i32() as u8;
        let ro_fd_ptr = p3.to_i32() as usize;

        Ok(to_wasm_return(p::async_socket::sock_open(
            data,
            &mut mem as &mut Memory,
            af,
            ty,
            WasmPtr::from(ro_fd_ptr),
        )))
    } else {
        Err(CoreError::Execution(CoreExecutionError::FuncSigMismatch))
    }
}

fn sock_bind(
    data: &mut WasiCtx,
    _inst: &mut Instance,
    frame: &mut CallingFrame,
    args: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, CoreError> {
    let mem = frame
        .memory_mut(0)
        .ok_or(CoreError::Execution(CoreExecutionError::MemoryOutOfBounds))?;

    if let Some([p1, p2, p3]) = args.get(0..3) {
        let fd = p1.to_i32();
        let addr_ptr = p2.to_i32() as usize;
        let port = p3.to_i32() as u32;
        Ok(to_wasm_return(p::async_socket::sock_bind(
            data,
            &mem as &Memory,
            fd,
            WasmPtr::from(addr_ptr),
            port,
        )))
    } else {
        Err(CoreError::Execution(CoreExecutionError::FuncSigMismatch))
    }
}

fn sock_listen(
    data: &mut WasiCtx,
    _inst: &mut Instance,
    frame: &mut CallingFrame,
    args: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, CoreError> {
    let mut mem = frame
        .memory_mut(0)
        .ok_or(CoreError::Execution(CoreExecutionError::MemoryOutOfBounds))?;

    if let Some([p1, p2]) = args.get(0..2) {
        let fd = p1.to_i32();
        let backlog = p2.to_i32() as u32;

        Ok(to_wasm_return(p::async_socket::sock_listen(
            data,
            &mut mem as &mut Memory,
            fd,
            backlog,
        )))
    } else {
        Err(CoreError::Execution(CoreExecutionError::FuncSigMismatch))
    }
}

fn sock_accept<'data, 'inst, 'frame, 'fut>(
    data: &'data mut WasiCtx,
    _inst: &'inst mut AsyncInstance,
    frame: &'frame mut CallingFrame,
    args: Vec<WasmValue>,
) -> Box<dyn Future<Output = Result<Vec<WasmValue>, CoreError>> + Send + 'fut>
where
    'data: 'fut,
    'frame: 'fut,
    'inst: 'fut,
{
    Box::new(async move {
        let mut mem = frame
            .memory_mut(0)
            .ok_or(CoreError::Execution(CoreExecutionError::MemoryOutOfBounds))?;

        if let Some([p1, p2]) = args.get(0..2) {
            let fd = p1.to_i32();
            let ro_fd_ptr = p2.to_i32() as usize;

            Ok(to_wasm_return(
                p::async_socket::sock_accept(
                    data,
                    &mut mem as &mut Memory,
                    fd,
                    WasmPtr::from(ro_fd_ptr),
                )
                .await,
            ))
        } else {
            Err(CoreError::Execution(CoreExecutionError::FuncSigMismatch))
        }
    })
}

async fn sock_connect(
    data: &mut WasiCtx,
    _inst: &mut AsyncInstance,
    frame: &mut CallingFrame,
    args: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, CoreError> {
    let mem = frame
        .memory_mut(0)
        .ok_or(CoreError::Execution(CoreExecutionError::MemoryOutOfBounds))?;

    if let Some([p1, p2, p3]) = args.get(0..3) {
        let fd = p1.to_i32();
        let addr_ptr = p2.to_i32() as usize;
        let port = p3.to_i32() as u32;

        Ok(to_wasm_return(
            p::async_socket::sock_connect(data, &mem as &Memory, fd, WasmPtr::from(addr_ptr), port)
                .await,
        ))
    } else {
        Err(CoreError::Execution(CoreExecutionError::FuncSigMismatch))
    }
}

async fn sock_recv(
    data: &mut WasiCtx,
    _inst: &mut AsyncInstance,
    frame: &mut CallingFrame,
    args: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, CoreError> {
    let mut mem = frame
        .memory_mut(0)
        .ok_or(CoreError::Execution(CoreExecutionError::MemoryOutOfBounds))?;

    if let Some([p1, p2, p3, p4, p5, p6]) = args.get(0..6) {
        let fd = p1.to_i32();
        let buf_ptr = p2.to_i32() as usize;
        let buf_len = p3.to_i32() as u32;
        let flags = p4.to_i32() as u16;
        let ro_data_len_ptr = p5.to_i32() as usize;
        let ro_flags_ptr = p6.to_i32() as usize;

        Ok(to_wasm_return(
            p::async_socket::sock_recv(
                data,
                &mut mem as &mut Memory,
                fd,
                WasmPtr::from(buf_ptr),
                buf_len,
                flags,
                WasmPtr::from(ro_data_len_ptr),
                WasmPtr::from(ro_flags_ptr),
            )
            .await,
        ))
    } else {
        Err(CoreError::Execution(CoreExecutionError::FuncSigMismatch))
    }
}

async fn sock_recv_from(
    data: &mut WasiCtx,
    _inst: &mut AsyncInstance,
    frame: &mut CallingFrame,
    args: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, CoreError> {
    let mut mem = frame
        .memory_mut(0)
        .ok_or(CoreError::Execution(CoreExecutionError::MemoryOutOfBounds))?;

    if let Some([p1, p2, p3, p4, p5, p6, p7, p8]) = args.get(0..8) {
        let fd = p1.to_i32();
        let buf_ptr = p2.to_i32() as usize;
        let buf_len = p3.to_i32() as u32;
        let wasi_addr_ptr = p4.to_i32() as usize;
        let flags = p5.to_i32() as u16;
        let port_ptr = p6.to_i32() as usize;
        let ro_data_len_ptr = p7.to_i32() as usize;
        let ro_flags_ptr = p8.to_i32() as usize;

        Ok(to_wasm_return(
            p::async_socket::sock_recv_from(
                data,
                &mut mem as &mut Memory,
                fd,
                WasmPtr::from(buf_ptr),
                buf_len,
                WasmPtr::from(wasi_addr_ptr),
                flags,
                WasmPtr::from(port_ptr),
                WasmPtr::from(ro_data_len_ptr),
                WasmPtr::from(ro_flags_ptr),
            )
            .await,
        ))
    } else {
        Err(CoreError::Execution(CoreExecutionError::FuncSigMismatch))
    }
}

async fn sock_send(
    data: &mut WasiCtx,
    _inst: &mut AsyncInstance,
    frame: &mut CallingFrame,
    args: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, CoreError> {
    let mut mem = frame
        .memory_mut(0)
        .ok_or(CoreError::Execution(CoreExecutionError::MemoryOutOfBounds))?;

    if let Some([p1, p2, p3, p4, p5]) = args.get(0..5) {
        let fd = p1.to_i32();
        let buf_ptr = p2.to_i32() as usize;
        let buf_len = p3.to_i32() as u32;
        let flags = p4.to_i32() as u16;
        let send_len_ptr = p5.to_i32() as usize;

        Ok(to_wasm_return(
            p::async_socket::sock_send(
                data,
                &mut mem as &mut Memory,
                fd,
                WasmPtr::from(buf_ptr),
                buf_len,
                flags,
                WasmPtr::from(send_len_ptr),
            )
            .await,
        ))
    } else {
        Err(CoreError::Execution(CoreExecutionError::FuncSigMismatch))
    }
}

async fn sock_send_to(
    data: &mut WasiCtx,
    _inst: &mut AsyncInstance,
    frame: &mut CallingFrame,
    args: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, CoreError> {
    let mut mem = frame
        .memory_mut(0)
        .ok_or(CoreError::Execution(CoreExecutionError::MemoryOutOfBounds))?;

    if let Some([p1, p2, p3, p4, p5, p6, p7]) = args.get(0..7) {
        let fd = p1.to_i32();
        let buf_ptr = p2.to_i32() as usize;
        let buf_len = p3.to_i32() as u32;
        let wasi_addr_ptr = p4.to_i32() as usize;
        let port = p5.to_i32() as u32;
        let flags = p6.to_i32() as u16;
        let send_len_ptr = p7.to_i32() as usize;

        Ok(to_wasm_return(
            p::async_socket::sock_send_to(
                data,
                &mut mem as &mut Memory,
                fd,
                WasmPtr::from(buf_ptr),
                buf_len,
                WasmPtr::from(wasi_addr_ptr),
                port,
                flags,
                WasmPtr::from(send_len_ptr),
            )
            .await,
        ))
    } else {
        Err(CoreError::Execution(CoreExecutionError::FuncSigMismatch))
    }
}

fn sock_shutdown(
    data: &mut WasiCtx,
    _inst: &mut Instance,
    frame: &mut CallingFrame,
    args: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, CoreError> {
    let mut mem = frame
        .memory_mut(0)
        .ok_or(CoreError::Execution(CoreExecutionError::MemoryOutOfBounds))?;

    if let Some([p1, p2]) = args.get(0..2) {
        let fd = p1.to_i32();
        let how = p2.to_i32() as u8;
        Ok(to_wasm_return(p::async_socket::sock_shutdown(
            data,
            &mut mem as &mut Memory,
            fd,
            how,
        )))
    } else {
        Err(CoreError::Execution(CoreExecutionError::FuncSigMismatch))
    }
}

fn sock_getpeeraddr(
    data: &mut WasiCtx,
    _inst: &mut Instance,
    frame: &mut CallingFrame,
    args: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, CoreError> {
    let mut mem = frame
        .memory_mut(0)
        .ok_or(CoreError::Execution(CoreExecutionError::MemoryOutOfBounds))?;

    if let Some([p1, p2, p3, p4]) = args.get(0..4) {
        let fd = p1.to_i32();
        let wasi_addr_ptr = p2.to_i32() as usize;
        let addr_type = p3.to_i32() as usize;
        let port_ptr = p4.to_i32() as usize;
        Ok(to_wasm_return(p::async_socket::sock_getpeeraddr(
            data,
            &mut mem as &mut Memory,
            fd,
            WasmPtr::from(wasi_addr_ptr),
            WasmPtr::from(addr_type),
            WasmPtr::from(port_ptr),
        )))
    } else {
        Err(CoreError::Execution(CoreExecutionError::FuncSigMismatch))
    }
}

fn sock_getlocaladdr(
    data: &mut WasiCtx,
    _inst: &mut Instance,
    frame: &mut CallingFrame,
    args: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, CoreError> {
    let mut mem = frame
        .memory_mut(0)
        .ok_or(CoreError::Execution(CoreExecutionError::MemoryOutOfBounds))?;

    if let Some([p1, p2, p3, p4]) = args.get(0..4) {
        let fd = p1.to_i32();
        let wasi_addr_ptr = p2.to_i32() as usize;
        let addr_type = p3.to_i32() as usize;
        let port_ptr = p4.to_i32() as usize;
        Ok(to_wasm_return(p::async_socket::sock_getlocaladdr(
            data,
            &mut mem as &mut Memory,
            fd,
            WasmPtr::from(wasi_addr_ptr),
            WasmPtr::from(addr_type),
            WasmPtr::from(port_ptr),
        )))
    } else {
        Err(CoreError::Execution(CoreExecutionError::FuncSigMismatch))
    }
}

fn sock_getsockopt(
    data: &mut WasiCtx,
    _inst: &mut Instance,
    frame: &mut CallingFrame,
    args: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, CoreError> {
    let mut mem = frame
        .memory_mut(0)
        .ok_or(CoreError::Execution(CoreExecutionError::MemoryOutOfBounds))?;

    if let Some([p1, p2, p3, p4, p5]) = args.get(0..5) {
        let fd = p1.to_i32();
        let level = p2.to_i32() as u32;
        let name = p3.to_i32() as u32;
        let flag = p4.to_i32() as usize;
        let flag_size_ptr = p5.to_i32() as usize;
        Ok(to_wasm_return(p::async_socket::sock_getsockopt(
            data,
            &mut mem as &mut Memory,
            fd,
            level,
            name,
            WasmPtr::from(flag),
            WasmPtr::from(flag_size_ptr),
        )))
    } else {
        Err(CoreError::Execution(CoreExecutionError::FuncSigMismatch))
    }
}

fn sock_setsockopt(
    data: &mut WasiCtx,
    _inst: &mut Instance,
    frame: &mut CallingFrame,
    args: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, CoreError> {
    let mem = frame
        .memory_mut(0)
        .ok_or(CoreError::Execution(CoreExecutionError::MemoryOutOfBounds))?;

    if let Some([p1, p2, p3, p4, p5]) = args.get(0..5) {
        let fd = p1.to_i32();
        let level = p2.to_i32() as u32;
        let name = p3.to_i32() as u32;
        let flag = p4.to_i32() as usize;
        let flag_size = p5.to_i32() as u32;
        Ok(to_wasm_return(p::async_socket::sock_setsockopt(
            data,
            &mem as &Memory,
            fd,
            level,
            name,
            WasmPtr::from(flag),
            flag_size,
        )))
    } else {
        Err(CoreError::Execution(CoreExecutionError::FuncSigMismatch))
    }
}

fn sock_getaddrinfo(
    data: &mut WasiCtx,
    _inst: &mut Instance,
    frame: &mut CallingFrame,
    args: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, CoreError> {
    let mut mem = frame
        .memory_mut(0)
        .ok_or(CoreError::Execution(CoreExecutionError::MemoryOutOfBounds))?;

    if let Some([p1, p2, p3, p4, p5, p6, p7, p8]) = args.get(0..8) {
        let node = p1.to_i32() as usize;
        let node_len = p2.to_i32() as u32;
        let server = p3.to_i32() as usize;
        let server_len = p4.to_i32() as u32;
        let hint = p5.to_i32() as usize;
        let res = p6.to_i32() as usize;
        let max_len = p7.to_i32() as u32;
        let res_len = p8.to_i32() as usize;

        Ok(to_wasm_return(p::async_socket::addrinfo::sock_getaddrinfo(
            data,
            &mut mem as &mut Memory,
            WasmPtr::from(node),
            node_len,
            WasmPtr::from(server),
            server_len,
            WasmPtr::from(hint),
            WasmPtr::from(res),
            max_len,
            WasmPtr::from(res_len),
        )))
    } else {
        Err(CoreError::Execution(CoreExecutionError::FuncSigMismatch))
    }
}

async fn poll_oneoff(
    data: &mut WasiCtx,
    _inst: &mut AsyncInstance,
    frame: &mut CallingFrame,
    args: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, CoreError> {
    let mut mem = frame
        .memory_mut(0)
        .ok_or(CoreError::Execution(CoreExecutionError::MemoryOutOfBounds))?;

    if let Some([p1, p2, p3, p4]) = args.get(0..4) {
        let in_ptr = p1.to_i32() as usize;
        let out_ptr = p2.to_i32() as usize;
        let nsubscriptions = p3.to_i32() as u32;
        let revents_num_ptr = p4.to_i32() as usize;

        Ok(to_wasm_return(
            p::async_poll::poll_oneoff(
                data,
                &mut mem as &mut Memory,
                WasmPtr::from(in_ptr),
                WasmPtr::from(out_ptr),
                nsubscriptions,
                WasmPtr::from(revents_num_ptr),
            )
            .await,
        ))
    } else {
        Err(CoreError::Execution(CoreExecutionError::FuncSigMismatch))
    }
}

async fn sock_lookup_ip(
    data: &mut WasiCtx,
    _inst: &mut AsyncInstance,
    frame: &mut CallingFrame,
    args: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, CoreError> {
    let mut mem = frame
        .memory_mut(0)
        .ok_or(CoreError::Execution(CoreExecutionError::MemoryOutOfBounds))?;

    if let Some([p1, p2, p3, p4, p5, p6]) = args.get(0..6) {
        let host_name_ptr = p1.to_i32() as usize;
        let host_name_len = p2.to_i32() as u32;
        let lookup_type = p3.to_i32() as u8;
        let addr_buf = p4.to_i32() as usize;
        let addr_buf_max_len = p5.to_i32() as u32;
        let raddr_num_ptr = p6.to_i32() as usize;
        Ok(to_wasm_return(
            p::async_socket::sock_lookup_ip(
                data,
                &mut mem as &mut Memory,
                WasmPtr::from(host_name_ptr),
                host_name_len,
                lookup_type,
                WasmPtr::from(addr_buf),
                addr_buf_max_len,
                WasmPtr::from(raddr_num_ptr),
            )
            .await,
        ))
    } else {
        Err(CoreError::Execution(CoreExecutionError::FuncSigMismatch))
    }
}

#[inline]
fn box_future<
    'data,
    'inst,
    'frame,
    'fut,
    Data: Send,
    Fut: Future<Output = Result<Vec<WasmValue>, CoreError>> + Send + 'fut,
    F: FnOnce(
        &'data mut Data,
        &'inst mut AsyncInstance,
        &'frame mut CallingFrame,
        Vec<WasmValue>,
    ) -> Fut,
>(
    data: &'data mut Data,
    inst: &'inst mut AsyncInstance,
    frame: &'frame mut CallingFrame,
    args: Vec<WasmValue>,
) -> Box<dyn Future<Output = Result<Vec<WasmValue>, CoreError>> + Send + 'fut>
where
    'data: 'fut,
    'inst: 'fut,
    'frame: 'fut,
{
    let f: F = unsafe { std::mem::zeroed() };
    Box::new(f(data, inst, frame, args))
}

#[allow(clippy::complexity)]
fn wrap_future<
    'data,
    'inst,
    'frame,
    'fut,
    Data: Send + 'data,
    Fut: Future<Output = Result<Vec<WasmValue>, CoreError>> + Send + 'fut,
    F: FnOnce(
        &'data mut Data,
        &'inst mut AsyncInstance,
        &'frame mut CallingFrame,
        Vec<WasmValue>,
    ) -> Fut,
>(
    _f: F,
) -> fn(
    data: &'data mut Data,
    inst: &'inst mut AsyncInstance,
    frame: &'frame mut CallingFrame,
    args: Vec<WasmValue>,
) -> Box<dyn Future<Output = Result<Vec<WasmValue>, CoreError>> + Send + 'fut>
where
    'data: 'fut,
    'inst: 'fut,
    'frame: 'fut,
{
    box_future::<Data, Fut, F>
}

enum WasiFunc<'data, 'inst, 'frame, 'fut, T: Sized>
where
    'data: 'fut,
    'inst: 'fut,
    'frame: 'fut,
{
    SyncFn(String, (Vec<ValType>, Vec<ValType>), SyncFn<T>),
    AsyncFn(
        String,
        (Vec<ValType>, Vec<ValType>),
        AsyncFn<'data, 'inst, 'frame, 'fut, T>,
    ),
}

fn wasi_impls<'data, 'inst, 'frame, 'fut>() -> Vec<WasiFunc<'data, 'inst, 'frame, 'fut, WasiCtx>> {
    macro_rules! sync_fn {
        ($name:expr, $ty:expr, $f:ident) => {
            WasiFunc::SyncFn($name.into(), $ty, $f)
        };
    }
    macro_rules! async_fn {
        ($name:expr, $ty:expr, $f:expr) => {
            WasiFunc::AsyncFn($name.into(), $ty, $f)
        };
    }
    vec![
        sync_fn!(
            "args_get",
            (vec![ValType::I32, ValType::I32], vec![ValType::I32]),
            args_get
        ),
        sync_fn!(
            "args_sizes_get",
            (vec![ValType::I32, ValType::I32], vec![ValType::I32]),
            args_sizes_get
        ),
        sync_fn!(
            "environ_get",
            (vec![ValType::I32, ValType::I32], vec![ValType::I32]),
            environ_get
        ),
        sync_fn!(
            "environ_sizes_get",
            (vec![ValType::I32, ValType::I32], vec![ValType::I32]),
            environ_sizes_get
        ),
        sync_fn!(
            "clock_res_get",
            (vec![ValType::I32, ValType::I64], vec![ValType::I32]),
            clock_res_get
        ),
        sync_fn!(
            "clock_time_get",
            (
                vec![ValType::I32, ValType::I64, ValType::I32],
                vec![ValType::I32],
            ),
            clock_time_get
        ),
        sync_fn!(
            "random_get",
            (vec![ValType::I32, ValType::I32], vec![ValType::I32]),
            random_get
        ),
        sync_fn!(
            "fd_prestat_get",
            (vec![ValType::I32, ValType::I32], vec![ValType::I32]),
            fd_prestat_get
        ),
        sync_fn!(
            "fd_prestat_dir_name",
            (
                vec![ValType::I32, ValType::I32, ValType::I32],
                vec![ValType::I32],
            ),
            fd_prestat_dir_name
        ),
        sync_fn!(
            "fd_renumber",
            (vec![ValType::I32, ValType::I32], vec![ValType::I32]),
            fd_renumber
        ),
        sync_fn!(
            "fd_advise",
            (
                vec![ValType::I32, ValType::I64, ValType::I64, ValType::I32],
                vec![ValType::I32],
            ),
            fd_advise
        ),
        sync_fn!(
            "fd_allocate",
            (
                vec![ValType::I32, ValType::I64, ValType::I64],
                vec![ValType::I32],
            ),
            fd_allocate
        ),
        sync_fn!(
            "fd_close",
            (vec![ValType::I32], vec![ValType::I32]),
            fd_close
        ),
        sync_fn!(
            "fd_seek",
            (
                vec![ValType::I32, ValType::I64, ValType::I32, ValType::I32],
                vec![ValType::I32],
            ),
            fd_seek
        ),
        sync_fn!("fd_sync", (vec![ValType::I32], vec![ValType::I32]), fd_sync),
        sync_fn!(
            "fd_datasync",
            (vec![ValType::I32], vec![ValType::I32]),
            fd_datasync
        ),
        sync_fn!(
            "fd_tell",
            (vec![ValType::I32, ValType::I32], vec![ValType::I32]),
            fd_tell
        ),
        sync_fn!(
            "fd_fdstat_get",
            (vec![ValType::I32, ValType::I32], vec![ValType::I32]),
            fd_fdstat_get
        ),
        sync_fn!(
            "fd_fdstat_set_flags",
            (vec![ValType::I32, ValType::I32], vec![ValType::I32]),
            fd_fdstat_set_flags
        ),
        sync_fn!(
            "fd_fdstat_set_rights",
            (
                vec![ValType::I32, ValType::I64, ValType::I64],
                vec![ValType::I32],
            ),
            fd_fdstat_set_rights
        ),
        sync_fn!(
            "fd_filestat_get",
            (vec![ValType::I32, ValType::I32], vec![ValType::I32]),
            fd_filestat_get
        ),
        sync_fn!(
            "fd_filestat_set_size",
            (vec![ValType::I32, ValType::I32], vec![ValType::I32]),
            fd_filestat_set_size
        ),
        sync_fn!(
            "fd_filestat_set_times",
            (
                vec![ValType::I32, ValType::I64, ValType::I64, ValType::I32],
                vec![ValType::I32],
            ),
            fd_filestat_set_times
        ),
        sync_fn!(
            "fd_read",
            (
                vec![ValType::I32, ValType::I32, ValType::I32, ValType::I32],
                vec![ValType::I32],
            ),
            fd_read
        ),
        sync_fn!(
            "fd_pread",
            (
                vec![
                    ValType::I32,
                    ValType::I32,
                    ValType::I32,
                    ValType::I64,
                    ValType::I32,
                ],
                vec![ValType::I32],
            ),
            fd_pread
        ),
        sync_fn!(
            "fd_write",
            (
                vec![ValType::I32, ValType::I32, ValType::I32, ValType::I32],
                vec![ValType::I32],
            ),
            fd_write
        ),
        sync_fn!(
            "fd_pwrite",
            (
                vec![
                    ValType::I32,
                    ValType::I32,
                    ValType::I32,
                    ValType::I64,
                    ValType::I32,
                ],
                vec![ValType::I32],
            ),
            fd_pwrite
        ),
        sync_fn!(
            "fd_readdir",
            (
                vec![
                    ValType::I32,
                    ValType::I32,
                    ValType::I32,
                    ValType::I64,
                    ValType::I32,
                ],
                vec![ValType::I32],
            ),
            fd_readdir
        ),
        sync_fn!(
            "path_create_directory",
            (
                vec![ValType::I32, ValType::I32, ValType::I32],
                vec![ValType::I32],
            ),
            path_create_directory
        ),
        sync_fn!(
            "path_filestat_get",
            (
                vec![
                    ValType::I32,
                    ValType::I32,
                    ValType::I32,
                    ValType::I32,
                    ValType::I32,
                ],
                vec![ValType::I32],
            ),
            path_filestat_get
        ),
        sync_fn!(
            "path_filestat_set_times",
            (
                vec![
                    ValType::I32,
                    ValType::I32,
                    ValType::I32,
                    ValType::I32,
                    ValType::I64,
                    ValType::I64,
                    ValType::I32,
                ],
                vec![ValType::I32],
            ),
            path_filestat_set_times
        ),
        sync_fn!(
            "path_link",
            (
                vec![
                    ValType::I32,
                    ValType::I32,
                    ValType::I32,
                    ValType::I32,
                    ValType::I32,
                    ValType::I32,
                    ValType::I32,
                ],
                vec![ValType::I32],
            ),
            path_link
        ),
        sync_fn!(
            "path_open",
            (
                vec![
                    ValType::I32,
                    ValType::I32,
                    ValType::I32,
                    ValType::I32,
                    ValType::I32,
                    ValType::I64,
                    ValType::I64,
                    ValType::I32,
                    ValType::I32,
                ],
                vec![ValType::I32],
            ),
            path_open
        ),
        sync_fn!(
            "path_readlink",
            (
                vec![
                    ValType::I32,
                    ValType::I32,
                    ValType::I32,
                    ValType::I32,
                    ValType::I32,
                    ValType::I32,
                ],
                vec![ValType::I32],
            ),
            path_readlink
        ),
        sync_fn!(
            "path_remove_directory",
            (
                vec![ValType::I32, ValType::I32, ValType::I32],
                vec![ValType::I32],
            ),
            path_remove_directory
        ),
        sync_fn!(
            "path_rename",
            (
                vec![
                    ValType::I32,
                    ValType::I32,
                    ValType::I32,
                    ValType::I32,
                    ValType::I32,
                    ValType::I32,
                ],
                vec![ValType::I32],
            ),
            path_rename
        ),
        sync_fn!(
            "path_symlink",
            (
                vec![
                    ValType::I32,
                    ValType::I32,
                    ValType::I32,
                    ValType::I32,
                    ValType::I32,
                ],
                vec![ValType::I32],
            ),
            path_symlink
        ),
        sync_fn!(
            "path_unlink_file",
            (
                vec![ValType::I32, ValType::I32, ValType::I32],
                vec![ValType::I32],
            ),
            path_unlink_file
        ),
        sync_fn!("proc_exit", (vec![ValType::I32], vec![]), proc_exit),
        sync_fn!(
            "proc_raise",
            (vec![ValType::I32], vec![ValType::I32]),
            proc_raise
        ),
        sync_fn!("sched_yield", (vec![], vec![ValType::I32]), sched_yield),
        sync_fn!(
            "sock_open",
            (
                vec![ValType::I32, ValType::I32, ValType::I32],
                vec![ValType::I32],
            ),
            sock_open
        ),
        sync_fn!(
            "sock_bind",
            (
                vec![ValType::I32, ValType::I32, ValType::I32],
                vec![ValType::I32],
            ),
            sock_bind
        ),
        sync_fn!(
            "sock_listen",
            (vec![ValType::I32, ValType::I32], vec![ValType::I32]),
            sock_listen
        ),
        async_fn!(
            "sock_accept",
            (vec![ValType::I32, ValType::I32], vec![ValType::I32]),
            sock_accept
        ),
        async_fn!(
            "sock_connect",
            (
                vec![ValType::I32, ValType::I32, ValType::I32],
                vec![ValType::I32],
            ),
            wrap_future(sock_connect)
        ),
        async_fn!(
            "sock_recv",
            (
                vec![
                    ValType::I32,
                    ValType::I32,
                    ValType::I32,
                    ValType::I32,
                    ValType::I32,
                    ValType::I32,
                ],
                vec![ValType::I32],
            ),
            wrap_future(sock_recv)
        ),
        async_fn!(
            "sock_recv_from",
            (
                vec![
                    ValType::I32,
                    ValType::I32,
                    ValType::I32,
                    ValType::I32,
                    ValType::I32,
                    ValType::I32,
                    ValType::I32,
                    ValType::I32,
                ],
                vec![ValType::I32],
            ),
            wrap_future(sock_recv_from)
        ),
        async_fn!(
            "sock_send",
            (
                vec![
                    ValType::I32,
                    ValType::I32,
                    ValType::I32,
                    ValType::I32,
                    ValType::I32,
                ],
                vec![ValType::I32],
            ),
            wrap_future(sock_send)
        ),
        async_fn!(
            "sock_send_to",
            (
                vec![
                    ValType::I32,
                    ValType::I32,
                    ValType::I32,
                    ValType::I32,
                    ValType::I32,
                    ValType::I32,
                    ValType::I32,
                ],
                vec![ValType::I32],
            ),
            wrap_future(sock_send_to)
        ),
        sync_fn!(
            "sock_shutdown",
            (vec![ValType::I32, ValType::I32], vec![ValType::I32]),
            sock_shutdown
        ),
        sync_fn!(
            "sock_getpeeraddr",
            (
                vec![ValType::I32, ValType::I32, ValType::I32, ValType::I32],
                vec![ValType::I32],
            ),
            sock_getpeeraddr
        ),
        sync_fn!(
            "sock_getlocaladdr",
            (
                vec![ValType::I32, ValType::I32, ValType::I32, ValType::I32],
                vec![ValType::I32],
            ),
            sock_getlocaladdr
        ),
        sync_fn!(
            "sock_getsockopt",
            (
                vec![
                    ValType::I32,
                    ValType::I32,
                    ValType::I32,
                    ValType::I32,
                    ValType::I32,
                ],
                vec![ValType::I32],
            ),
            sock_getsockopt
        ),
        sync_fn!(
            "sock_setsockopt",
            (
                vec![
                    ValType::I32,
                    ValType::I32,
                    ValType::I32,
                    ValType::I32,
                    ValType::I32,
                ],
                vec![ValType::I32],
            ),
            sock_setsockopt
        ),
        async_fn!(
            "poll_oneoff",
            (
                vec![ValType::I32, ValType::I32, ValType::I32, ValType::I32],
                vec![ValType::I32],
            ),
            wrap_future(poll_oneoff)
        ),
        async_fn!(
            "epoll_oneoff",
            (
                vec![ValType::I32, ValType::I32, ValType::I32, ValType::I32],
                vec![ValType::I32],
            ),
            wrap_future(poll_oneoff)
        ),
        async_fn!(
            "sock_lookup_ip",
            (
                vec![
                    ValType::I32,
                    ValType::I32,
                    ValType::I32,
                    ValType::I32,
                    ValType::I32,
                    ValType::I32,
                ],
                vec![ValType::I32],
            ),
            wrap_future(sock_lookup_ip)
        ),
        sync_fn!(
            "sock_getaddrinfo",
            (
                vec![
                    ValType::I32,
                    ValType::I32,
                    ValType::I32,
                    ValType::I32,
                    ValType::I32,
                    ValType::I32,
                    ValType::I32,
                    ValType::I32,
                ],
                vec![ValType::I32],
            ),
            sock_getaddrinfo
        ),
    ]
}

fn to_wasm_return(r: Result<(), Errno>) -> Vec<WasmValue> {
    let code = if let Err(e) = r { e.0 } else { 0 };
    log::trace!("wasi return {code}");
    vec![WasmValue::from_i32(code as i32)]
}

impl async_wasi::snapshots::common::memory::Memory for Memory {
    fn get_data<T: Sized>(&self, offset: WasmPtr<T>) -> Result<&T, Errno> {
        unsafe {
            let r = std::mem::size_of::<T>();
            let ptr = self
                .data_pointer(offset.0 as u32, r as u32)
                .map_err(|_| Errno::__WASI_ERRNO_FAULT)?;
            Ok(ptr.cast::<T>().as_ref().unwrap())
        }
    }

    fn get_slice<T: Sized>(&self, offset: WasmPtr<T>, len: usize) -> Result<&[T], Errno> {
        unsafe {
            let r = std::mem::size_of::<T>() * len;
            let ptr = self
                .data_pointer(offset.0 as u32, r as u32)
                .map_err(|_| Errno::__WASI_ERRNO_FAULT)? as *const T;
            Ok(std::slice::from_raw_parts(ptr, len))
        }
    }

    fn get_iovec<'a>(
        &self,
        iovec_ptr: WasmPtr<__wasi_ciovec_t>,
        iovec_len: __wasi_size_t,
    ) -> Result<Vec<std::io::IoSlice<'a>>, Errno> {
        unsafe {
            let iovec = self.get_slice(iovec_ptr, iovec_len as usize)?.to_vec();
            let mut result = Vec::with_capacity(iovec.len());
            for i in iovec {
                let ptr = self
                    .data_pointer(i.buf, i.buf_len)
                    .map_err(|_| Errno::__WASI_ERRNO_FAULT)?;
                let s = std::io::IoSlice::new(std::slice::from_raw_parts(ptr, i.buf_len as usize));
                result.push(s);
            }
            Ok(result)
        }
    }

    fn mut_data<T: Sized>(&mut self, offset: WasmPtr<T>) -> Result<&mut T, Errno> {
        unsafe {
            let r = std::mem::size_of::<T>();
            let ptr = self
                .data_pointer_mut(offset.0 as u32, r as u32)
                .map_err(|_| Errno::__WASI_ERRNO_FAULT)?;
            Ok(ptr.cast::<T>().as_mut().unwrap())
        }
    }

    fn mut_slice<T: Sized>(&mut self, offset: WasmPtr<T>, len: usize) -> Result<&mut [T], Errno> {
        unsafe {
            let r = std::mem::size_of::<T>() * len;
            let ptr = self
                .data_pointer_mut(offset.0 as u32, r as u32)
                .map_err(|_| Errno::__WASI_ERRNO_FAULT)? as *mut T;
            Ok(std::slice::from_raw_parts_mut(ptr, len))
        }
    }

    fn mut_iovec(
        &mut self,
        iovec_ptr: WasmPtr<async_wasi::snapshots::env::wasi_types::__wasi_iovec_t>,
        iovec_len: async_wasi::snapshots::env::wasi_types::__wasi_size_t,
    ) -> Result<Vec<std::io::IoSliceMut<'_>>, Errno> {
        unsafe {
            let iovec = self.get_slice(iovec_ptr, iovec_len as usize)?.to_vec();
            let mut result = Vec::with_capacity(iovec.len());
            for i in iovec {
                let ptr = self
                    .data_pointer_mut(i.buf, i.buf_len)
                    .map_err(|_| Errno::__WASI_ERRNO_FAULT)?;
                let s = std::io::IoSliceMut::new(std::slice::from_raw_parts_mut(
                    ptr,
                    i.buf_len as usize,
                ));
                result.push(s);
            }
            Ok(result)
        }
    }

    fn write_data<T: Sized>(&mut self, offset: WasmPtr<T>, data: T) -> Result<(), Errno> {
        let p = self.mut_data(offset)?;
        *p = data;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{r#async::fiber::AsyncState, Executor, Loader, Store, Validator};

    #[tokio::test]
    async fn test_async_wasi_module() -> Result<(), Box<dyn std::error::Error>> {
        // create an Executor
        let result = Executor::create(None, None);
        assert!(result.is_ok());
        let mut executor = result.unwrap();
        assert!(!executor.inner.0.is_null());

        // create a Store
        let result = Store::create();
        assert!(result.is_ok());
        let mut store = result.unwrap();

        // create an AsyncWasiModule
        let result = AsyncWasiModule::create(Some(vec!["abc"]), Some(vec![("ENV", "1")]));
        assert!(result.is_ok());
        let mut async_wasi_module = result.unwrap();

        // register async_wasi module into the store
        let result = executor.register_import_module(&mut store, async_wasi_module.as_mut());
        assert!(result.is_ok());

        let wasm_file = std::env::current_dir()
            .unwrap()
            .ancestors()
            .nth(2)
            .unwrap()
            .join("examples/wasmedge-sys/async_hello.wasm");
        let module = Loader::create(None)?.from_file(&wasm_file)?;
        Validator::create(None)?.validate(&module)?;
        let mut instance = executor.register_active_module(&mut store, &module)?;
        let mut fn_start = instance.get_func_mut("_start")?;

        async fn tick() {
            let mut i = 0;
            loop {
                println!("[tick] i={i}");
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                i += 1;
            }
        }
        tokio::spawn(tick());

        let async_state = AsyncState::new();
        let _ = executor
            .call_func_async(&async_state, &mut fn_start, [])
            .await?;

        Ok(())
    }
}
