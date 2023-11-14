//! Defines WasmEdge Vm struct.
use crate::{
    error::{VmError, WasmEdgeError},
    ImportObject, Instance, Module, Store, WasmEdgeResult, WasmValue,
};
use sys::AsInstance;
use wasmedge_sys as sys;

pub trait SyncInst: AsInstance {}
impl<T> SyncInst for ImportObject<T> {}
impl SyncInst for Instance {}

/// A [Vm] defines a virtual environment for managing WebAssembly programs.
///
/// # Example
///
/// The example below presents how to register a module as named module in a Vm instance and run a target wasm function.
///
/// ```rust
/// use std::collections::HashMap;
/// use wasmedge_sdk::{params, Store, Module, WasmVal, wat2wasm, ValType, NeverType, Vm, vm::SyncInst};
///
/// // create a Vm context
/// let mut vm =
///     Vm::new(Store::new(None, HashMap::<String, &mut dyn SyncInst>::new()).unwrap());
/// // register a wasm module from the given in-memory wasm bytes
/// // load wasm module
/// let result = wat2wasm(
///     br#"(module
///     (export "fib" (func $fib))
///     (func $fib (param $n i32) (result i32)
///      (if
///       (i32.lt_s
///        (get_local $n)
///        (i32.const 2)
///       )
///       (return
///        (i32.const 1)
///       )
///      )
///      (return
///       (i32.add
///        (call $fib
///         (i32.sub
///          (get_local $n)
///          (i32.const 2)
///         )
///        )
///        (call $fib
///         (i32.sub
///          (get_local $n)
///          (i32.const 1)
///         )
///        )
///       )
///      )
///     )
///    )
/// "#,
/// );
/// assert!(result.is_ok());
/// let wasm_bytes = result.unwrap();
/// // run `fib` function from the wasm bytes
/// let fib_module = Module::from_bytes(None, wasm_bytes).unwrap();
/// vm.register_module(None, fib_module).unwrap();
/// let result = vm.run_func(None, "fib", params!(10i32));
/// assert!(result.is_ok());
/// let returns = result.unwrap();
/// assert_eq!(returns.len(), 1);
/// assert_eq!(returns[0].to_i32(), 89);
/// ```
#[derive(Debug)]
pub struct Vm<'inst, T: ?Sized + SyncInst> {
    store: Store<'inst, T>,
    active_instance: Option<sys::Instance>,
}
impl<'inst, T: ?Sized + SyncInst> Vm<'inst, T> {
    pub fn new(store: Store<'inst, T>) -> Self {
        // create a Vm instance
        Vm {
            store,
            active_instance: None,
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
    pub fn run_func(
        &mut self,
        mod_name: Option<&str>,
        func_name: impl AsRef<str>,
        args: impl IntoIterator<Item = WasmValue>,
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
        executor.call_func(&mut func, args)
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
    pub fn run_func_with_timeout(
        &mut self,
        mod_name: Option<&str>,
        func_name: impl AsRef<str>,
        args: impl IntoIterator<Item = WasmValue>,
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
        executor.call_func_with_timeout(&mut func, args, timeout)
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use wasmedge_types::wat2wasm;

    use super::*;
    use crate::{params, WasmVal};

    #[cfg(target_os = "linux")]
    #[test]
    fn test_vmbuilder() -> Result<(), Box<dyn std::error::Error>> {
        use crate::{params, plugin::PluginManager};

        // load plugins from the default plugin path
        PluginManager::load(None)?;

        PluginManager::names().iter().for_each(|name| {
            println!("plugin name: {}", name);
        });

        let wasm_app_file = "examples/wasmedge-sys/data/test_crypto.wasm";

        let mut wasi = crate::wasi::WasiModule::create(None, None, None).unwrap();
        let mut wasi_crypto_asymmetric_common =
            PluginManager::load_wasi_crypto_asymmetric_common().unwrap();
        let mut wasi_crypto_signatures = PluginManager::load_wasi_crypto_signatures().unwrap();
        let mut wasi_crypto_symmetric = PluginManager::load_wasi_crypto_symmetric().unwrap();

        let mut instances = HashMap::new();
        instances.insert(wasi.name().to_string(), wasi.as_mut());
        instances.insert(
            wasi_crypto_asymmetric_common.name().unwrap(),
            &mut wasi_crypto_asymmetric_common,
        );
        instances.insert(
            wasi_crypto_signatures.name().unwrap(),
            &mut wasi_crypto_signatures,
        );
        instances.insert(
            wasi_crypto_symmetric.name().unwrap(),
            &mut wasi_crypto_symmetric,
        );

        let mut vm = Vm::new(Store::new(None, instances).unwrap());

        let module = Module::from_file(None, &wasm_app_file).unwrap();

        vm.register_module(Some("wasm-app"), module).unwrap();
        vm.run_func(Some("wasm-app"), "_start", params!())?;

        Ok(())
    }

    #[test]
    fn test_vm_run_func_from_file() {
        // create a Vm context
        let mut vm =
            Vm::new(Store::new(None, HashMap::<String, &mut dyn SyncInst>::new()).unwrap());

        // register a wasm module from a specified wasm file
        let file = std::env::current_dir()
            .unwrap()
            .join("examples/wasmedge-sys/data/fibonacci.wat");

        // run `fib` function from the wasm file
        let fib_module = Module::from_file(None, file).unwrap();
        vm.register_module(None, fib_module).unwrap();
        let result = vm.run_func(None, "fib", params!(10i32));
        assert!(result.is_ok());
        let returns = result.unwrap();
        assert_eq!(returns.len(), 1);
        assert_eq!(returns[0].to_i32(), 89);
    }

    #[test]
    fn test_vm_run_func_from_bytes() {
        // create a Vm context
        let mut vm =
            Vm::new(Store::new(None, HashMap::<String, &mut dyn SyncInst>::new()).unwrap());

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
        let fib_module = Module::from_bytes(None, wasm_bytes).unwrap();
        vm.register_module(None, fib_module).unwrap();
        let result = vm.run_func(None, "fib", params!(10i32));
        assert!(result.is_ok());
        let returns = result.unwrap();
        assert_eq!(returns.len(), 1);
        assert_eq!(returns[0].to_i32(), 89);
    }

    #[test]
    fn test_vm_run_func_in_named_module_instance() {
        // create a Vm context
        let mut vm =
            Vm::new(Store::new(None, HashMap::<String, &mut dyn SyncInst>::new()).unwrap());

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
        let fib_module = Module::from_bytes(None, wasm_bytes).unwrap();
        vm.register_module(Some("extern"), fib_module).unwrap();
        // run `fib` function in the named module instance
        let result = vm.run_func(Some("extern"), "fib", params!(10));
        assert!(result.is_ok());
        let returns = result.unwrap();
        assert_eq!(returns.len(), 1);
        assert_eq!(returns[0].to_i32(), 89);
    }
}
