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
///                (local.get $n)
///                (i32.const 2)
///               )
///               (then
///                (return (i32.const 1))
///               )
///              )
///              (return
///               (i32.add
///                (call $fib
///                 (i32.sub
///                  (local.get $n)
///                  (i32.const 2)
///                 )
///                )
///                (call $fib
///                 (i32.sub
///                  (local.get $n)
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use wasmedge_types::wat2wasm;

    use super::*;
    use crate::{io::WasmVal, params};

    #[tokio::test]
    async fn test_vm_run_func_from_file() {
        // create a Vm context
        let mut vm = Vm::new(
            Store::new(None, HashMap::<String, &mut (dyn AsyncInst + Send)>::new()).unwrap(),
        );

        // register a wasm module from a specified wasm file
        let file = std::env::current_dir()
            .unwrap()
            .join("examples/wasmedge-sys/data/fibonacci.wat");

        // run `fib` function from the wasm file
        let fib_module = Module::from_file(None, file).unwrap();
        vm.register_module(None, fib_module).unwrap();
        let result = vm.run_func(None, "fib", params!(10)).await;
        assert!(result.is_ok());
        let returns = result.unwrap();
        assert_eq!(returns.len(), 1);
        assert_eq!(returns[0].to_i32(), 89);
    }

    #[tokio::test]
    async fn test_vm_run_func_from_bytes() {
        // create a Vm context
        let mut vm = Vm::new(
            Store::new(None, HashMap::<String, &mut (dyn AsyncInst + Send)>::new()).unwrap(),
        );

        // register a wasm module from the given in-memory wasm bytes
        // load wasm module
        let result = wat2wasm(
            br#"(module
            (export "fib" (func $fib))
            (func $fib (param $n i32) (result i32)
             (if
              (i32.lt_s
               (local.get $n)
               (i32.const 2)
              )
              (then
                (return (i32.const 1))
              )
             )
             (return
              (i32.add
               (call $fib
                (i32.sub
                 (local.get $n)
                 (i32.const 2)
                )
               )
               (call $fib
                (i32.sub
                 (local.get $n)
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
        let result = vm.run_func(None, "fib", params!(10)).await;
        assert!(result.is_ok());
        let returns = result.unwrap();
        assert_eq!(returns.len(), 1);
        assert_eq!(returns[0].to_i32(), 89);
    }

    #[tokio::test]
    async fn test_vm_run_func_in_named_module_instance() {
        // create a Vm context
        let mut vm = Vm::new(
            Store::new(None, HashMap::<String, &mut (dyn AsyncInst + Send)>::new()).unwrap(),
        );

        // register a wasm module from the given in-memory wasm bytes
        // load wasm module
        let result = wat2wasm(
            br#"(module
            (export "fib" (func $fib))
            (func $fib (param $n i32) (result i32)
             (if
              (i32.lt_s
               (local.get $n)
               (i32.const 2)
              )
              (then
                (return (i32.const 1))
              )
             )
             (return
              (i32.add
               (call $fib
                (i32.sub
                 (local.get $n)
                 (i32.const 2)
                )
               )
               (call $fib
                (i32.sub
                 (local.get $n)
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
        let result = vm.run_func(Some("extern"), "fib", params!(10)).await;
        assert!(result.is_ok());
        let returns = result.unwrap();
        assert_eq!(returns.len(), 1);
        assert_eq!(returns[0].to_i32(), 89);
    }
}
