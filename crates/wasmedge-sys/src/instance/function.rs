//! Defines WasmEdge Function and FuncType structs.

use crate::{
    ffi::{self},
    CallingFrame, Instance, WasmEdgeResult, WasmValue,
};
use core::ffi::c_void;

use wasmedge_types::{
    error::{CoreError, FuncError, WasmEdgeError},
    ValType,
};

use super::{module::InnerInstance, InnerRef};

pub type SyncFn<Data> = for<'a, 'b, 'c> fn(
    &'a mut Data,
    &'b mut Instance,
    &'c mut CallingFrame,
    Vec<WasmValue>,
) -> Result<Vec<WasmValue>, CoreError>;

pub type CustomFnWrapper = unsafe extern "C" fn(
    key_ptr: *mut c_void,
    data_ptr: *mut c_void,
    calling_frame_ctx: *const ffi::WasmEdge_CallingFrameContext,
    params: *const ffi::WasmEdge_Value,
    param_len: u32,
    returns: *mut ffi::WasmEdge_Value,
    return_len: u32,
) -> ffi::WasmEdge_Result;

// Wrapper function for thread-safe scenarios.
unsafe extern "C" fn wrap_fn<Data>(
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
    let mut inst = std::mem::ManuallyDrop::new(Instance {
        inner: InnerInstance(inst_ctx as _),
    });
    let data = &mut *(data as *mut Data);

    let input = {
        let raw_input = unsafe { std::slice::from_raw_parts(params, param_len as usize) };
        raw_input.iter().map(|r| (*r).into()).collect::<Vec<_>>()
    };

    let return_len = return_len as usize;

    let raw_returns = unsafe { std::slice::from_raw_parts_mut(returns, return_len) };

    let real_fn: SyncFn<Data> = std::mem::transmute(key_ptr);

    match real_fn(data, &mut inst, &mut frame, input) {
        Ok(returns) => {
            assert!(returns.len() == return_len, "[wasmedge-sys] check the number of returns of host function. Expected: {}, actual: {}", return_len, returns.len());
            for (idx, wasm_value) in returns.into_iter().enumerate() {
                raw_returns[idx] = wasm_value.as_raw();
            }
            ffi::WasmEdge_Result { Code: 0 }
        }

        Err(err) => err.into(),
    }
}

/// Defines a host function.
///
/// A WasmEdge [Function] defines a WebAssembly host function described by its [type](crate::FuncType). A host function is a closure of the original function defined in either the host or the WebAssembly module.
#[derive(Debug)]
pub struct Function {
    pub(crate) inner: InnerFunc,
}
impl Function {
    /// Creates a [host function](crate::Function) with the given function type.
    ///
    /// N.B. that this function is used for thread-safe scenarios.
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
    /// # Example
    ///
    /// The example defines a host function `real_add`, and creates a [Function] binding to it by calling
    /// the `create_binding` method.
    ///
    /// ```rust
    /// use wasmedge_macro::sys_host_function;
    /// use wasmedge_sys::{FuncType, Function, WasmValue, CallingFrame};
    /// use wasmedge_types::{error::HostFuncError, ValType, WasmEdgeResult, NeverType};
    ///
    /// #[sys_host_function]
    /// fn real_add(_frame: CallingFrame, inputs: Vec<WasmValue>) -> Result<Vec<WasmValue>, HostFuncError> {
    ///     if inputs.len() != 2 {
    ///         return Err(HostFuncError::User(1));
    ///     }
    ///
    ///     let a = if inputs[0].ty() == ValType::I32 {
    ///         inputs[0].to_i32()
    ///     } else {
    ///         return Err(HostFuncError::User(2));
    ///     };
    ///
    ///     let b = if inputs[1].ty() == ValType::I32 {
    ///         inputs[1].to_i32()
    ///     } else {
    ///         return Err(HostFuncError::User(3));
    ///     };
    ///
    ///     let c = a + b;
    ///
    ///     Ok(vec![WasmValue::from_i32(c)])
    /// }
    ///
    /// // create a FuncType
    /// let func_ty = FuncType::create(vec![ValType::I32; 2], vec![ValType::I32]).expect("fail to create a FuncType");
    ///
    /// // create a Function instance
    /// let func = Function::create_sync_func::<NeverType>(&func_ty, Box::new(real_add), None, 0).expect("fail to create a Function instance");
    /// ```
    pub fn create_sync_func<T>(
        ty: &wasmedge_types::FuncType,
        real_fn: SyncFn<T>,
        data: *mut T,
        cost: u64,
    ) -> WasmEdgeResult<Self> {
        unsafe { Self::create_with_data(ty, real_fn, data, cost) }
    }

