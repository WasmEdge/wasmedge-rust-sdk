//! Defines WasmEdge Statistics struct.

use crate::{ffi, WasmEdgeResult};
use std::sync::Arc;
use wasmedge_types::error::WasmEdgeError;

#[derive(Debug, Clone)]
/// Struct of WasmEdge Statistics.
pub struct Statistics {
    pub(crate) inner: Arc<InnerStat>,
}
impl Statistics {
    /// Creates a new [Statistics].
    ///
    /// # Error
    ///
    /// If fail to create a [Statistics], then an error is returned.
    pub fn create() -> WasmEdgeResult<Self> {
        let ctx = unsafe { ffi::WasmEdge_StatisticsCreate() };
        match ctx.is_null() {
            true => Err(Box::new(WasmEdgeError::StatisticsCreate)),
            false => Ok(Statistics {
                inner: Arc::new(InnerStat(ctx)),
            }),
        }
    }

    /// Returns the instruction count in execution.
    pub fn instr_count(&self) -> u64 {
        unsafe { ffi::WasmEdge_StatisticsGetInstrCount(self.inner.0) }
    }

    /// Returns the instruction count per second in execution.
    ///
    /// # Notice
    ///
    /// For the following cases,
    /// * [Statistics] is not enabled, or
    /// * the total execution time is 0
    ///
    /// The instructions per second could be `NaN`, which represents `divided-by-zero`.
    /// Use the `is_nan` function of F64 to check the return value before use it,
    /// for example,
    ///
    /// ```
    /// use wasmedge_sys::Statistics;
    ///
    /// // create a Statistics instance
    /// let stat = Statistics::create().expect("fail to create a Statistics");
    ///
    /// // check instruction count per second
    /// assert!(stat.instr_per_sec().is_nan());
    /// ```
    pub fn instr_per_sec(&self) -> f64 {
        unsafe { ffi::WasmEdge_StatisticsGetInstrPerSecond(self.inner.0) }
    }

    /// Returns the total cost in execution.
    pub fn cost_in_total(&self) -> u64 {
        unsafe { ffi::WasmEdge_StatisticsGetTotalCost(self.inner.0) }
    }

    /// Sets the cost of instructions.
    ///
    /// # Arguments
    ///
    /// * `cost_table` - The slice of cost table.
    pub fn set_cost_table(&mut self, cost_table: impl AsRef<[u64]>) {
        unsafe {
            ffi::WasmEdge_StatisticsSetCostTable(
                self.inner.0,
                cost_table.as_ref().as_ptr() as *mut _,
                cost_table.as_ref().len() as u32,
            )
        }
    }

    /// Sets the cost limit in execution.
    ///
    /// # Arguments
    ///
    /// * `limit` - The cost limit.
    pub fn set_cost_limit(&mut self, limit: u64) {
        unsafe { ffi::WasmEdge_StatisticsSetCostLimit(self.inner.0, limit) }
    }

    /// Clears the data in this statistics.
    pub fn clear(&mut self) {
        unsafe { ffi::WasmEdge_StatisticsClear(self.inner.0) }
    }

    /// Provides a raw pointer to the inner Statistics context.
    #[cfg(feature = "ffi")]
    #[cfg_attr(docsrs, doc(cfg(feature = "ffi")))]
    pub fn as_ptr(&self) -> *const ffi::WasmEdge_StatisticsContext {
        self.inner.0 as *const _
    }
}
impl Drop for Statistics {
    fn drop(&mut self) {
        if Arc::strong_count(&self.inner) == 1 && !self.inner.0.is_null() {
            unsafe { ffi::WasmEdge_StatisticsDelete(self.inner.0) }
        }
    }
}

#[derive(Debug)]
pub(crate) struct InnerStat(pub(crate) *mut ffi::WasmEdge_StatisticsContext);
unsafe impl Send for InnerStat {}
unsafe impl Sync for InnerStat {}

