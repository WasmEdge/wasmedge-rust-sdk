//! Defines WasmEdge Vm struct.
use crate::{
    error::{VmError, WasmEdgeError},
    vm::SyncInst,
    Instance, Module, Store, WasmEdgeResult, WasmValue,
};
use sys::{r#async::fiber::AsyncState, AsInstance};
use wasmedge_sys as sys;

use super::import::ImportObject;

pub trait AsyncInst: AsInstance {}

impl<T: Send> AsyncInst for ImportObject<T> {}
impl<T: Send + SyncInst> AsyncInst for T {}

/// A [Vm] defines a virtual environment for managing WebAssembly programs.
///
/// # Example
///
/// The example below presents how to register a module as named module in a Vm instance and run a target wasm function.
///
/// ```rust
/// // If the version of rust used is less than v1.63, please uncomment the follow attribute.
/// // #![feature(explicit_generic_args_with_impl_trait)]
/// #[cfg(not(feature = "async"))]
/// use wasmedge_sdk::{params, VmBuilder, WasmVal, wat2wasm, ValType, NeverType};
///
/// #[cfg_attr(test, test)]
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     #[cfg(not(feature = "async"))]
///     {
///         // create a Vm context
///         let vm = VmBuilder::new().build()?;
///
///         // register a wasm module from the given in-memory wasm bytes
///         let wasm_bytes = wat2wasm(
///             br#"(module
///             (export "fib" (func $fib))
///             (func $fib (param $n i32) (result i32)
///              (if
///               (i32.lt_s
///                (get_local $n)
///                (i32.const 2)
///               )
///               (return
///                (i32.const 1)
///               )
///              )
///              (return
///               (i32.add
///                (call $fib
///                 (i32.sub
///                  (get_local $n)
///                  (i32.const 2)
///                 )
///                )
///                (call $fib
///                 (i32.sub
///                  (get_local $n)
///                  (i32.const 1)
///                 )
///                )
///               )
///              )
///             )
///            )
///         "#,
///         )?;
///         let mut vm = vm.register_module_from_bytes("extern", wasm_bytes)?;
///
///         // run `fib` function in the named module instance
///         let returns = vm.run_func(Some("extern"), "fib", params!(10))?;
///         assert_eq!(returns.len(), 1);
///         assert_eq!(returns[0].to_i32(), 89);
///     }
///
///     Ok(())
/// }
/// ```
#[derive(Debug)]
pub struct Vm<'inst, T: ?Sized + Send + AsyncInst> {
    store: Store<'inst, T>,
    active_instance: Option<sys::Instance>,
    async_state: AsyncState,
}
impl<'inst, T: ?Sized + Send + AsyncInst> Vm<'inst, T> {
    pub fn new(store: Store<'inst, T>) -> Self {
        // create a Vm instance
        Self {
            store,
            active_instance: None,
            async_state: AsyncState::new(),
        }
    }

    /// Registers a [wasm module](crate::Module) into this vm as a named or active module [instance](crate::Instance).
    ///
    /// # Arguments
    ///
    /// * `mod_name` - The exported name for the registered module. If `None`, then the module is registered as an active instance.
    ///
    /// * `module` - The module to be registered.
    ///
    /// # Error
    ///
    /// If fail to register the given [module](crate::Module), then an error is returned.
    ///
    pub fn register_module(
        &mut self,
        mod_name: Option<&str>,
        module: Module,
    ) -> WasmEdgeResult<&mut Self> {
        match mod_name {
            Some(name) => {
                self.store.register_named_module(name, &module)?;
            }
            None => {
                self.active_instance = Some(self.store.register_active_module(&module)?);
            }
        };

        Ok(self)
    }

