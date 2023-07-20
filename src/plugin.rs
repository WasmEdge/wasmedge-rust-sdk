//! Defines plugin related structs.

use crate::{
    instance::Instance, io::WasmValTypeList, FuncType, Global, Memory, Table, WasmEdgeResult,
};
use wasmedge_sys::{self as sys, AsImport};
pub mod ffi {
    pub use wasmedge_sys::ffi::{
        WasmEdge_ModuleDescriptor, WasmEdge_ModuleInstanceContext, WasmEdge_PluginDescriptor,
    };
}

use crate::{error::HostFuncError, CallingFrame, WasmValue};

/// Defines the API to manage plugins.
#[derive(Debug)]
pub struct PluginManager {}
impl PluginManager {
    /// Load plugins from the given path.
    ///
    /// * If the path is not given, then the default plugin paths will be used. The default plugin paths are
    ///
    ///     * The environment variable "WASMEDGE_PLUGIN_PATH".
    ///   
    ///     * The `../plugin/` directory related to the WasmEdge installation path.
    ///
    ///     * The `wasmedge/` directory under the library path if the WasmEdge is installed under the "/usr".
    ///
    /// * If the path is given, then
    ///
    ///     * If the path is pointing at a file , then it indicates that a single plugin will be loaded from the file.
    ///
    ///     * If the path is pointing at a directory, then the method will load plugins from the files in the directory.
    ///
    /// # Argument
    ///
    /// * `path` - A path to a plugin file or a directory holding plugin files. If `None`, then the default plugin path will be used.
    pub fn load(path: Option<&std::path::Path>) -> WasmEdgeResult<()> {
        match path {
            Some(p) => sys::plugin::PluginManager::load_plugins(p),
            None => {
                sys::plugin::PluginManager::load_plugins_from_default_paths();
                Ok(())
            }
        }
    }

    /// Returns the count of loaded plugins.
    pub fn count() -> u32 {
        sys::plugin::PluginManager::count()
    }

    /// Returns the names of all loaded plugins.
    pub fn names() -> Vec<String> {
        sys::plugin::PluginManager::names()
    }

    /// Returns the target plugin by its name.
    ///
    /// # Argument
    ///
    /// * `name` - The name of the target plugin.
    pub fn find(name: impl AsRef<str>) -> Option<Plugin> {
        sys::plugin::PluginManager::find(name.as_ref()).map(|p| Plugin { inner: p })
    }

    /// Initializes the `wasmedge_process` plugin module instance with the parameters.
    ///
    /// # Arguments
    ///
    /// * `allowed_cmds` - A white list of commands.
    ///
    /// * `allowed` - Determines if wasmedge_process is allowed to execute all commands on the white list.
    #[cfg(all(
        target_os = "linux",
        feature = "wasmedge_process",
        not(feature = "static")
    ))]
    pub fn init_wasmedge_process(allowed_cmds: Option<Vec<&str>>, allowed: bool) {
        sys::plugin::PluginManager::init_wasmedge_process(allowed_cmds, allowed);
    }
}

/// Represents a loaded plugin. It provides the APIs for accessing the plugin.
#[derive(Debug)]
pub struct Plugin {
    inner: sys::plugin::Plugin,
}
impl Plugin {
    /// Returns the name of this plugin.
    pub fn name(&self) -> String {
        self.inner.name()
    }

    /// Returns the count of the module instances in this plugin.
    pub fn mod_count(&self) -> u32 {
        self.inner.mod_count()
    }

    /// Returns the names of all module instances in this plugin.
    pub fn mod_names(&self) -> Vec<String> {
        self.inner.mod_names()
    }

    /// Returns a module instance that is generated from the module with the given name in this plugin.
    ///
    /// # Argument
    ///
    /// * `name` - The name of the target module.
    pub fn mod_instance(&self, name: impl AsRef<str>) -> Option<Instance> {
        self.inner
            .mod_instance(name.as_ref())
            .map(|i| Instance { inner: i })
    }
}

/// Defines the type of the function that creates a module instance for a plugin.
pub type ModuleInstanceCreateFn = sys::plugin::ModuleInstanceCreateFn;

/// Defines the type of the program options.
pub type ProgramOptionType = sys::plugin::ProgramOptionType;

/// Represents Plugin descriptor for plugins.
#[derive(Debug)]
pub struct PluginDescriptor {
    inner: sys::plugin::PluginDescriptor,
}
impl PluginDescriptor {
    /// Creates a new plugin descriptor.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the plugin.
    ///
    /// * `desc` - The description of the plugin.
    ///
    /// * `version` - The version of the plugin.
    ///
    /// # Error
    ///
    /// If fail to create the plugin descriptor, then an error will be returned.
    pub fn new(
        name: impl AsRef<str>,
        desc: impl AsRef<str>,
        version: PluginVersion,
    ) -> WasmEdgeResult<Self> {
        Ok(Self {
            inner: sys::plugin::PluginDescriptor::create(name, desc, version.inner)?,
        })
    }

