//! Defines WasmEdge Store struct.

use std::{collections::HashMap, fmt::Debug};

use crate::{config::Config, Module, WasmEdgeResult};
use sys::{AsInstance, Instance};
use wasmedge_sys as sys;

/// Represents all global state that can be manipulated by WebAssembly programs. A [store](crate::Store) consists of the runtime representation of all instances of [functions](crate::Func), [tables](crate::Table), [memories](crate::Memory), and [globals](crate::Global).
// #[derive(Debug)]
pub struct Store<'inst, T: ?Sized> {
    pub(crate) inner: sys::Store,
    pub(crate) instances: HashMap<String, &'inst mut T>,
    pub(crate) wasm_instance_map: HashMap<String, Instance>,
    pub(crate) executor: sys::Executor,
}

impl<T: ?Sized> Debug for Store<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Store")
            .field("inner", &self.inner)
            .field("instance_map", &self.instances.keys())
            .field("wasm_instance_map", &self.wasm_instance_map.keys())
            .field("wasm_instance_map", &self.executor)
            .finish()
    }
}

impl<'inst, T: AsInstance + ?Sized> Store<'inst, T> {
    /// Creates a new [Store].
    ///
    /// # Error
    ///
    /// If fail to create a new [Store], then an error is returned.
    pub fn new(
        config: Option<&Config>,
        instances: HashMap<String, &'inst mut T>,
    ) -> WasmEdgeResult<Self> {
        let mut store = sys::Store::create()?;
        let mut executor = sys::Executor::create(config.map(|cfg| cfg.inner.as_ref()), None)?;

        for (_k, v) in &instances {
            executor.register_import_module(&mut store, *v)?;
        }

        Ok(Self {
            inner: store,
            instances,
            wasm_instance_map: Default::default(),
            executor,
        })
    }

    /// Registers and instantiates a WasmEdge [compiled module](crate::Module) into this [store](crate::Store) as an anonymous active [module instance](crate::Instance), and returns the module instance.
    ///
    /// # Arguments
    ///
    /// * `executor` - The [executor](crate::Executor) that runs the host functions in this [store](crate::Store).
    ///
    /// * `module` - The validated [module](crate::Module) to be registered.
    ///
    /// # Error
    ///
    /// If fail to register the given [module](crate::Module), then an error is returned.
    pub fn register_active_module(&mut self, module: &Module) -> WasmEdgeResult<Instance> {
        let Store {
            inner, executor, ..
        } = self;
        let inner = executor.register_active_module(inner, &module.inner)?;
        Ok(inner)
    }

    pub fn register_named_module(
        &mut self,
        name: impl AsRef<str>,
        module: &Module,
    ) -> WasmEdgeResult<()> {
        let Store {
            inner,
            executor,
            wasm_instance_map,
            ..
        } = self;
        let name = name.as_ref().to_string();
        let inst = executor.register_named_module(inner, &module.inner, &name)?;
        wasm_instance_map.insert(name, inst);
        Ok(())
    }

    /// Returns the number of the named [module instances](crate::Instance) in this [store](crate::Store).
    pub fn named_instance_count(&self) -> usize {
        self.instances.len() + self.wasm_instance_map.len()
    }

    /// Returns the names of all registered named [module instances](crate::Instance).
    pub fn instance_names(&self) -> Vec<String> {
        self.instances
            .keys()
            .chain(self.wasm_instance_map.keys())
            .cloned()
            .collect()
    }

    /// Checks if the [store](crate::Store) contains a named module instance.
    ///
    /// # Argument
    ///
    /// * `mod_name` - The name of the named module.
    ///
    pub fn contains(&self, mod_name: impl AsRef<str>) -> bool {
        let mod_name = mod_name.as_ref().to_string();
        self.instances.contains_key(&mod_name) || self.wasm_instance_map.contains_key(&mod_name)
    }

    pub fn get_instance_and_executor(
        &mut self,
        mod_name: impl AsRef<str>,
    ) -> Option<(&mut T, &mut sys::Executor)> {
        let inst = self
            .instances
            .get_mut(mod_name.as_ref())
            .map(|p| *p as &mut T)?;

        Some((inst, &mut self.executor))
    }

    pub fn get_named_wasm_and_executor(
        &mut self,
        mod_name: impl AsRef<str>,
    ) -> Option<(&mut Instance, &mut sys::Executor)> {
        let wasm_mod = self.wasm_instance_map.get_mut(mod_name.as_ref())?;
        Some((wasm_mod, &mut self.executor))
    }

    pub fn executor(&mut self) -> &mut sys::Executor {
        &mut self.executor
    }
}

// #[cfg(test)]
#[cfg(ignore)]
mod tests {
    use super::*;
    use crate::{
        config::{CommonConfigOptions, ConfigBuilder},
        error::HostFuncError,
        types::Val,
        CallingFrame, Executor, Global, GlobalType, ImportObjectBuilder, Memory, MemoryType,
        Module, Mutability, NeverType, RefType, Statistics, Table, TableType, ValType, WasmValue,
    };

    #[test]
    #[allow(clippy::assertions_on_result_states)]
    fn test_store_create() {
        let result = ConfigBuilder::new(CommonConfigOptions::default()).build();
        assert!(result.is_ok());
        let config = result.unwrap();

        let result = Statistics::new();
        assert!(result.is_ok());
        let mut stat = result.unwrap();

        let result = Executor::new(Some(&config), Some(&mut stat));
        assert!(result.is_ok());

        let result = Store::new();
        assert!(result.is_ok());
        let store = result.unwrap();

        assert_eq!(store.named_instance_count(), 0);
    }

