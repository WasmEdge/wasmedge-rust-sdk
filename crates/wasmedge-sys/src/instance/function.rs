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

    let input = if params.is_null() || param_len == 0 {
        vec![]
    } else {
        let raw_input = unsafe { std::slice::from_raw_parts(params, param_len as usize) };
        raw_input.iter().map(|r| (*r).into()).collect::<Vec<_>>()
    };

    let return_len = return_len as usize;

    let mut empty_return = [];
    let raw_returns = if returns.is_null() || return_len == 0 {
        &mut empty_return
    } else {
        unsafe { std::slice::from_raw_parts_mut(returns, return_len) }
    };

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
    /// # Safety
    ///
    /// The lifetime of `data` must be greater than that of `Function` itself.
    pub unsafe fn create_sync_func<T>(
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

    /// # Safety
    ///
    /// The lifetime of the returned pointer must not exceed that of the object itself.
    pub unsafe fn as_ptr(&self) -> *mut ffi::WasmEdge_FunctionInstanceContext {
        self.inner.0
    }

    /// # Safety
    ///
    /// This function will take over the lifetime management of `ctx`, so do not call `ffi::WasmEdge_FunctionInstanceDelete` on `ctx` after this.
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
    /// # Safety
    ///
    /// The lifetime of the returned pointer must not exceed that of the object itself.
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{types::WasmValue, AsInstance, Executor, ImportModule};

    use wasmedge_types::{error::CoreExecutionError, FuncType, ValType};

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
            let func_ty = result.unwrap();

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
            let func_ty = result.unwrap();

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
            _host_data: &mut Data<i32, &str>,
            _inst: &mut Instance,
            _frame: &mut CallingFrame,
            input: Vec<WasmValue>,
        ) -> Result<Vec<WasmValue>, CoreError> {
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

            let c = a + b;
            println!("Rust: calcuating in real_add c: {c:?}");

            println!("Rust: Leaving Rust function real_add");
            Ok(vec![WasmValue::from_i32(c)])
        }

        let mut import_module = ImportModule::create("test_module", Box::new(data)).unwrap();

        // create a FuncType
        let func_ty = FuncType::new(vec![ValType::I32; 2], vec![ValType::I32]);
        // create a host function
        let result = unsafe {
            Function::create_sync_func(
                &func_ty,
                real_add::<Data<i32, &str>>,
                import_module.get_host_data_mut(),
                0,
            )
        };
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

        import_module.add_func("add", host_func);

        // run this function
        let result = Executor::create(None, None);
        assert!(result.is_ok());
        let mut executor = result.unwrap();

        let mut add_func = import_module.get_func_mut("add").unwrap();

        let result = executor.call_func(
            &mut add_func,
            vec![WasmValue::from_i32(1), WasmValue::from_i32(2)],
        );

        assert!(result.is_ok());
        let returns = result.unwrap();
        assert_eq!(returns[0].to_i32(), 3);
    }
}