    /// Runs an exported wasm function in a (named or active) [module instance](crate::Instance).
    ///
    /// # Arguments
    ///
    /// * `mod_name` - The exported name of the module instance, which holds the target function. If `None`, then the active module is used.
    ///
    /// * `func_name` - The exported name of the target wasm function.
    ///
    /// * `args` - The arguments to be passed to the target wasm function.
    ///
    /// # Error
    ///
    /// If fail to run the wasm function, then an error is returned.
    pub async fn run_func(
        &mut self,
        mod_name: Option<&str>,
        func_name: impl AsRef<str>,
        args: impl IntoIterator<Item = WasmValue> + Send,
    ) -> WasmEdgeResult<Vec<WasmValue>> {
        let (mut func, executor) = match mod_name {
            Some(mod_name) => {
                if let Some((inst, executor)) = self.store.get_instance_and_executor(mod_name) {
                    (inst.get_func_mut(func_name.as_ref())?, executor)
                } else if let Some((wasm_mod, executor)) =
                    self.store.get_named_wasm_and_executor(mod_name)
                {
                    (wasm_mod.get_func_mut(func_name.as_ref())?, executor)
                } else {
                    return Err(Box::new(WasmEdgeError::Vm(VmError::NotFoundModule(
                        mod_name.into(),
                    ))));
                }
            }
            None => {
                let active_inst = self
                    .active_instance
                    .as_mut()
                    .ok_or(Box::new(WasmEdgeError::Vm(VmError::NotFoundActiveModule)))?;

                (
                    active_inst.get_func_mut(func_name.as_ref())?,
                    self.store.executor(),
                )
            }
        };
        executor
            .call_func_async(&self.async_state, &mut func, args)
            .await
    }

    /// Runs an exported wasm function in a (named or active) [module instance](crate::Instance) with a timeout setting
    ///
    /// # Arguments
    ///
    /// * `mod_name` - The exported name of the module instance, which holds the target function. If `None`, then the active module is used.
    ///
    /// * `func_name` - The exported name of the target wasm function.
    ///
    /// * `args` - The arguments to be passed to the target wasm function.
    ///
    /// * `timeout` - The maximum execution time of the function to be run.
    ///
    /// # Error
    ///
    /// If fail to run the wasm function, then an error is returned.
    #[cfg(all(target_os = "linux", not(target_env = "musl")))]
    pub async fn run_func_with_timeout(
        &mut self,
        mod_name: Option<&str>,
        func_name: impl AsRef<str>,
        args: impl IntoIterator<Item = WasmValue> + Send,
        timeout: std::time::Duration,
    ) -> WasmEdgeResult<Vec<WasmValue>> {
        let (mut func, executor) = match mod_name {
            Some(mod_name) => {
                if let Some((inst, executor)) = self.store.get_instance_and_executor(mod_name) {
                    (inst.get_func_mut(func_name.as_ref())?, executor)
                } else if let Some((wasm_mod, executor)) =
                    self.store.get_named_wasm_and_executor(mod_name)
                {
                    (wasm_mod.get_func_mut(func_name.as_ref())?, executor)
                } else {
                    return Err(Box::new(WasmEdgeError::Vm(VmError::NotFoundModule(
                        mod_name.into(),
                    ))));
                }
            }
            None => {
                let active_inst = self
                    .active_instance
                    .as_mut()
                    .ok_or(Box::new(WasmEdgeError::Vm(VmError::NotFoundActiveModule)))?;

                (
                    active_inst.get_func_mut(func_name.as_ref())?,
                    self.store.executor(),
                )
            }
        };
        executor
            .call_func_async_with_timeout(&self.async_state, &mut func, args, timeout)
            .await
    }

    /// Returns a reference to the internal [store](crate::Store) from this vm.
    pub fn store(&self) -> &Store<'inst, T> {
        &self.store
    }

    /// Returns a mutable reference to the internal [store](crate::Store) from this vm.
    pub fn store_mut(&mut self) -> &mut Store<'inst, T> {
        &mut self.store
    }

    /// Returns a reference to the active [module instance](crate::Instance) from this vm.
    ///
    /// # Error
    ///
    /// If fail to get the reference to the active module instance, then an error is returned.
    pub fn active_module(&self) -> Option<&Instance> {
        self.active_instance.as_ref()
    }

    /// Returns a mutable reference to the active [module instance](crate::Instance) from this vm.
    ///
    /// # Error
    ///
    /// If fail to get the mutable reference to the active module instance, then an error is returned.
    pub fn active_module_mut(&mut self) -> Option<&mut Instance> {
        self.active_instance.as_mut()
    }

    /// Checks if the vm contains a named module instance.
    ///
    /// # Argument
    ///
    /// * `mod_name` - The exported name of the target module instance.
    ///
    pub fn contains_module(&self, mod_name: impl AsRef<str>) -> bool {
        self.store.contains(mod_name)
    }

    /// Returns the count of the named [module instances](crate::Instance) this vm holds.
    pub fn named_instance_count(&self) -> usize {
        self.store.named_instance_count()
    }

    /// Returns the names of all named [module instances](crate::Instance) this vm holds.
    pub fn instance_names(&self) -> Vec<String> {
        self.store.instance_names()
    }
}