#[cfg(test)]
mod tests {
    use crate::{Config, Engine, Executor, Loader, Statistics, Store, Validator, WasmValue};
    use std::{
        sync::{Arc, Mutex},
        thread,
    };
    use wasmedge_types::error::{
        CoreError, CoreExecutionError, InstanceError, StoreError, WasmEdgeError,
    };

    #[test]
    fn test_stat_send() {
        let result = Statistics::create();
        assert!(result.is_ok());
        let stat = result.unwrap();

        let handle = thread::spawn(move || {
            assert!(!stat.inner.0.is_null());
            println!("{:?}", stat.inner);
        });

        handle.join().unwrap();
    }

    #[test]
    fn test_stat_sync() {
        let result = Statistics::create();
        assert!(result.is_ok());
        let stat = Arc::new(Mutex::new(result.unwrap()));

        let stat_cloned = Arc::clone(&stat);
        let handle = thread::spawn(move || {
            let result = stat_cloned.lock();
            assert!(result.is_ok());
            let stat = result.unwrap();

            assert!(!stat.inner.0.is_null());
        });

        handle.join().unwrap();
    }

    #[allow(unused_assignments)]
    #[test]
    fn test_executor_with_statistics() {
        // create a Config context
        let result = Config::create();
        assert!(result.is_ok());
        let mut config = result.unwrap();
        // enable Statistics
        config.count_instructions(true);
        config.measure_time(true);
        config.measure_cost(true);

        // create a Statistics context
        let result = Statistics::create();
        assert!(result.is_ok());
        let mut stat = result.unwrap();
        // set cost table
        stat.set_cost_table([]);
        let mut cost_table = vec![20u64; 512];
        stat.set_cost_table(&mut cost_table);
        // set cost limit
        stat.set_cost_limit(100_000_000_000_000);

        // create an Executor context
        let result = Executor::create(Some(&config), Some(&mut stat));
        assert!(result.is_ok());
        let mut executor = result.unwrap();

        // create an ImportObj module
        let import = common::create_extern_module("extern");

        // create a Store context
        let result = Store::create();
        assert!(result.is_ok());
        let mut store = result.unwrap();

        // register the import_obj module into the store context
        let result = executor.register_import_module(&mut store, &import);
        assert!(result.is_ok());

        // load module from a wasm file
        let result = Config::create();
        assert!(result.is_ok());
        let config = result.unwrap();
        let result = Loader::create(Some(&config));
        assert!(result.is_ok());
        let loader = result.unwrap();
        let path = std::env::current_dir()
            .unwrap()
            .ancestors()
            .nth(2)
            .unwrap()
            .join("examples/wasmedge-sys/data/test.wat");
        let result = loader.from_file(path);
        assert!(result.is_ok());
        let module = result.unwrap();

        // validate module
        let result = Config::create();
        assert!(result.is_ok());
        let config = result.unwrap();
        let result = Validator::create(Some(&config));
        assert!(result.is_ok());
        let validator = result.unwrap();
        let result = validator.validate(&module);
        assert!(result.is_ok());

        // register a wasm module into the store context
        let result = executor.register_named_module(&mut store, &module, "module");
        assert!(result.is_ok());

        // load module from a wasm file
        let result = Config::create();
        assert!(result.is_ok());
        let config = result.unwrap();
        let result = Loader::create(Some(&config));
        assert!(result.is_ok());
        let loader = result.unwrap();
        let path = std::env::current_dir()
            .unwrap()
            .ancestors()
            .nth(2)
            .unwrap()
            .join("examples/wasmedge-sys/data/test.wat");
        let result = loader.from_file(path);
        assert!(result.is_ok());
        let module = result.unwrap();

        // validate module
        let result = Config::create();
        assert!(result.is_ok());
        let config = result.unwrap();
        let result = Validator::create(Some(&config));
        assert!(result.is_ok());
        let validator = result.unwrap();
        let result = validator.validate(&module);
        assert!(result.is_ok());

        // register a wasm module as an active module
        let result = executor.register_active_module(&mut store, &module);
        assert!(result.is_ok());
        let active_instance = result.unwrap();

        // get the exported functions from the active module
        let result = active_instance.get_func("func-mul-2");
        assert!(result.is_ok());
        let func_mul_2 = result.unwrap();
        let result = executor.run_func(
            &func_mul_2,
            [WasmValue::from_i32(123), WasmValue::from_i32(456)],
        );
        assert!(result.is_ok());
        let returns = result.unwrap();
        let returns = returns.iter().map(|x| x.to_i32()).collect::<Vec<_>>();
        assert_eq!(returns, vec![246, 912]);

        // function type mismatched
        let result = executor.run_func(&func_mul_2, []);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            Box::new(WasmEdgeError::Core(CoreError::Execution(
                CoreExecutionError::FuncTypeMismatch
            )))
        );