    #[test]
    #[allow(clippy::assertions_on_result_states)]
    fn test_store_register_import_module() {
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

        // create an executor
        let result = ConfigBuilder::new(CommonConfigOptions::default()).build();
        assert!(result.is_ok());
        let config = result.unwrap();

        let result = Statistics::new();
        assert!(result.is_ok());
        let mut stat = result.unwrap();

        let result = Executor::new(Some(&config), Some(&mut stat));
        assert!(result.is_ok());
        let mut executor = result.unwrap();

        // create a store
        let result = Store::new();
        assert!(result.is_ok());
        let mut store = result.unwrap();

        // register an import module into store
        let result = store.register_import_module(&mut executor, &import);
        assert!(result.is_ok());

        assert_eq!(store.named_instance_count(), 1);
        assert_eq!(store.instance_names(), ["extern-module"]);

        // get active module instance
        let result = store.named_instance("extern-module");
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
    #[allow(clippy::assertions_on_result_states)]
    fn test_store_register_named_module() {
        // create an executor
        let result = ConfigBuilder::new(CommonConfigOptions::default()).build();
        assert!(result.is_ok());
        let config = result.unwrap();

        let result = Statistics::new();
        assert!(result.is_ok());
        let mut stat = result.unwrap();

        let result = Executor::new(Some(&config), Some(&mut stat));
        assert!(result.is_ok());
        let mut executor = result.unwrap();

        // create a store
        let result = Store::new();
        assert!(result.is_ok());
        let mut store = result.unwrap();

        // load wasm module
        let file = std::env::current_dir()
            .unwrap()
            .join("examples/wasmedge-sys/data/fibonacci.wat");

        let result = Module::from_file(Some(&config), file);
        assert!(result.is_ok());
        let module = result.unwrap();

        // register a module into store as a named module
        let result = store.register_named_module(&mut executor, "extern-module", &module);
        assert!(result.is_ok());

        assert_eq!(store.named_instance_count(), 1);
        assert_eq!(store.instance_names(), ["extern-module"]);

        // get active module instance
        let result = store.named_instance("extern-module");
        assert!(result.is_ok());
        let instance = result.unwrap();
        assert!(instance.name().is_some());
        assert_eq!(instance.name().unwrap(), "extern-module");
    }

    #[test]
    fn test_store_register_active_module() {
        // create an executor
        let result = ConfigBuilder::new(CommonConfigOptions::default()).build();
        assert!(result.is_ok());
        let config = result.unwrap();

        let result = Statistics::new();
        assert!(result.is_ok());
        let mut stat = result.unwrap();

        let result = Executor::new(Some(&config), Some(&mut stat));
        assert!(result.is_ok());
        let mut executor = result.unwrap();

        // create a store
        let result = Store::new();
        assert!(result.is_ok());
        let mut store = result.unwrap();

        // load wasm module
        let file = std::env::current_dir()
            .unwrap()
            .join("examples/wasmedge-sys/data/fibonacci.wat");

        let result = Module::from_file(Some(&config), file);
        assert!(result.is_ok());
        let module = result.unwrap();

        // register a module into store as active module
        let result = store.register_active_module(&mut executor, &module);
        assert!(result.is_ok());
        let active_instance = result.unwrap();
        assert!(active_instance.name().is_none());
        let result = active_instance.func("fib");
        assert!(result.is_ok());

        assert_eq!(store.named_instance_count(), 0);
        assert_eq!(store.instance_names().len(), 0);
    }

    #[test]
    #[allow(clippy::assertions_on_result_states)]
    fn test_store_basic() {
        // create an executor
        let result = ConfigBuilder::new(CommonConfigOptions::default()).build();
        assert!(result.is_ok());
        let config = result.unwrap();

        let result = Statistics::new();
        assert!(result.is_ok());
        let mut stat = result.unwrap();

        let result = Executor::new(Some(&config), Some(&mut stat));
        assert!(result.is_ok());
        let mut executor = result.unwrap();

        // create a store
        let result = Store::new();
        assert!(result.is_ok());
        let mut store = result.unwrap();

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

        // register a module into store as a named module
        let result = store.register_import_module(&mut executor, &import);
        assert!(result.is_ok());

        // add a wasm module from a file
        let file = std::env::current_dir()
            .unwrap()
            .join("examples/wasmedge-sys/data/fibonacci.wat");
        let result = Module::from_file(Some(&config), file);
        assert!(result.is_ok());
        let module = result.unwrap();

        let result = store.register_named_module(&mut executor, "fib-module", &module);
        assert!(result.is_ok());

        // check the exported instances
        assert_eq!(store.named_instance_count(), 2);
        let mod_names = store.instance_names();
        assert_eq!(mod_names[0], "extern-module");
        assert_eq!(mod_names[1], "fib-module");

        assert_eq!(mod_names[0], "extern-module");
        let result = store.named_instance(&mod_names[0]);
        assert!(result.is_ok());
        let instance = result.unwrap();
        assert!(instance.name().is_some());
        assert_eq!(instance.name().unwrap(), mod_names[0]);

        assert_eq!(mod_names[1], "fib-module");
        let result = store.named_instance(&mod_names[1]);
        assert!(result.is_ok());
        let instance = result.unwrap();
        assert!(instance.name().is_some());
        assert_eq!(instance.name().unwrap(), mod_names[1]);
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
