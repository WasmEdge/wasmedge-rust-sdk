use crate::{
    ffi, instance::module::InnerInstance, r#async::fiber::AsyncCx, CallingFrame, FuncType,
    Function, Instance, WasmEdgeResult, WasmValue,
};
use std::{future::Future, os::raw::c_void};
use wasmedge_types::error::CoreError;

use super::module::AsyncInstance;

pub type AsyncFn<'data, 'inst, 'frame, 'fut, Data>
    // where
    //     'data: 'fut,
    //     'inst: 'fut,
    //     'frame: 'fut,
    = fn(
    &'data mut Data,
    &'inst mut AsyncInstance,
    &'frame mut CallingFrame,
    Vec<WasmValue>,
) -> Box<dyn Future<Output = Result<Vec<WasmValue>, CoreError>> + Send + 'fut>;

unsafe extern "C" fn wrap_async_fn<Data>(
    key_ptr: *mut c_void,
    data: *mut std::os::raw::c_void,
    call_frame_ctx: *const ffi::WasmEdge_CallingFrameContext,
    params: *const ffi::WasmEdge_Value,
    param_len: u32,
    returns: *mut ffi::WasmEdge_Value,
    return_len: u32,
) -> ffi::WasmEdge_Result {
    let mut frame = CallingFrame::create(call_frame_ctx);
    // let executor_ctx = ffi::WasmEdge_CallingFrameGetExecutor(call_frame_ctx);
    let inst_ctx = ffi::WasmEdge_CallingFrameGetModuleInstance(call_frame_ctx);
    let mut inst = std::mem::ManuallyDrop::new(AsyncInstance(Instance {
        inner: InnerInstance(inst_ctx as _),
    }));
    let data = &mut *(data as *mut Data);

    // arguments
    let input = {
        let raw_input = unsafe { std::slice::from_raw_parts(params, param_len as usize) };
        raw_input.iter().map(|r| (*r).into()).collect::<Vec<_>>()
    };

    // returns
    let return_len = return_len as usize;
    let raw_returns = unsafe { std::slice::from_raw_parts_mut(returns, return_len) };

    // get and call host function
    let real_fn: AsyncFn<'_, '_, '_, '_, Data> = std::mem::transmute(key_ptr);

    let async_cx = AsyncCx::new();
    let mut future = std::pin::Pin::from(real_fn(data, &mut inst, &mut frame, input));
    // call host function
    let result = match unsafe { async_cx.block_on(future.as_mut()) } {
        Ok(Ok(ret)) => Ok(ret),
        Ok(Err(err)) => Err(err),
        Err(_err) => Err(CoreError::Common(
            wasmedge_types::error::CoreCommonError::Interrupted,
        )),
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
        Err(err) => err.into(),
    }
}

pub struct AsyncFunction(pub(crate) Function);

impl AsyncFunction {
    /// Creates an async [host function](crate::Function) with the given function type.
    ///
    /// # Arguments
    ///
    /// * `ty` - The types of the arguments and returns of the target function.
    ///
    /// * `real_fn` - The pointer to the target function.
    ///
    /// * `data` - The host context data used in this function.
    ///
    /// * `cost` - The function cost in the [Statistics](crate::Statistics). Pass 0 if the calculation is not needed.
    ///
    /// # Error
    ///
    /// * If fail to create a [Function], then [WasmEdgeError::Func(FuncError::Create)](wasmedge_types::error::FuncError) is returned.
    ///
    pub fn create_async_func<'data, 'inst, 'frame, 'fut, T: Send>(
        ty: &FuncType,
        real_fn: AsyncFn<'data, 'inst, 'frame, 'fut, T>,
        data: *mut T,
        cost: u64,
    ) -> WasmEdgeResult<Self>
    where
        'data: 'fut,
        'inst: 'fut,
        'frame: 'fut,
    {
        let f = unsafe {
            Function::create_with_custom_wrapper(
                ty,
                wrap_async_fn::<T>,
                real_fn as _,
                data as _,
                cost,
            )
        }?;
        Ok(Self(f))
    }
}

impl AsRef<Function> for AsyncFunction {
    fn as_ref(&self) -> &Function {
        &self.0
    }
}

impl AsMut<Function> for AsyncFunction {
    fn as_mut(&mut self) -> &mut Function {
        &mut self.0
    }
}

#[cfg(ignore)]
/// Defines the signature of a host function.
pub(crate) type HostFn<T> = fn(
    CallingFrame,
    Vec<WasmValue>,
    Option<&'static mut T>,
) -> Result<Vec<WasmValue>, HostFuncError>;

#[cfg(ignore)]
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
#[cfg(ignore)]
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
        Err(_err) => Err(HostFuncError::Runtime(0x07)),
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