        // function type mismatched
        let result = executor.run_func(
            &func_mul_2,
            [WasmValue::from_i64(123), WasmValue::from_i32(456)],
        );
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            Box::new(WasmEdgeError::Core(CoreError::Execution(
                CoreExecutionError::FuncTypeMismatch
            )))
        );

        // try to get non-existent exported function
        let result = active_instance.get_func("func-mul-3");
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            Box::new(WasmEdgeError::Instance(InstanceError::NotFoundFunc(
                "func-mul-3".into()
            )))
        );

        // call host function by using external reference
        let result = active_instance.get_table("tab-ext");
        assert!(result.is_ok());
        let mut table = result.unwrap();

        let mut test_value = 0u32;
        let test_value_ref = &mut test_value;

        let data = WasmValue::from_extern_ref(test_value_ref);
        let result = table.set_data(data, 0);
        assert!(result.is_ok());
        let result = table.set_data(data, 1);
        assert!(result.is_ok());
        let result = table.set_data(data, 2);
        assert!(result.is_ok());
        let result = table.set_data(data, 3);
        assert!(result.is_ok());

        // get the exported host function named "func-host-add"
        let result = active_instance.get_func("func-host-add");
        assert!(result.is_ok());
        let func_host_add = result.unwrap();
        // Call add: (777) + (223)
        test_value = 777;
        let result = executor.run_func(&func_host_add, [WasmValue::from_i32(223)]);
        assert!(result.is_ok());
        let returns = result.unwrap();
        assert_eq!(returns[0].to_i32(), 1000);

        // get the exported host function named "func-host-add"
        let result = active_instance.get_func("func-host-sub");
        assert!(result.is_ok());
        let func_host_sub = result.unwrap();
        // Call sub: (123) - (456)
        test_value = 123;
        let result = executor.run_func(&func_host_sub, [WasmValue::from_i32(456)]);
        assert!(result.is_ok());
        let returns = result.unwrap();
        assert_eq!(returns[0].to_i32(), -333);

        // get the exported host function named "func-host-add"
        let result = active_instance.get_func("func-host-mul");
        assert!(result.is_ok());
        let func_host_mul = result.unwrap();
        // Call mul: (-30) * (-66)
        test_value = -30i32 as u32;
        let result = executor.run_func(&func_host_mul, [WasmValue::from_i32(-66)]);
        assert!(result.is_ok());
        let returns = result.unwrap();
        assert_eq!(returns[0].to_i32(), 1980);

        // get the exported host function named "func-host-add"
        let result = active_instance.get_func("func-host-div");
        assert!(result.is_ok());
        let func_host_div = result.unwrap();
        // Call div: (-9999) / (1234)
        test_value = -9999i32 as u32;
        let result = executor.run_func(&func_host_div, [WasmValue::from_i32(1234)]);
        assert!(result.is_ok());
        let returns = result.unwrap();
        assert_eq!(returns[0].to_i32(), -8);

        // get the module instance named "extern"
        let result = store.module("extern");
        assert!(result.is_ok());
        let extern_instance = result.unwrap();

        // get the exported host function named "func-add"
        let result = extern_instance.get_func("func-add");
        assert!(result.is_ok());
        let func_add = result.unwrap();
        // Invoke the functions in the registered module
        test_value = 5000;
        let result = executor.run_func(
            &func_add,
            [
                WasmValue::from_extern_ref(&mut test_value),
                WasmValue::from_i32(1500),
            ],
        );
        assert!(result.is_ok());
        let returns = result.unwrap();
        assert_eq!(returns[0].to_i32(), 6500);
        // Function type mismatch
        let result = executor.run_func(&func_add, []);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            Box::new(WasmEdgeError::Core(CoreError::Execution(
                CoreExecutionError::FuncTypeMismatch
            )))
        );
        // Function type mismatch
        let result = executor.run_func(
            &func_add,
            [
                WasmValue::from_extern_ref(&mut test_value),
                WasmValue::from_i64(1500),
            ],
        );
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            Box::new(WasmEdgeError::Core(CoreError::Execution(
                CoreExecutionError::FuncTypeMismatch
            )))
        );
        // Module not found
        let result = store.module("error-name");
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            Box::new(WasmEdgeError::Store(StoreError::NotFoundModule(
                "error-name".into()
            )))
        );
        // Function not found
        let result = extern_instance.get_func("func-add2");
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            Box::new(WasmEdgeError::Instance(InstanceError::NotFoundFunc(
                "func-add2".into()
            )))
        );

        // get the exported host function named "func-term"
        let result = extern_instance.get_func("func-term");
        assert!(result.is_ok());
        let func_term = result.unwrap();
        // Invoke host function to terminate execution
        let result = executor.run_func(&func_term, []);
        assert!(result.is_ok());
        let returns = result.unwrap();
        assert_eq!(returns[0].to_i32(), 1234);

        // get the exported host function named "func-term"
        let result = extern_instance.get_func("func-fail");
        assert!(result.is_ok());
        let func_fail = result.unwrap();
        // Invoke host function to fail execution
        let result = executor.run_func(&func_fail, []);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), Box::new(WasmEdgeError::User(2)));
    }

    mod common {
        use crate::{AsImport, CallingFrame, FuncType, Function, ImportModule, WasmValue};
        use wasmedge_macro::sys_host_function;
        use wasmedge_types::{error::HostFuncError, NeverType, ValType};

        pub(crate) fn create_extern_module(name: impl AsRef<str>) -> ImportModule<NeverType> {
            // create an import module
            let result = ImportModule::<NeverType>::create(name, None);
            assert!(result.is_ok());
            let mut import = result.unwrap();

            // add host function: "func-add"
            let result =
                FuncType::create(vec![ValType::ExternRef, ValType::I32], vec![ValType::I32]);
            let func_ty = result.unwrap();
            let result =
                Function::create_sync_func::<NeverType>(&func_ty, Box::new(extern_add), None, 0);
            assert!(result.is_ok());
            let host_func = result.unwrap();
            import.add_func("func-add", host_func);

            // add host function: "func-sub"
            let result =
                FuncType::create(vec![ValType::ExternRef, ValType::I32], vec![ValType::I32]);
            let func_ty = result.unwrap();
            let result =
                Function::create_sync_func::<NeverType>(&func_ty, Box::new(extern_sub), None, 0);
            assert!(result.is_ok());
            let host_func = result.unwrap();
            import.add_func("func-sub", host_func);

            // add host function: "func-mul"
            let result =
                FuncType::create(vec![ValType::ExternRef, ValType::I32], vec![ValType::I32]);
            let func_ty = result.unwrap();
            let result =
                Function::create_sync_func::<NeverType>(&func_ty, Box::new(extern_mul), None, 0);
            assert!(result.is_ok());
            let host_func = result.unwrap();
            import.add_func("func-mul", host_func);

            // add host function: "func-div"
            let result =
                FuncType::create(vec![ValType::ExternRef, ValType::I32], vec![ValType::I32]);
            let func_ty = result.unwrap();
            let result =
                Function::create_sync_func::<NeverType>(&func_ty, Box::new(extern_div), None, 0);
            assert!(result.is_ok());
            let host_func = result.unwrap();
            import.add_func("func-div", host_func);

            // add host function: "func-term"
            let result = FuncType::create([], [ValType::I32]);
            assert!(result.is_ok());
            let func_ty = result.unwrap();
            let result =
                Function::create_sync_func::<NeverType>(&func_ty, Box::new(extern_term), None, 0);
            let host_func = result.unwrap();
            import.add_func("func-term", host_func);

            // add host function: "func-fail"
            let result = FuncType::create([], [ValType::I32]);
            assert!(result.is_ok());
            let func_ty = result.unwrap();
            let result =
                Function::create_sync_func::<NeverType>(&func_ty, Box::new(extern_fail), None, 0);
            let host_func = result.unwrap();
            import.add_func("func-fail", host_func);

            import
        }

        #[sys_host_function]
        fn extern_add(
            _frame: CallingFrame,
            inputs: Vec<WasmValue>,
        ) -> Result<Vec<WasmValue>, HostFuncError> {
            let val1 = if inputs[0].ty() == ValType::ExternRef {
                inputs[0]
            } else {
                return Err(HostFuncError::User(2));
            };
            let val1 = val1
                .extern_ref::<i32>()
                .expect("fail to get i32 from an ExternRef");

            let val2 = if inputs[1].ty() == ValType::I32 {
                inputs[1].to_i32()
            } else {
                return Err(HostFuncError::User(3));
            };

            Ok(vec![WasmValue::from_i32(val1 + val2)])
        }

        #[sys_host_function]
        fn extern_sub(
            _frame: CallingFrame,
            inputs: Vec<WasmValue>,
        ) -> Result<Vec<WasmValue>, HostFuncError> {
            let val1 = if inputs[0].ty() == ValType::ExternRef {
                inputs[0]
            } else {
                return Err(HostFuncError::User(2));
            };

            let val1 = val1
                .extern_ref::<i32>()
                .expect("fail to get i32 from an ExternRef");

            let val2 = if inputs[1].ty() == ValType::I32 {
                inputs[1].to_i32()
            } else {
                return Err(HostFuncError::User(3));
            };

            Ok(vec![WasmValue::from_i32(val1 - val2)])
        }

        #[sys_host_function]
        fn extern_mul(
            _frame: CallingFrame,
            inputs: Vec<WasmValue>,
        ) -> Result<Vec<WasmValue>, HostFuncError> {
            let val1 = if inputs[0].ty() == ValType::ExternRef {
                inputs[0]
            } else {
                return Err(HostFuncError::User(2));
            };
            let val1 = val1
                .extern_ref::<i32>()
                .expect("fail to get i32 from an ExternRef");

            let val2 = if inputs[1].ty() == ValType::I32 {
                inputs[1].to_i32()
            } else {
                return Err(HostFuncError::User(3));
            };

            Ok(vec![WasmValue::from_i32(val1 * val2)])
        }

        #[sys_host_function]
        fn extern_div(
            _frame: CallingFrame,
            inputs: Vec<WasmValue>,
        ) -> Result<Vec<WasmValue>, HostFuncError> {
            let val1 = if inputs[0].ty() == ValType::ExternRef {
                inputs[0]
            } else {
                return Err(HostFuncError::User(2));
            };
            let val1 = val1
                .extern_ref::<i32>()
                .expect("fail to get i32 from an ExternRef");

            let val2 = if inputs[1].ty() == ValType::I32 {
                inputs[1].to_i32()
            } else {
                return Err(HostFuncError::User(3));
            };

            Ok(vec![WasmValue::from_i32(val1 / val2)])
        }

        #[sys_host_function]
        fn extern_term(
            _frame: CallingFrame,
            _inputs: Vec<WasmValue>,
        ) -> Result<Vec<WasmValue>, HostFuncError> {
            Ok(vec![WasmValue::from_i32(1234)])
        }

        #[sys_host_function]
        fn extern_fail(
            _frame: CallingFrame,
            _inputs: Vec<WasmValue>,
        ) -> Result<Vec<WasmValue>, HostFuncError> {
            Err(HostFuncError::User(2))
        }
    }
}