// #[cfg(test)]
#[cfg(ignore)]
mod tests {
    use super::*;
    use crate::{
        config::{
            CommonConfigOptions, ConfigBuilder, HostRegistrationConfigOptions,
            StatisticsConfigOptions,
        },
        error::HostFuncError,
        io::WasmVal,
        params,
        types::Val,
        wat2wasm, CallingFrame, Global, GlobalType, ImportObjectBuilder, Memory, MemoryType,
        Mutability, NeverType, RefType, Table, TableType, ValType, WasmValue,
    };

    #[test]
    fn test_vm_run_func_from_file() {
        // create a Vm context
        let result = VmBuilder::new().build();
        assert!(result.is_ok());
        let mut vm = result.unwrap();

        // register a wasm module from a specified wasm file
        let file = std::env::current_dir()
            .unwrap()
            .join("examples/wasmedge-sys/data/fibonacci.wat");

        // run `fib` function from the wasm file
        let result = vm.run_func_from_file(file, "fib", params!(10));
        assert!(result.is_ok());
        let returns = result.unwrap();
        assert_eq!(returns.len(), 1);
        assert_eq!(returns[0].to_i32(), 89);
    }

    #[test]
    fn test_vm_run_func_from_bytes() {
        // create a Vm context
        let result = VmBuilder::new().build();
        assert!(result.is_ok());
        let mut vm = result.unwrap();

        // register a wasm module from the given in-memory wasm bytes
        // load wasm module
        let result = wat2wasm(
            br#"(module
            (export "fib" (func $fib))
            (func $fib (param $n i32) (result i32)
             (if
              (i32.lt_s
               (get_local $n)
               (i32.const 2)
              )
              (return
               (i32.const 1)
              )
             )
             (return
              (i32.add
               (call $fib
                (i32.sub
                 (get_local $n)
                 (i32.const 2)
                )
               )
               (call $fib
                (i32.sub
                 (get_local $n)
                 (i32.const 1)
                )
               )
              )
             )
            )
           )
        "#,
        );
        assert!(result.is_ok());
        let wasm_bytes = result.unwrap();

