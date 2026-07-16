//! Defines WasmEdge Store struct.

use std::{collections::HashMap, fmt::Debug};

use crate::{
    Module, WasmEdgeResult,
    config::Config,
    error::{VmError, WasmEdgeError},
};
use sys::{AsInstance, FuncRef, Instance};
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
            .field("executor", &self.executor)
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
    /// * `executor` - The [executor](sys::Executor) that runs the host functions in this [store](crate::Store).
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
        let mod_name = mod_name.as_ref();
        self.instances.contains_key(mod_name) || self.wasm_instance_map.contains_key(mod_name)
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

    /// Resolves the `(function, executor)` pair needed to run an exported wasm function.
    ///
    /// If `mod_name` is `Some`, the target module (a named instance registered on this
    /// [Store], or a named wasm module instance) is looked up by name. If `mod_name` is
    /// `None`, `active_instance` is used instead (the caller's active module instance, if
    /// any).
    ///
    /// This centralizes the lookup previously duplicated across `Vm::run_func`,
    /// `Vm::run_func_with_timeout`, and their `r#async` equivalents.
    pub(crate) fn resolve_func_and_executor<'a>(
        &'a mut self,
        mod_name: Option<&str>,
        func_name: &str,
        active_instance: Option<&'a mut Instance>,
    ) -> WasmEdgeResult<(FuncRef<&'a mut Instance>, &'a mut sys::Executor)> {
        match mod_name {
            Some(mod_name) => {
                // NB: this deliberately borrows `self.instances`/`self.wasm_instance_map`
                // directly (rather than through the `get_instance_and_executor`/
                // `get_named_wasm_and_executor` helpers) so the borrow checker can see the two
                // branches touch disjoint fields; going through the helper methods here made
                // both branches look like they borrow all of `*self`, which conflicts once the
                // return type ties the borrow to the explicit `'a`.
                if let Some(inst) = self.instances.get_mut(mod_name).map(|p| *p as &mut T) {
                    Ok((inst.get_func_mut(func_name)?, &mut self.executor))
                } else if let Some(wasm_mod) = self.wasm_instance_map.get_mut(mod_name) {
                    Ok((wasm_mod.get_func_mut(func_name)?, &mut self.executor))
                } else {
                    Err(Box::new(WasmEdgeError::Vm(VmError::NotFoundModule(
                        mod_name.into(),
                    ))))
                }
            }
            None => {
                let active_inst = active_instance
                    .ok_or_else(|| Box::new(WasmEdgeError::Vm(VmError::NotFoundActiveModule)))?;

                Ok((active_inst.get_func_mut(func_name)?, &mut self.executor))
            }
        }
    }
}