    /// Adds a module descriptor to the plugin descriptor.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the module.
    ///
    /// * `desc` - The description of the module.
    ///
    /// * `f` - The function that creates a module instance for the plugin.
    ///
    /// # Error
    ///
    /// If fail to add the module descriptor, then an error will be returned.
    pub fn add_module_descriptor(
        mut self,
        name: impl AsRef<str>,
        desc: impl AsRef<str>,
        f: Option<ModuleInstanceCreateFn>,
    ) -> WasmEdgeResult<Self> {
        self.inner = self.inner.add_module_descriptor(name, desc, f)?;
        Ok(self)
    }

    /// Adds a program option to the plugin descriptor.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the program option.
    ///
    /// * `desc` - The description of the program option.
    ///
    /// * `ty` - The type of the program option.
    ///
    /// # Error
    ///
    /// If fail to add the program option, then an error will be returned.
    pub fn add_program_option(
        mut self,
        name: impl AsRef<str>,
        desc: impl AsRef<str>,
        ty: ProgramOptionType,
    ) -> WasmEdgeResult<Self> {
        self.inner = self.inner.add_program_option(name, desc, ty)?;
        Ok(self)
    }

    /// Returns the raw pointer to the inner `WasmEdge_PluginDescriptor`.
    #[cfg(feature = "ffi")]
    pub fn as_raw_ptr(&self) -> *const sys::ffi::WasmEdge_PluginDescriptor {
        self.inner.as_raw_ptr()
    }
}

/// Defines the version of a plugin.
#[derive(Debug)]
pub struct PluginVersion {
    pub inner: sys::plugin::PluginVersion,
}
impl PluginVersion {
    /// Creates a new plugin version.
    pub fn new(major: u32, minor: u32, patch: u32, build: u32) -> Self {
        Self {
            inner: sys::plugin::PluginVersion::create(major, minor, patch, build),
        }
    }
}

/// Creates a [plugin module](crate::plugin::PluginModule).
///
/// # Example
///
/// [Create a simple math plugin](https://github.com/second-state/wasmedge-rustsdk-examples/tree/main/simple-plugin)
///
#[derive(Debug, Default)]
pub struct PluginModuleBuilder<T: Send + Sync + Clone> {
    funcs: Vec<(String, sys::Function)>,
    globals: Vec<(String, sys::Global)>,
    memories: Vec<(String, sys::Memory)>,
    tables: Vec<(String, sys::Table)>,
    host_data: Option<Box<T>>,
}
impl<T: Send + Sync + Clone> PluginModuleBuilder<T> {
    /// Creates a new [PluginModuleBuilder].
    pub fn new() -> Self {
        Self {
            funcs: Vec::new(),
            globals: Vec::new(),
            memories: Vec::new(),
            tables: Vec::new(),
            host_data: None,
        }
    }

    /// Adds a [host function](crate::Func) to the [ImportObject] to create.
    ///
    /// N.B. that this function can be used in thread-safe scenarios.
    ///
    /// # Arguments
    ///
    /// * `name` - The exported name of the [host function](crate::Func) to add.
    ///
    /// * `real_func` - The native function.
    ///
    /// * `data` - The additional data object to set to this host function context.
    ///
    /// # error
    ///
    /// If fail to create or add the [host function](crate::Func), then an error is returned.
    pub fn with_func<Args, Rets, D>(
        mut self,
        name: impl AsRef<str>,
        real_func: impl Fn(
                CallingFrame,
                Vec<WasmValue>,
                *mut std::os::raw::c_void,
            ) -> Result<Vec<WasmValue>, HostFuncError>
            + Send
            + Sync
            + 'static,
        data: Option<Box<D>>,
    ) -> WasmEdgeResult<Self>
    where
        Args: WasmValTypeList,
        Rets: WasmValTypeList,
    {
        let boxed_func = Box::new(real_func);
        let args = Args::wasm_types();
        let returns = Rets::wasm_types();
        let ty = FuncType::new(Some(args.to_vec()), Some(returns.to_vec()));
        let inner_func = sys::Function::create_sync_func::<D>(&ty.into(), boxed_func, data, 0)?;
        self.funcs.push((name.as_ref().to_owned(), inner_func));
        Ok(self)
    }

