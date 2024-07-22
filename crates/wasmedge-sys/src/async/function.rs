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
    let input = if params.is_null() || param_len == 0 {
        vec![]
    } else {
        let raw_input = unsafe { std::slice::from_raw_parts(params, param_len as usize) };
        raw_input.iter().map(|r| (*r).into()).collect::<Vec<_>>()
    };

    // returns
    let return_len = return_len as usize;
    let mut empty_return = [];
    let raw_returns = if returns.is_null() || return_len == 0 {
        &mut empty_return
    } else {
        unsafe { std::slice::from_raw_parts_mut(returns, return_len) }
    };

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        instance::function::AsFunc,
        r#async::{fiber::AsyncState, module::AsyncImportObject},
        types::WasmValue,
        AsInstance, Executor,
    };

    use wasmedge_types::{error::CoreExecutionError, FuncType, ValType};

    #[tokio::test]
    async fn test_func_basic() {
        #[derive(Debug)]
        struct Data<T, S> {
            _x: i32,
            _y: String,
            _v: Vec<T>,
            _s: Vec<S>,
        }

        let data: Data<i32, &str> = Data {
            _x: 12,
            _y: "hello".to_string(),
            _v: vec![1, 2, 3],
            _s: vec!["macos", "linux", "windows"],
        };

        fn real_add<T: core::fmt::Debug>(
            _host_data: &mut Data<i32, &str>,
            _inst: &mut AsyncInstance,
            _frame: &mut CallingFrame,
            input: Vec<WasmValue>,
        ) -> Box<dyn Future<Output = Result<Vec<WasmValue>, CoreError>> + Send> {
            Box::new(async move {
                println!("Rust: Entering Rust function real_add");

                if input.len() != 2 {
                    return Err(CoreError::Execution(CoreExecutionError::FuncSigMismatch));
                }

                let a = if input[0].ty() == ValType::I32 {
                    input[0].to_i32()
                } else {
                    return Err(CoreError::Execution(CoreExecutionError::FuncSigMismatch));
                };

                let b = if input[1].ty() == ValType::I32 {
                    input[1].to_i32()
                } else {
                    return Err(CoreError::Execution(CoreExecutionError::FuncSigMismatch));
                };

                tokio::time::sleep(std::time::Duration::from_millis(300)).await;

                let c = a + b;
                println!("Rust: calcuating in real_add c: {c:?}");

                println!("Rust: Leaving Rust function real_add");
                Ok(vec![WasmValue::from_i32(c)])
            })
        }

        let mut import_module = AsyncImportObject::create("test_module", Box::new(data)).unwrap();

        // create a FuncType
        let func_ty = FuncType::new(vec![ValType::I32; 2], vec![ValType::I32]);
        // create a host function
        let result = AsyncFunction::create_async_func(
            &func_ty,
            real_add::<Data<i32, &str>>,
            import_module.get_host_data_mut(),
            0,
        );
        assert!(result.is_ok());
        let host_func = result.unwrap();

        // get func type
        let result = host_func.ty();
        assert!(result.is_some());
        let ty = result.unwrap();

        // check parameters
        assert_eq!(ty.args_len(), 2);
        assert_eq!(ty.args(), &[ValType::I32; 2]);

        // check returns
        assert_eq!(ty.returns_len(), 1);
        assert_eq!(ty.returns(), &[ValType::I32]);

        import_module.add_async_func("add", host_func);

        // run this function
        let result = Executor::create(None, None);
        assert!(result.is_ok());
        let mut executor = result.unwrap();

        let mut add_func = import_module.get_func_mut("add").unwrap();

        let async_state = AsyncState::new();

        let result = executor
            .call_func_async(
                &async_state,
                &mut add_func,
                vec![WasmValue::from_i32(1), WasmValue::from_i32(2)],
            )
            .await;

        assert!(result.is_ok());
        let returns = result.unwrap();
        assert_eq!(returns[0].to_i32(), 3);
    }
}
