use crate::{
    ffi, instance::function::InnerFunc, r#async::fiber::AsyncCx, CallingFrame, FuncType,
    WasmEdgeResult, WasmValue,
};
use parking_lot::Mutex;
use std::{pin::Pin, sync::Arc};
use wasmedge_types::{
    error::{FuncError, HostFuncError, WasmEdgeError},
    NeverType,
};

/// Defines a host function.
///
/// A WasmEdge [Function] defines a WebAssembly host function described by its [type](crate::FuncType). A host function is a closure of the original function defined in either the host or the WebAssembly module.
#[derive(Debug)]
pub(crate) struct WasiFunction {
    pub(crate) inner: Arc<Mutex<InnerFunc>>,
    pub(crate) registered: bool,
}
impl WasiFunction {
    pub(crate) fn create_wasi_func<T>(
        ty: &FuncType,
        real_fn: HostFn<T>,
        ctx_data: Option<&mut T>,
        cost: u64,
    ) -> WasmEdgeResult<Self> {
        let data = match ctx_data {
            Some(d) => d as *mut T as *mut std::os::raw::c_void,
            None => std::ptr::null_mut(),
        };

        let ctx = unsafe {
            ffi::WasmEdge_FunctionInstanceCreateBinding(
                ty.inner.0,
                Some(wrap_sync_wasi_fn::<T>),
                real_fn as *mut _,
                data,
                cost,
            )
        };

        match ctx.is_null() {
            true => Err(Box::new(WasmEdgeError::Func(FuncError::Create))),
            false => Ok(Self {
                inner: Arc::new(Mutex::new(InnerFunc(ctx))),
                registered: false,
            }),
        }
    }

    pub(crate) fn create_async_wasi_func<T>(
        ty: &FuncType,
        real_fn: AsyncHostFn<T>,
        ctx_data: Option<&mut T>,
        cost: u64,
    ) -> WasmEdgeResult<Self> {
        let data = match ctx_data {
            Some(d) => d as *mut T as *mut std::os::raw::c_void,
            None => std::ptr::null_mut(),
        };

        let ctx = unsafe {
            ffi::WasmEdge_FunctionInstanceCreateBinding(
                ty.inner.0,
                Some(wrap_async_wasi_fn::<T>),
                real_fn as *mut _,
                data,
                cost,
            )
        };

        match ctx.is_null() {
            true => Err(Box::new(WasmEdgeError::Func(FuncError::Create))),
            false => Ok(Self {
                inner: Arc::new(Mutex::new(InnerFunc(ctx))),
                registered: false,
            }),
        }
    }
}
impl Clone for WasiFunction {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            registered: self.registered,
        }
    }
}

/// Defines the signature of an asynchronous host function.
pub(crate) type AsyncHostFn<T> =
    fn(
        CallingFrame,
        Vec<WasmValue>,
        Option<&'static mut T>,
    ) -> Box<dyn std::future::Future<Output = Result<Vec<WasmValue>, HostFuncError>> + Send>;

/// Defines the signature of a host function.
pub(crate) type HostFn<T> = fn(
    CallingFrame,
    Vec<WasmValue>,
    Option<&'static mut T>,
) -> Result<Vec<WasmValue>, HostFuncError>;