    /// Creates a [host function](crate::Function) with the given function type.
    ///
    /// N.B. that this function is used for thread-safe scenarios.
    ///
    /// # Arguments
    ///
    /// * `ty` - The types of the arguments and returns of the target function.
    ///
    /// * `real_fn` - The pointer to the target function.
    ///
    /// * `data` - The pointer to the host context data used in this function.
    ///
    /// * `cost` - The function cost in the [Statistics](crate::Statistics). Pass 0 if the calculation is not needed.
    ///
    /// # Error
    ///
    /// * If fail to create a [Function], then [WasmEdgeError::Func(FuncError::Create)](crate::error::FuncError) is returned.
    ///
    unsafe fn create_with_data<T>(
        ty: &wasmedge_types::FuncType,
        real_fn: SyncFn<T>,
        data: *mut T,
        cost: u64,
    ) -> WasmEdgeResult<Self> {
        Self::create_with_custom_wrapper(ty, wrap_fn::<T>, real_fn as _, data as _, cost)
    }

    /// Creates a [host function](crate::Function) with the given function type and the custom function wrapper.
    ///
    /// # Arguments
    ///
    /// * `ty` - The types of the arguments and returns of the target function.
    ///
    /// * `fn_wrapper` - The custom function wrapper.
    ///
    /// * `real_fn` - The pointer to the target function.
    ///
    /// * `data` - The pointer to the host context data used in this function.
    ///
    /// * `cost` - The function cost in the [Statistics](crate::Statistics). Pass 0 if the calculation is not needed.
    ///
    /// # Error
    ///
    /// * If fail to create a [Function], then [WasmEdgeError::Func(FuncError::Create)](wasmedge_types::error::FuncError) is returned.
    ///
    /// # Safety
    ///
    /// Notice that the caller should guarantee the life cycle of both the `real_fn` and the `data` object.
    ///
    pub unsafe fn create_with_custom_wrapper(
        ty: &wasmedge_types::FuncType,
        fn_wrapper: CustomFnWrapper,
        real_fn: *mut c_void,
        data: *mut c_void,
        cost: u64,
    ) -> WasmEdgeResult<Self> {
        let ty: FuncTypeOwn = ty.into();
        let ctx = ffi::WasmEdge_FunctionInstanceCreateBinding(
            ty.inner.0,
            Some(fn_wrapper),
            real_fn,
            data,
            cost,
        );

        if ctx.is_null() {
            Err(Box::new(WasmEdgeError::Func(FuncError::Create)))
        } else {
            Ok(Self {
                inner: InnerFunc(ctx),
            })
        }
    }

    pub unsafe fn as_ptr(&self) -> *mut ffi::WasmEdge_FunctionInstanceContext {
        self.inner.0
    }

    pub unsafe fn from_raw(ctx: *mut ffi::WasmEdge_FunctionInstanceContext) -> Self {
        Self {
            inner: InnerFunc(ctx),
        }
    }
}
impl Drop for Function {
    fn drop(&mut self) {
        unsafe { ffi::WasmEdge_FunctionInstanceDelete(self.inner.0) };
    }
}

/// Defines a reference to a [host function](crate::Function).
pub type FuncRef<Ref> = InnerRef<Function, Ref>;

pub trait AsFunc {
    unsafe fn get_func_raw(&self) -> *mut ffi::WasmEdge_FunctionInstanceContext;

    fn ty(&self) -> Option<wasmedge_types::FuncType>
    where
        Self: Sized,
    {
        let ty = unsafe { ffi::WasmEdge_FunctionInstanceGetFunctionType(self.get_func_raw()) };
        if ty.is_null() {
            None
        } else {
            let value = std::mem::ManuallyDrop::new(FuncTypeOwn {
                inner: InnerFuncType(ty),
            });
            Some((&*value).into())
        }
    }
}

