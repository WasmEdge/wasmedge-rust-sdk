//! Defines WasmEdge Store struct.

use std::{collections::HashMap, fmt::Debug};

use crate::{config::Config, Module, WasmEdgeResult};
use sys::{AsInstance, Instance};
use wasmedge_sys as sys;

/// The [Store] is a collection of registered modules and assists wasm modules in finding the import modules they need.
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

        for v in instances.values() {
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