    /// Adds an [async host function](crate::Func) to the [PluginModule] to create.
    ///
    /// N.B. that this function can be used in thread-safe scenarios.
    ///
    /// # Arguments
    ///
    /// * `name` - The exported name of the [host function](crate::Func) to add.
    ///
    /// * `real_func` - The native function.
    ///
    /// # error
    ///
    /// If fail to create or add the [host function](crate::Func), then an error is returned.
    #[cfg(all(feature = "async", target_os = "linux"))]
    pub fn with_async_func<Args, Rets, D>(
        mut self,
        name: impl AsRef<str>,
        real_func: impl Fn(
                CallingFrame,
                Vec<WasmValue>,
                *mut std::os::raw::c_void,
            ) -> Box<
                dyn std::future::Future<Output = Result<Vec<WasmValue>, HostFuncError>> + Send,
            > + Send
            + Sync
            + 'static,
        data: Option<Box<D>>,
    ) -> WasmEdgeResult<Self>
    where
        Args: WasmValTypeList,
        Rets: WasmValTypeList,
        D: Send + Sync,
    {
        let args = Args::wasm_types();
        let returns = Rets::wasm_types();
        let ty = FuncType::new(Some(args.to_vec()), Some(returns.to_vec()));
        let inner_func =
            sys::Function::create_async_func(&ty.into(), Box::new(real_func), data, 0)?;
        self.funcs.push((name.as_ref().to_owned(), inner_func));
        Ok(self)
    }

    /// Adds a [global](crate::Global) to the [PluginModule] to create.
    ///
    /// # Arguments
    ///
    /// * `name` - The exported name of the [global](crate::Global) to add.
    ///
    /// * `global` - The wasm [global instance](crate::Global) to add.
    ///
    pub fn with_global(mut self, name: impl AsRef<str>, global: Global) -> Self {
        self.globals.push((name.as_ref().to_owned(), global.inner));
        self
    }

    /// Adds a [memory](crate::Memory) to the [PluginModule] to create.
    ///
    /// # Arguments
    ///
    /// * `name` - The exported name of the [memory](crate::Memory) to add.
    ///
    /// * `memory` - The wasm [memory instance](crate::Memory) to add.
    ///
    pub fn with_memory(mut self, name: impl AsRef<str>, memory: Memory) -> Self {
        self.memories.push((name.as_ref().to_owned(), memory.inner));
        self
    }

    /// Adds a [table](crate::Table) to the [PluginModule] to create.
    ///
    /// # Arguments
    ///
    /// * `name` - The exported name of the [table](crate::Table) to add.
    ///
    /// * `table` - The wasm [table instance](crate::Table) to add.
    ///
    pub fn with_table(mut self, name: impl AsRef<str>, table: Table) -> Self {
        self.tables.push((name.as_ref().to_owned(), table.inner));
        self
    }

    /// Adds host data to the [PluginModule] to create.
    ///
    /// # Arguments
    ///
    /// * `host_data` - The host data to be stored in the module instance.
    ///
    /// * `finalizer` - The function to drop the host data. Notice that this argument is available only if `host_data` is set some value.
    ///
    pub fn with_host_data(mut self, host_data: Box<T>) -> Self {
        self.host_data = Some(host_data);
        self
    }

    /// Creates a new [PluginModule].
    ///
    /// # Argument
    ///
    /// * `name` - The name of the [PluginModule] to create.
    ///
    /// # Error
    ///
    /// If fail to create the [PluginModule], then an error is returned.
    pub fn build(self, name: impl AsRef<str>) -> WasmEdgeResult<PluginModule<T>> {
        let mut inner = sys::plugin::PluginModule::create(name.as_ref(), self.host_data)?;

        // add func
        for (name, func) in self.funcs.into_iter() {
            inner.add_func(name, func);
        }

        // add global
        for (name, global) in self.globals.into_iter() {
            inner.add_global(name, global);
        }

        // add memory
        for (name, memory) in self.memories.into_iter() {
            inner.add_memory(name, memory);
        }

        // add table
        for (name, table) in self.tables.into_iter() {
            inner.add_table(name, table);
        }

        Ok(PluginModule(inner))
    }
}

/// Defines an import object that contains the required import data used when instantiating a [module](crate::Module).
///
/// An [PluginModule] instance is created with [PluginModuleBuilder](crate::plugin::PluginModuleBuilder).
#[derive(Debug, Clone)]
pub struct PluginModule<T: Send + Sync + Clone>(pub(crate) sys::plugin::PluginModule<T>);
impl<T: Send + Sync + Clone> PluginModule<T> {
    /// Returns the name of the plugin module instance.
    pub fn name(&self) -> &str {
        self.0.name()
    }

    /// Returns the raw pointer to the inner `WasmEdge_ModuleInstanceContext`.
    #[cfg(feature = "ffi")]
    pub fn as_raw_ptr(&self) -> *const sys::ffi::WasmEdge_ModuleInstanceContext {
        self.0.as_raw_ptr()
    }
}