extern "C" fn wrap_sync_wasi_fn<T: 'static>(
    key_ptr: *mut std::ffi::c_void,
    data: *mut std::ffi::c_void,
    call_frame_ctx: *const ffi::WasmEdge_CallingFrameContext,
    params: *const ffi::WasmEdge_Value,
    param_len: u32,
    returns: *mut ffi::WasmEdge_Value,
    return_len: u32,
) -> ffi::WasmEdge_Result {
    let frame = CallingFrame::create(call_frame_ctx);

    // recover the async host function
    let real_func: HostFn<T> = unsafe { std::mem::transmute(key_ptr) };

    // recover the context data
    let data = if std::any::TypeId::of::<T>() == std::any::TypeId::of::<NeverType>() {
        None
    } else {
        let data: &'static mut T = unsafe { &mut *(data as *mut T) };
        Some(data)
    };

    // input arguments
    let input = {
        let raw_input = unsafe {
            std::slice::from_raw_parts(
                params,
                param_len
                    .try_into()
                    .expect("len of params should not greater than usize"),
            )
        };
        raw_input.iter().map(|r| (*r).into()).collect::<Vec<_>>()
    };

    // returns
    let return_len = return_len
        .try_into()
        .expect("len of returns should not greater than usize");
    let raw_returns = unsafe { std::slice::from_raw_parts_mut(returns, return_len) };

    match real_func(frame, input, data) {
        Ok(returns) => {
            assert!(returns.len() == return_len, "[wasmedge-sys] check the number of returns of host function. Expected: {}, actual: {}", return_len, returns.len());
            for (idx, wasm_value) in returns.into_iter().enumerate() {
                raw_returns[idx] = wasm_value.as_raw();
            }
            ffi::WasmEdge_Result { Code: 0 }
        }
        Err(err) => match err {
            HostFuncError::User(code) => unsafe {
                ffi::WasmEdge_ResultGen(ffi::WasmEdge_ErrCategory_UserLevelError, code)
            },
            HostFuncError::Runtime(code) => unsafe {
                ffi::WasmEdge_ResultGen(ffi::WasmEdge_ErrCategory_WASM, code)
            },
        },
    }
}

extern "C" fn wrap_async_wasi_fn<T: 'static>(
    key_ptr: *mut std::ffi::c_void,
    data: *mut std::ffi::c_void,
    call_frame_ctx: *const ffi::WasmEdge_CallingFrameContext,
    params: *const ffi::WasmEdge_Value,
    param_len: u32,
    returns: *mut ffi::WasmEdge_Value,
    return_len: u32,
) -> ffi::WasmEdge_Result {
    let frame = CallingFrame::create(call_frame_ctx);

    // recover the async host function
    let real_func: AsyncHostFn<T> = unsafe { std::mem::transmute(key_ptr) };

    // recover the context data
    let data = if std::any::TypeId::of::<T>() == std::any::TypeId::of::<NeverType>() {
        None
    } else {
        let data: &'static mut T = unsafe { &mut *(data as *mut T) };
        Some(data)
    };

    // arguments
    let input = {
        let raw_input = unsafe {
            std::slice::from_raw_parts(
                params,
                param_len
                    .try_into()
                    .expect("len of params should not greater than usize"),
            )
        };
        raw_input.iter().map(|r| (*r).into()).collect::<Vec<_>>()
    };

    // returns
    let return_len = return_len
        .try_into()
        .expect("len of returns should not greater than usize");
    let raw_returns = unsafe { std::slice::from_raw_parts_mut(returns, return_len) };

    let async_cx = AsyncCx::new();
    let mut future = Pin::from(real_func(frame, input, data));
    let result = match unsafe { async_cx.block_on(future.as_mut()) } {
        Ok(Ok(ret)) => Ok(ret),
        Ok(Err(err)) => Err(err),
        Err(_err) => Err(HostFuncError::User(0x87)),
    };

    // parse result
    match result {
        Ok(returns) => {
            assert!(returns.len() == return_len, "[wasmedge-sys] check the number of returns of async host function. Expected: {}, actual: {}", return_len, returns.len());
            for (idx, wasm_value) in returns.into_iter().enumerate() {
                raw_returns[idx] = wasm_value.as_raw();
            }
            ffi::WasmEdge_Result { Code: 0 }
        }
        Err(err) => match err {
            HostFuncError::User(code) => unsafe {
                ffi::WasmEdge_ResultGen(ffi::WasmEdge_ErrCategory_UserLevelError, code)
            },
            HostFuncError::Runtime(code) => unsafe {
                ffi::WasmEdge_ResultGen(ffi::WasmEdge_ErrCategory_WASM, code)
            },
        },
    }
}