        // run `fib` function from the wasm bytes
        let result = vm.run_func_from_bytes(&wasm_bytes, "fib", params!(10));
        assert!(result.is_ok());
        let returns = result.unwrap();
        assert_eq!(returns.len(), 1);
        assert_eq!(returns[0].to_i32(), 89);
    }

    #[test]
    fn test_vm_run_func_from_module() {
        // create a Vm context
        let result = VmBuilder::new().build();
        assert!(result.is_ok());
        let mut vm = result.unwrap();

        // load wasm module
        let result = wat2wasm(
            br#"(module
            (export "fib" (func $fib))
            (func $fib (param $n i32) (result i32)
             (if
              (i32.lt_s
               (get_local $n)
               (i32.const 2)
              )
              (return
               (i32.const 1)
              )
             )
             (return
              (i32.add
               (call $fib
                (i32.sub
                 (get_local $n)
                 (i32.const 2)
                )
               )
               (call $fib
                (i32.sub
                 (get_local $n)
                 (i32.const 1)
                )
               )
              )
             )
            )
           )
        "#,
        );
        assert!(result.is_ok());
        let wasm_bytes = result.unwrap();
        let result = Module::from_bytes(None, wasm_bytes);
        assert!(result.is_ok());
        let module = result.unwrap();

        // run `fib` function from the compiled module
        let result = vm.run_func_from_module(module, "fib", params!(10));
        assert!(result.is_ok());
        let returns = result.unwrap();
        assert_eq!(returns.len(), 1);
        assert_eq!(returns[0].to_i32(), 89);
    }

    #[test]
    fn test_vm_run_func_in_named_module_instance() {
        // create a Vm context
        let result = VmBuilder::new().build();
        assert!(result.is_ok());
        let vm = result.unwrap();

        // register a wasm module from the given in-memory wasm bytes
        // load wasm module
        let result = wat2wasm(
            br#"(module
            (export "fib" (func $fib))
            (func $fib (param $n i32) (result i32)
             (if
              (i32.lt_s
               (get_local $n)
               (i32.const 2)
              )
              (return
               (i32.const 1)
              )
             )
             (return
              (i32.add
               (call $fib
                (i32.sub
                 (get_local $n)
                 (i32.const 2)
                )
               )
               (call $fib
                (i32.sub
                 (get_local $n)
                 (i32.const 1)
                )
               )
              )
             )
            )
           )
        "#,
        );
        assert!(result.is_ok());
        let wasm_bytes = result.unwrap();
        let result = vm.register_module_from_bytes("extern", wasm_bytes);
        assert!(result.is_ok());
        let vm = result.unwrap();

        // run `fib` function in the named module instance
        let result = vm.run_func(Some("extern"), "fib", params!(10));
        assert!(result.is_ok());
        let returns = result.unwrap();
        assert_eq!(returns.len(), 1);
        assert_eq!(returns[0].to_i32(), 89);
    }

    #[test]
    fn test_vm_run_func_in_active_module_instance() {
        // create a Vm context
        let result = VmBuilder::new().build();
        assert!(result.is_ok());
        let vm = result.unwrap();

        // load wasm module
        let result = wat2wasm(
            br#"(module
            (export "fib" (func $fib))
            (func $fib (param $n i32) (result i32)
             (if
              (i32.lt_s
               (get_local $n)
               (i32.const 2)
              )
              (return
               (i32.const 1)
              )
             )
             (return
              (i32.add
               (call $fib
                (i32.sub
                 (get_local $n)
                 (i32.const 2)
                )
               )
               (call $fib
                (i32.sub
                 (get_local $n)
                 (i32.const 1)
                )
               )
              )
             )
            )
           )
        "#,
        );
        assert!(result.is_ok());
        let wasm_bytes = result.unwrap();
        let result = Module::from_bytes(None, wasm_bytes);
        assert!(result.is_ok());
        let module = result.unwrap();

        // register the wasm module into vm
        let result = vm.register_module(None, module);
        assert!(result.is_ok());
        let vm = result.unwrap();

        // run `fib` function in the active module instance
        let result = vm.run_func(None, "fib", params!(10));
        assert!(result.is_ok());
        let returns = result.unwrap();
        assert_eq!(returns.len(), 1);
        assert_eq!(returns[0].to_i32(), 89);
    }

    #[test]
    #[allow(clippy::assertions_on_result_states)]
    fn test_vm_create() {
        {
            let result = VmBuilder::new().build();
            assert!(result.is_ok());
        }

        {
            // create a Config
            let result = ConfigBuilder::new(CommonConfigOptions::default()).build();
            assert!(result.is_ok());
            let config = result.unwrap();

            // create a Vm context
            let result = VmBuilder::new().with_config(config).build();
            assert!(result.is_ok());
            let _vm = result.unwrap();
        }
    }

    #[test]
    fn test_vm_wasi_module() {
        let host_reg_options = HostRegistrationConfigOptions::default().wasi(true);
        let result = ConfigBuilder::new(CommonConfigOptions::default())
            .with_host_registration_config(host_reg_options)
            .build();
        assert!(result.is_ok());
        let config = result.unwrap();

        // create a vm with the config settings
        let result = VmBuilder::new().with_config(config).build();
        assert!(result.is_ok());
        let vm = result.unwrap();

        // get the wasi module
        let result = vm.wasi_module();
        assert!(result.is_some());
        let wasi_instance = result.unwrap();

        assert_eq!(wasi_instance.name(), "wasi_snapshot_preview1");
    }

    #[test]
    fn test_vm_statistics() {
        // set config options related to Statistics
        let stat_config_options = StatisticsConfigOptions::new()
            .measure_cost(true)
            .measure_time(true)
            .count_instructions(true);
        // create a Config
        let result = ConfigBuilder::new(CommonConfigOptions::default())
            .with_statistics_config(stat_config_options)
            .build();
        assert!(result.is_ok());
        let config = result.unwrap();

        // create a Vm context
        let result = VmBuilder::new().with_config(config).build();
        assert!(result.is_ok());
        let _vm = result.unwrap();

        // get the statistics
        // let _stat = vm.statistics_mut();
    }

    #[test]
    #[allow(clippy::assertions_on_result_states)]
    fn test_vm_register_module_from_file() {
        {
            // create a Vm context
            let result = VmBuilder::new().build();
            assert!(result.is_ok());
            let vm = result.unwrap();

            // register a wasm module from a specified wasm file
            let file = std::env::current_dir()
                .unwrap()
                .join("examples/wasmedge-sys/data/fibonacci.wat");
            let result = vm.register_module_from_file("extern", file);
            assert!(result.is_ok());
            let vm = result.unwrap();

            assert!(vm.named_instance_count() >= 1);
            assert!(vm.instance_names().iter().any(|x| x == "extern"));
        }

        {
            // create a Vm context
            let result = VmBuilder::new().build();
            assert!(result.is_ok());
            let vm = result.unwrap();

            // register a wasm module from a specified wasm file
            let file = std::env::current_dir()
                .unwrap()
                .join("examples/wasmedge-sys/data/fibonacci.wat");
            let result = vm.register_module_from_file("extern", file);
            assert!(result.is_ok());
            let vm = result.unwrap();

            assert!(vm.named_instance_count() >= 1);
            assert!(vm.instance_names().iter().any(|x| x == "extern"));
        }
    }

    #[test]
    #[allow(clippy::assertions_on_result_states)]
    fn test_vm_register_module_from_bytes() {
        // create a Vm context
        let result = VmBuilder::new().build();
        assert!(result.is_ok());
        let vm = result.unwrap();

        // register a wasm module from the given in-memory wasm bytes
        // load wasm module
        let result = wat2wasm(
            br#"(module
            (export "fib" (func $fib))
            (func $fib (param $n i32) (result i32)
             (if
              (i32.lt_s
               (get_local $n)
               (i32.const 2)
              )
              (return
               (i32.const 1)
              )
             )
             (return
              (i32.add
               (call $fib
                (i32.sub
                 (get_local $n)
                 (i32.const 2)
                )
               )
               (call $fib
                (i32.sub
                 (get_local $n)
                 (i32.const 1)
                )
               )
              )
             )
            )
           )
        "#,
        );
        assert!(result.is_ok());
        let wasm_bytes = result.unwrap();
        let result = vm.register_module_from_bytes("extern", wasm_bytes);
        assert!(result.is_ok());
        let vm = result.unwrap();

        assert!(vm.named_instance_count() >= 1);
        assert!(vm.instance_names().iter().any(|x| x == "extern"));
    }

    #[test]
    #[allow(clippy::assertions_on_result_states)]
    fn test_vm_register_import_module() {
        // create a Const global instance
        let result = Global::new(
            GlobalType::new(ValType::F32, Mutability::Const),
            Val::F32(3.5),
        );
        assert!(result.is_ok());
        let global_const = result.unwrap();

        // create a memory instance
        let result = MemoryType::new(10, None, false);
        assert!(result.is_ok());
        let memory_type = result.unwrap();
        let result = Memory::new(memory_type);
        assert!(result.is_ok());
        let memory = result.unwrap();

        // create a table instance
        let result = Table::new(TableType::new(RefType::FuncRef, 5, None));
        assert!(result.is_ok());
        let table = result.unwrap();

        // create an ImportModule instance
        let result = ImportObjectBuilder::new()
            .with_func::<(i32, i32), i32, NeverType>("add", real_add, None)
            .expect("failed to add host function")
            .with_global("global", global_const)
            .with_memory("mem", memory)
            .with_table("table", table)
            .build::<NeverType>("extern-module", None);
        assert!(result.is_ok());
        let import = result.unwrap();

        // create a Vm context
        let result = VmBuilder::new().build();
        assert!(result.is_ok());
        let mut vm = result.unwrap();

        // register an import module into vm
        let result = vm.register_import_module(&import);
        assert!(result.is_ok());

        assert!(vm.named_instance_count() >= 1);
        assert!(vm.instance_names().iter().any(|x| x == "extern-module"));

        // get active module instance
        let result = vm.named_module("extern-module");
        assert!(result.is_ok());
        let instance = result.unwrap();
        assert!(instance.name().is_some());
        assert_eq!(instance.name().unwrap(), "extern-module");

        let result = instance.global("global");
        assert!(result.is_ok());
        let global = result.unwrap();
        let ty = global.ty();
        assert_eq!(*ty, GlobalType::new(ValType::F32, Mutability::Const));
    }

    #[test]
    fn test_vm_register_named_module() {
        // create a Vm context
        let result = VmBuilder::new().build();
        assert!(result.is_ok());
        let vm = result.unwrap();

        // load wasm module
        let result = wat2wasm(
            br#"(module
            (export "fib" (func $fib))
            (func $fib (param $n i32) (result i32)
             (if
              (i32.lt_s
               (get_local $n)
               (i32.const 2)
              )
              (return
               (i32.const 1)
              )
             )
             (return
              (i32.add
               (call $fib
                (i32.sub
                 (get_local $n)
                 (i32.const 2)
                )
               )
               (call $fib
                (i32.sub
                 (get_local $n)
                 (i32.const 1)
                )
               )
              )
             )
            )
           )
        "#,
        );
        assert!(result.is_ok());
        let wasm_bytes = result.unwrap();
        let result = Module::from_bytes(None, wasm_bytes);
        assert!(result.is_ok());
        let module = result.unwrap();

        // register the wasm module into vm
        let result = vm.register_module(Some("extern"), module);
        assert!(result.is_ok());
        let vm = result.unwrap();

        // check the exported functions in the "extern" module
        assert!(vm.named_instance_count() >= 1);
        let result = vm.named_module("extern");
        assert!(result.is_ok());
        let instance = result.unwrap();

        assert_eq!(instance.func_count(), 1);
        let result = instance.func_names();
        assert!(result.is_some());
        let func_names = result.unwrap();
        assert_eq!(func_names, ["fib"]);

        // get host_func
        let result = instance.func("fib");
        assert!(result.is_ok());
        let fib = result.unwrap();

        // check the type of host_func
        let ty = fib.ty();
        assert!(ty.args().is_some());
        assert_eq!(ty.args().unwrap(), [ValType::I32]);
        assert!(ty.returns().is_some());
        assert_eq!(ty.returns().unwrap(), [ValType::I32]);
    }

    #[test]
    fn test_vm_register_active_module() {
        // create a Vm context
        let result = VmBuilder::new().build();
        assert!(result.is_ok());
        let vm = result.unwrap();

        // load wasm module
        let result = wat2wasm(
            br#"(module
            (export "fib" (func $fib))
            (func $fib (param $n i32) (result i32)
             (if
              (i32.lt_s
               (get_local $n)
               (i32.const 2)
              )
              (return
               (i32.const 1)
              )
             )
             (return
              (i32.add
               (call $fib
                (i32.sub
                 (get_local $n)
                 (i32.const 2)
                )
               )
               (call $fib
                (i32.sub
                 (get_local $n)
                 (i32.const 1)
                )
               )
              )
             )
            )
           )
        "#,
        );
        assert!(result.is_ok());
        let wasm_bytes = result.unwrap();
        let result = Module::from_bytes(None, wasm_bytes);
        assert!(result.is_ok());
        let module = result.unwrap();

        // register the wasm module into vm
        let result = vm.register_module(None, module);
        assert!(result.is_ok());
        let vm = result.unwrap();

        // check the exported functions in the "extern" module
        let result = vm.active_module();
        assert!(result.is_ok());
        let instance = result.unwrap();

        assert_eq!(instance.func_count(), 1);
        let result = instance.func_names();
        assert!(result.is_some());
        let func_names = result.unwrap();
        assert_eq!(func_names, ["fib"]);

        // get host_func
        let result = instance.func("fib");
        assert!(result.is_ok());
        let fib = result.unwrap();

        // check the type of host_func
        let ty = fib.ty();
        assert!(ty.args().is_some());
        assert_eq!(ty.args().unwrap(), [ValType::I32]);
        assert!(ty.returns().is_some());
        assert_eq!(ty.returns().unwrap(), [ValType::I32]);
    }

    fn real_add(
        _frame: CallingFrame,
        inputs: Vec<WasmValue>,
        _data: *mut std::os::raw::c_void,
    ) -> std::result::Result<Vec<WasmValue>, HostFuncError> {
        if inputs.len() != 2 {
            return Err(HostFuncError::User(1));
        }

        let a = if inputs[0].ty() == ValType::I32 {
            inputs[0].to_i32()
        } else {
            return Err(HostFuncError::User(2));
        };

        let b = if inputs[1].ty() == ValType::I32 {
            inputs[1].to_i32()
        } else {
            return Err(HostFuncError::User(3));
        };

        let c = a + b;

        Ok(vec![WasmValue::from_i32(c)])
    }
}