impl AsFunc for Function {
    unsafe fn get_func_raw(&self) -> *mut ffi::WasmEdge_FunctionInstanceContext {
        self.inner.0
    }
}
impl<F: AsRef<Function>> AsFunc for F {
    unsafe fn get_func_raw(&self) -> *mut ffi::WasmEdge_FunctionInstanceContext {
        self.as_ref().get_func_raw()
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct InnerFunc(pub(crate) *mut ffi::WasmEdge_FunctionInstanceContext);
unsafe impl Send for InnerFunc {}
unsafe impl Sync for InnerFunc {}

#[derive(Debug)]
pub(crate) struct FuncTypeOwn {
    pub(crate) inner: InnerFuncType,
}
impl FuncTypeOwn {
    /// Create a new [FuncType] to be associated with the given arguments and returns.
    ///
    /// # Arguments
    ///
    /// * `args` - The argument types of a [Function].
    ///
    /// * `returns` - The types of the returns of a [Function].
    ///
    /// # Error
    ///
    /// If fail to create a [FuncType], then an error is returned.
    ///
    /// # Example
    ///
    /// ```rust
    /// use wasmedge_sys::FuncType;
    /// use wasmedge_types::ValType;
    ///
    /// let func_ty = FuncType::create(vec![ValType::I32;2], vec![ValType::I32]).expect("fail to create a FuncType");
    /// ```
    pub(crate) fn create<I: IntoIterator<Item = ValType>, R: IntoIterator<Item = ValType>>(
        args: I,
        returns: R,
    ) -> WasmEdgeResult<Self> {
        let param_tys = args
            .into_iter()
            .map(|x| x.into())
            .collect::<Vec<ffi::WasmEdge_ValType>>();
        let ret_tys = returns
            .into_iter()
            .map(|x| x.into())
            .collect::<Vec<ffi::WasmEdge_ValType>>();

        let ctx = unsafe {
            ffi::WasmEdge_FunctionTypeCreate(
                param_tys.as_ptr() as *const _,
                param_tys.len() as u32,
                ret_tys.as_ptr() as *const _,
                ret_tys.len() as u32,
            )
        };
        match ctx.is_null() {
            true => Err(Box::new(WasmEdgeError::FuncTypeCreate)),
            false => Ok(Self {
                inner: InnerFuncType(ctx),
            }),
        }
    }
}
impl Drop for FuncTypeOwn {
    fn drop(&mut self) {
        unsafe { ffi::WasmEdge_FunctionTypeDelete(self.inner.0 as _) }
    }
}

impl FuncTypeOwn {
    /// Returns the number of the arguments of a [Function].
    pub(crate) fn params_len(&self) -> u32 {
        unsafe { ffi::WasmEdge_FunctionTypeGetParametersLength(self.inner.0) }
    }

    /// Returns an Iterator of the arguments of a [Function].
    pub(crate) fn params_type_iter(&self) -> impl Iterator<Item = ValType> {
        let len = self.params_len();
        let mut types = Vec::with_capacity(len as usize);
        unsafe {
            ffi::WasmEdge_FunctionTypeGetParameters(self.inner.0, types.as_mut_ptr(), len);
            types.set_len(len as usize);
        }

        types.into_iter().map(Into::into)
    }

    ///Returns the number of the returns of a [Function].
    pub(crate) fn returns_len(&self) -> u32 {
        unsafe { ffi::WasmEdge_FunctionTypeGetReturnsLength(self.inner.0) }
    }

    /// Returns an Iterator of the return types of a [Function].
    pub(crate) fn returns_type_iter(&self) -> impl Iterator<Item = ValType> {
        let len = self.returns_len();
        let mut types = Vec::with_capacity(len as usize);
        unsafe {
            ffi::WasmEdge_FunctionTypeGetReturns(self.inner.0, types.as_mut_ptr(), len);
            types.set_len(len as usize);
        }

        types.into_iter().map(Into::into)
    }
}

impl From<&wasmedge_types::FuncType> for FuncTypeOwn {
    fn from(ty: &wasmedge_types::FuncType) -> Self {
        FuncTypeOwn::create(ty.args().to_vec(), ty.returns().to_vec()).expect("[wasmedge-sys] Failed to convert wasmedge_types::FuncType into wasmedge_sys::FuncType.")
    }
}

impl From<wasmedge_types::FuncType> for FuncTypeOwn {
    fn from(ty: wasmedge_types::FuncType) -> Self {
        (&ty).into()
    }
}

impl From<&FuncTypeOwn> for wasmedge_types::FuncType {
    fn from(ty: &FuncTypeOwn) -> Self {
        let args = ty.params_type_iter().collect();
        let returns = ty.returns_type_iter().collect();

        wasmedge_types::FuncType::new(args, returns)
    }
}

#[derive(Debug)]
pub(crate) struct InnerFuncType(pub(crate) *const ffi::WasmEdge_FunctionTypeContext);
unsafe impl Send for InnerFuncType {}
unsafe impl Sync for InnerFuncType {}

// #[cfg(test)]
#[cfg(ignore)]
mod tests {
    use super::*;
    #[cfg(all(feature = "async", target_os = "linux"))]
    use crate::{r#async::AsyncWasiModule, WasiInstance, ASYNC_HOST_FUNCS};
    use crate::{types::WasmValue, AsImport, Executor, ImportModule, Store, HOST_FUNC_FOOTPRINTS};
    use std::{
        sync::{Arc, Mutex},
        thread,
    };
    use wasmedge_macro::sys_host_function;
    use wasmedge_types::{NeverType, ValType};

    #[test]
    fn test_func_type() {
        // test FuncType with args and returns
        {
            let param_tys = vec![
                ValType::I32,
                ValType::I64,
                ValType::F32,
                ValType::F64,
                ValType::V128,
                ValType::ExternRef,
            ];
            let param_len = param_tys.len();
            let ret_tys = vec![ValType::FuncRef, ValType::ExternRef, ValType::V128];
            let ret_len = ret_tys.len();

            // create FuncType
            let result = FuncTypeOwn::create(param_tys, ret_tys);
            assert!(result.is_ok());
            let func_ty = result.unwrap().get_ref();

            // check parameters
            assert_eq!(func_ty.params_len(), param_len as u32);
            let param_tys = func_ty.params_type_iter().collect::<Vec<_>>();
            assert_eq!(
                param_tys,
                vec![
                    ValType::I32,
                    ValType::I64,
                    ValType::F32,
                    ValType::F64,
                    ValType::V128,
                    ValType::ExternRef,
                ]
            );

            // check returns
            assert_eq!(func_ty.returns_len(), ret_len as u32);
            let return_tys = func_ty.returns_type_iter().collect::<Vec<_>>();
            assert_eq!(
                return_tys,
                vec![ValType::FuncRef, ValType::ExternRef, ValType::V128]
            );
        }

        // test FuncType without args and returns
        {
            // create FuncType
            let result = FuncTypeOwn::create([], []);
            assert!(result.is_ok());
            let func_ty = result.unwrap().get_ref();

            assert_eq!(func_ty.params_len(), 0);
            assert_eq!(func_ty.returns_len(), 0);
        }
    }

    #[test]
    fn test_func_basic() {
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
            _frame: CallingFrame,
            input: Vec<WasmValue>,
            data: *mut std::ffi::c_void,
        ) -> Result<Vec<WasmValue>, HostFuncError> {
            println!("Rust: Entering Rust function real_add");

            let host_data = unsafe { Box::from_raw(data as *mut T) };
            println!("host_data: {:?}", host_data);

            if input.len() != 2 {
                return Err(HostFuncError::User(1));
            }

            let a = if input[0].ty() == ValType::I32 {
                input[0].to_i32()
            } else {
                return Err(HostFuncError::User(2));
            };

            let b = if input[1].ty() == ValType::I32 {
                input[1].to_i32()
            } else {
                return Err(HostFuncError::User(3));
            };

            let c = a + b;
            println!("Rust: calcuating in real_add c: {c:?}");

            println!("Rust: Leaving Rust function real_add");
            Ok(vec![WasmValue::from_i32(c)])
        }

        assert_eq!(HOST_FUNCS.read().len(), 0);
        assert_eq!(HOST_FUNC_FOOTPRINTS.lock().len(), 0);

        // create a FuncType
        let result = FuncTypeOwn::create(vec![ValType::I32; 2], vec![ValType::I32]);
        assert!(result.is_ok());
        let func_ty = result.unwrap();
        // create a host function
        let result = Function::create_sync_func(
            &func_ty,
            Box::new(real_add::<Data<i32, &str>>),
            Some(Box::new(data)),
            0,
        );
        assert!(result.is_ok());
        let host_func = result.unwrap();

        // get func type
        let result = FuncType::get_from_func(&host_func);
        assert!(result.is_some());
        let ty = result.unwrap();

        // check parameters
        assert_eq!(ty.params_len(), 2);
        let param_tys = ty.params_type_iter().collect::<Vec<_>>();
        assert_eq!(param_tys, vec![ValType::I32; 2]);

        // check returns
        assert_eq!(ty.returns_len(), 1);
        let return_tys = ty.returns_type_iter().collect::<Vec<_>>();
        assert_eq!(return_tys, vec![ValType::I32]);

        // run this function
        let result = Executor::create(None, None);
        assert!(result.is_ok());
        let mut executor = result.unwrap();
        let result = host_func.call(
            &mut executor,
            vec![WasmValue::from_i32(1), WasmValue::from_i32(2)],
        );
        assert!(result.is_ok());
        let returns = result.unwrap();
        assert_eq!(returns[0].to_i32(), 3);
    }

    #[test]
    #[allow(clippy::assertions_on_result_states)]
    fn test_func_create_host_func_in_host_func() {
        #[sys_host_function]
        fn func(
            _frame: CallingFrame,
            _input: Vec<WasmValue>,
        ) -> Result<Vec<WasmValue>, HostFuncError> {
            println!("Entering host function: func");

            // spawn a new thread to create a new host function
            let handler = std::thread::spawn(|| {
                #[sys_host_function]
                fn real_add(
                    _frame: CallingFrame,
                    input: Vec<WasmValue>,
                ) -> Result<Vec<WasmValue>, HostFuncError> {
                    println!("Rust: Entering Rust function real_add");

                    if input.len() != 2 {
                        return Err(HostFuncError::User(1));
                    }

                    let a = if input[0].ty() == ValType::I32 {
                        input[0].to_i32()
                    } else {
                        return Err(HostFuncError::User(2));
                    };

                    let b = if input[1].ty() == ValType::I32 {
                        input[1].to_i32()
                    } else {
                        return Err(HostFuncError::User(3));
                    };

                    let c = a + b;

                    println!("Rust: Leaving Rust function real_add");
                    Ok(vec![WasmValue::from_i32(c)])
                }

                // create a FuncType
                let result = FuncType::create(vec![ValType::I32; 2], vec![ValType::I32]);
                assert!(result.is_ok());
                let func_ty = result.unwrap();
                // create a host function
                let result =
                    Function::create_sync_func::<NeverType>(&func_ty, Box::new(real_add), None, 0);
                assert!(result.is_ok());
                let host_func = result.unwrap();

                // run this function
                let result = Executor::create(None, None);
                assert!(result.is_ok());
                let mut executor = result.unwrap();
                let result = host_func.call(
                    &mut executor,
                    vec![WasmValue::from_i32(1), WasmValue::from_i32(2)],
                );
                assert!(result.is_ok());
                let returns = result.unwrap();
                assert_eq!(returns[0].to_i32(), 3);
            });
            handler.join().unwrap();

            println!("Leaving host function: func");
            Ok(vec![])
        }

        // create a FuncType
        let result = FuncType::create(vec![], vec![]);
        assert!(result.is_ok());
        let func_ty = result.unwrap();
        // create a host function
        let result = Function::create_sync_func::<NeverType>(&func_ty, Box::new(func), None, 0);
        assert!(result.is_ok());
        let host_func = result.unwrap();

        // run this function
        let result = Executor::create(None, None);
        assert!(result.is_ok());
        let mut executor = result.unwrap();
        let result = host_func.call(&mut executor, vec![]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_func_send() {
        // create a FuncType
        let result = FuncType::create(vec![ValType::I32; 2], vec![ValType::I32]);
        assert!(result.is_ok());
        let func_ty = result.unwrap();
        // create a host function
        let result = Function::create_sync_func::<NeverType>(&func_ty, Box::new(real_add), None, 0);
        assert!(result.is_ok());
        let host_func = result.unwrap();

        let handle = thread::spawn(move || {
            // get func type
            let result = host_func.ty();
            assert!(result.is_ok());
            let ty = result.unwrap();

            // check parameters
            assert_eq!(ty.params_len(), 2);
            let param_tys = ty.params_type_iter().collect::<Vec<_>>();
            assert_eq!(param_tys, vec![ValType::I32; 2]);

            // check returns
            assert_eq!(ty.returns_len(), 1);
            let return_tys = ty.returns_type_iter().collect::<Vec<_>>();
            assert_eq!(return_tys, vec![ValType::I32]);
        });

        handle.join().unwrap()
    }

    #[test]
    fn test_func_sync() {
        // create a FuncType
        let result = FuncType::create(vec![ValType::I32; 2], vec![ValType::I32]);
        assert!(result.is_ok());
        let func_ty = result.unwrap();
        // create a host function
        let result = Function::create_sync_func::<NeverType>(&func_ty, Box::new(real_add), None, 0);
        assert!(result.is_ok());
        let host_func = Arc::new(Mutex::new(result.unwrap()));

        let host_func_cloned = Arc::clone(&host_func);
        let handle = thread::spawn(move || {
            let result = host_func_cloned.lock();
            assert!(result.is_ok());
            let host_func = result.unwrap();

            // get func type
            let result = host_func.ty();
            assert!(result.is_ok());
            let ty = result.unwrap();

            // check parameters
            assert_eq!(ty.params_len(), 2);
            let param_tys = ty.params_type_iter().collect::<Vec<_>>();
            assert_eq!(param_tys, vec![ValType::I32; 2]);

            // check returns
            assert_eq!(ty.returns_len(), 1);
            let return_tys = ty.returns_type_iter().collect::<Vec<_>>();
            assert_eq!(return_tys, vec![ValType::I32]);
        });

        handle.join().unwrap();
    }

    #[sys_host_function]
    fn real_add(
        _frame: CallingFrame,
        input: Vec<WasmValue>,
    ) -> Result<Vec<WasmValue>, HostFuncError> {
        println!("Rust: Entering Rust function real_add");

        if input.len() != 2 {
            return Err(HostFuncError::User(1));
        }

        let a = if input[0].ty() == ValType::I32 {
            input[0].to_i32()
        } else {
            return Err(HostFuncError::User(2));
        };

        let b = if input[1].ty() == ValType::I32 {
            input[1].to_i32()
        } else {
            return Err(HostFuncError::User(3));
        };

        let c = a + b;
        println!("Rust: calcuating in real_add c: {c:?}");

        println!("Rust: Leaving Rust function real_add");
        Ok(vec![WasmValue::from_i32(c)])
    }

    #[test]
    fn test_func_closure() -> Result<(), Box<dyn std::error::Error>> {
        {
            // create a host function
            let real_add = |_: CallingFrame,
                            input: Vec<WasmValue>,
                            _: *mut std::os::raw::c_void|
             -> Result<Vec<WasmValue>, HostFuncError> {
                println!("Rust: Entering Rust function real_add");

                if input.len() != 2 {
                    return Err(HostFuncError::User(1));
                }

                let a = if input[0].ty() == ValType::I32 {
                    input[0].to_i32()
                } else {
                    return Err(HostFuncError::User(2));
                };

                let b = if input[1].ty() == ValType::I32 {
                    input[1].to_i32()
                } else {
                    return Err(HostFuncError::User(3));
                };

                let c = a + b;
                println!("Rust: calcuating in real_add c: {c:?}");

                println!("Rust: Leaving Rust function real_add");
                Ok(vec![WasmValue::from_i32(c)])
            };

            // create a FuncType
            let result = FuncType::create(vec![ValType::I32; 2], vec![ValType::I32]);
            assert!(result.is_ok());
            let func_ty = result.unwrap();

            // create a host function from the closure defined above
            let result =
                Function::create_sync_func::<NeverType>(&func_ty, Box::new(real_add), None, 0);
            assert!(result.is_ok());
            let host_func = result.unwrap();

            // create a Store
            let result = Store::create();
            assert!(result.is_ok());
            let mut store = result.unwrap();

            // create an ImportModule
            let mut import = ImportModule::<NeverType>::create("extern", None)?;
            import.add_func("add", host_func);

            // run this function
            let result = Executor::create(None, None);
            assert!(result.is_ok());
            let mut executor = result.unwrap();
            executor.register_import_module(&mut store, &import)?;

            let extern_instance = store.module("extern")?;
            let add = extern_instance.get_func("add")?;

            let result =
                executor.call_func(&add, vec![WasmValue::from_i32(1), WasmValue::from_i32(2)]);
            assert!(result.is_ok());
            let returns = result.unwrap();
            assert_eq!(returns[0].to_i32(), 3);
        }

        assert_eq!(HOST_FUNCS.read().len(), 0);
        assert_eq!(HOST_FUNC_FOOTPRINTS.lock().len(), 0);

        Ok(())
    }

    #[test]
    fn test_func_drop() -> Result<(), Box<dyn std::error::Error>> {
        // create a host function
        let real_add = |_: CallingFrame,
                        input: Vec<WasmValue>,
                        _: *mut std::os::raw::c_void|
         -> Result<Vec<WasmValue>, HostFuncError> {
            println!("Rust: Entering Rust function real_add");

            if input.len() != 2 {
                return Err(HostFuncError::User(1));
            }

            let a = if input[0].ty() == ValType::I32 {
                input[0].to_i32()
            } else {
                return Err(HostFuncError::User(2));
            };

            let b = if input[1].ty() == ValType::I32 {
                input[1].to_i32()
            } else {
                return Err(HostFuncError::User(3));
            };

            let c = a + b;
            println!("Rust: calcuating in real_add c: {c:?}");

            println!("Rust: Leaving Rust function real_add");
            Ok(vec![WasmValue::from_i32(c)])
        };

        // create a FuncType
        let result = FuncType::create(vec![ValType::I32; 2], vec![ValType::I32]);
        assert!(result.is_ok());
        let func_ty = result.unwrap();

        // create a host function
        let result = Function::create_sync_func::<NeverType>(&func_ty, Box::new(real_add), None, 0);
        assert!(result.is_ok());
        let host_func = result.unwrap();

        assert_eq!(Arc::strong_count(&host_func.inner), 1);
        assert!(!host_func.registered);

        assert_eq!(HOST_FUNCS.read().len(), 1);
        assert_eq!(HOST_FUNC_FOOTPRINTS.lock().len(), 1);

        // clone the host function before adding it to the import object
        let host_func_cloned = host_func.clone();

        assert_eq!(Arc::strong_count(&host_func_cloned.inner), 2);
        assert!(!host_func_cloned.registered);

        assert_eq!(HOST_FUNCS.read().len(), 1);
        assert_eq!(HOST_FUNC_FOOTPRINTS.lock().len(), 1);

        // create an ImportModule
        let mut import = ImportModule::<NeverType>::create("extern", None)?;
        // add the host function to the import module
        import.add_func("add", host_func);

        assert_eq!(Arc::strong_count(&host_func_cloned.inner), 2);
        assert!(!host_func_cloned.registered);

        assert_eq!(HOST_FUNCS.read().len(), 1);
        assert_eq!(HOST_FUNC_FOOTPRINTS.lock().len(), 1);

        drop(host_func_cloned);

        assert_eq!(HOST_FUNCS.read().len(), 1);
        assert_eq!(HOST_FUNC_FOOTPRINTS.lock().len(), 1);

        // create a Store
        let result = Store::create();
        assert!(result.is_ok());
        let mut store = result.unwrap();

        // run this function
        let result = Executor::create(None, None);
        assert!(result.is_ok());
        let mut executor = result.unwrap();
        executor.register_import_module(&mut store, &import)?;

        // get the registered host function
        let extern_instance = store.module("extern")?;
        let add = extern_instance.get_func("add")?;
        assert_eq!(Arc::strong_count(&add.inner), 1);
        assert!(add.registered);

        assert_eq!(HOST_FUNCS.read().len(), 1);
        assert_eq!(HOST_FUNC_FOOTPRINTS.lock().len(), 1);

        // clone the host function
        let add_cloned = add.clone();
        assert_eq!(Arc::strong_count(&add.inner), 2);
        assert!(add.registered);
        assert_eq!(Arc::strong_count(&add_cloned.inner), 2);
        assert!(add_cloned.registered);

        assert_eq!(HOST_FUNCS.read().len(), 1);
        assert_eq!(HOST_FUNC_FOOTPRINTS.lock().len(), 1);

        // drop the cloned host function
        drop(add_cloned);
        assert_eq!(Arc::strong_count(&add.inner), 1);
        assert!(add.registered);

        assert_eq!(HOST_FUNCS.read().len(), 1);
        assert_eq!(HOST_FUNC_FOOTPRINTS.lock().len(), 1);

        drop(add);

        assert_eq!(HOST_FUNCS.read().len(), 1);
        assert_eq!(HOST_FUNC_FOOTPRINTS.lock().len(), 1);

        // get the registered host function again
        let extern_instance = store.module("extern")?;
        let add_again = extern_instance.get_func("add")?;
        assert_eq!(Arc::strong_count(&add_again.inner), 1);
        assert!(add_again.registered);

        assert_eq!(HOST_FUNCS.read().len(), 1);
        assert_eq!(HOST_FUNC_FOOTPRINTS.lock().len(), 1);

        // ! notice that `add_again` should be dropped before or not be used after dropping `import`
        drop(add_again);

        assert_eq!(HOST_FUNCS.read().len(), 1);
        assert_eq!(HOST_FUNC_FOOTPRINTS.lock().len(), 1);

        // drop the import object
        drop(import);

        assert!(store.module("extern").is_err());

        assert_eq!(HOST_FUNCS.read().len(), 0);
        assert_eq!(HOST_FUNC_FOOTPRINTS.lock().len(), 0);

        // ! if `add_again` is not dropped before dropping `import`, then calling `add_again` will crash
        // let result = executor.call_func(
        //     &add_again,
        //     vec![WasmValue::from_i32(1), WasmValue::from_i32(2)],
        // );

        Ok(())
    }

    #[cfg(all(feature = "async", target_os = "linux"))]
    #[tokio::test]
    async fn test_func_async_closure() -> Result<(), Box<dyn std::error::Error>> {
        {
            #[derive(Debug)]
            struct Data<T, S> {
                _x: i32,
                _y: String,
                _v: Vec<T>,
                _s: Vec<S>,
            }
            impl<T, S> Drop for Data<T, S> {
                fn drop(&mut self) {
                    println!("Dropping Data");
                }
            }

            let data: Data<i32, &str> = Data {
                _x: 12,
                _y: "hello".to_string(),
                _v: vec![1, 2, 3],
                _s: vec!["macos", "linux", "windows"],
            };

            // define an async closure
            let c = |_frame: CallingFrame,
                     _args: Vec<WasmValue>,
                     data: *mut std::os::raw::c_void|
             -> Box<
                (dyn std::future::Future<Output = Result<Vec<WasmValue>, HostFuncError>> + Send),
            > {
                // let host_data = unsafe { &mut *(data as *mut Data<i32, &str>) };
                let host_data = unsafe { Box::from_raw(data as *mut Data<i32, &str>) };

                Box::new(async move {
                    for _ in 0..10 {
                        println!("[async hello] say hello");
                        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

                        println!("host_data: {:?}", host_data);
                    }

                    println!("[async hello] Done!");

                    Ok(vec![])
                })
            };

            // create a FuncType
            let result = FuncType::create(vec![], vec![]);
            assert!(result.is_ok());
            let func_ty = result.unwrap();

            // create an async host function
            let result =
                Function::create_async_func(&func_ty, Box::new(c), Some(Box::new(data)), 0);
            assert!(result.is_ok());
            let async_hello_func = result.unwrap();

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
            let result = AsyncWasiModule::create(Some(vec!["abc"]), Some(vec![("a", "1")]), None);
            assert!(result.is_ok());
            let async_wasi_module = result.unwrap();

            // register async_wasi module into the store
            let wasi_import = WasiInstance::AsyncWasi(async_wasi_module);
            let result = executor.register_wasi_instance(&mut store, &wasi_import);
            assert!(result.is_ok());

            // create an ImportModule
            let mut import = ImportModule::<NeverType>::create("extern", None)?;
            import.add_func("async_hello", async_hello_func);

            executor.register_import_module(&mut store, &import)?;

            let extern_instance = store.module("extern")?;
            let async_hello = extern_instance.get_func("async_hello")?;

            async fn tick() {
                let mut i = 0;
                loop {
                    println!("[tick] i={i}");
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                    i += 1;
                }
            }
            tokio::spawn(tick());

            let async_state = crate::r#async::fiber::AsyncState::new();
            let _ = executor
                .call_func_async(&async_state, &async_hello, [])
                .await?;
        }

        assert_eq!(ASYNC_HOST_FUNCS.read().len(), 0);
        assert_eq!(HOST_FUNC_FOOTPRINTS.lock().len(), 0);

        Ok(())
    }

    #[cfg(all(feature = "async", target_os = "linux"))]
    #[tokio::test]
    async fn test_func_async_func() -> Result<(), Box<dyn std::error::Error>> {
        {
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

            // define async host function
            fn f<T: core::fmt::Debug + Send + Sync + 'static>(
                _frame: CallingFrame,
                _args: Vec<WasmValue>,
                data: *mut std::ffi::c_void,
            ) -> Box<(dyn std::future::Future<Output = Result<Vec<WasmValue>, HostFuncError>> + Send)>
            {
                let data = unsafe { Box::from_raw(data as *mut T) };

                Box::new(async move {
                    for _ in 0..10 {
                        println!("[async hello] say hello");
                        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                        println!("host_data: {:?}", data);
                    }

                    println!("[async hello] Done!");

                    Ok(vec![])
                })
            }

            // create a FuncType
            let result = FuncType::create(vec![], vec![]);
            assert!(result.is_ok());
            let func_ty = result.unwrap();

            // create an async host function
            let result = Function::create_async_func(
                &func_ty,
                Box::new(f::<Data<i32, &str>>),
                Some(Box::new(data)),
                0,
            );
            assert!(result.is_ok());
            let async_hello_func = result.unwrap();

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
            let result = AsyncWasiModule::create(Some(vec!["abc"]), Some(vec![("a", "1")]), None);
            assert!(result.is_ok());
            let async_wasi_module = result.unwrap();

            // register async_wasi module into the store
            let wasi_import = WasiInstance::AsyncWasi(async_wasi_module);
            let result = executor.register_wasi_instance(&mut store, &wasi_import);
            assert!(result.is_ok());

            // create an ImportModule
            let mut import = ImportModule::<NeverType>::create("extern", None)?;
            import.add_func("async_hello", async_hello_func);

            executor.register_import_module(&mut store, &import)?;

            let extern_instance = store.module("extern")?;
            let async_hello = extern_instance.get_func("async_hello")?;

            async fn tick() {
                let mut i = 0;
                loop {
                    println!("[tick] i={i}");
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                    i += 1;
                }
            }
            tokio::spawn(tick());

            let async_state = crate::r#async::fiber::AsyncState::new();
            let _ = executor
                .call_func_async(&async_state, &async_hello, [])
                .await?;
        }

        assert_eq!(ASYNC_HOST_FUNCS.read().len(), 0);
        assert_eq!(HOST_FUNC_FOOTPRINTS.lock().len(), 0);

        Ok(())
    }
}
