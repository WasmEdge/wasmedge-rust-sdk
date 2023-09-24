//! Defines plugin related structs.

use crate::{instance::Instance, WasmEdgeResult};
use wasmedge_sys::{self as sys};

/// Defines low-level types used in Plugin development.
pub mod ffi {
    pub use wasmedge_sys::ffi::{
        WasmEdge_ModuleDescriptor, WasmEdge_ModuleInstanceContext, WasmEdge_PluginDescriptor,
    };
}

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
    ///
    /// # Error
    ///
    /// If failed to return the plugin module instance, then return [PluginError::NotFound](wasmedge_types::error::PluginError::NotFound) error.
    pub fn find(name: impl AsRef<str>) -> WasmEdgeResult<Plugin> {
        sys::plugin::PluginManager::find(name.as_ref()).map(|p| Plugin { inner: p })
    }

    pub fn create_plugin_instance(
        pname: impl AsRef<str>,
        mname: impl AsRef<str>,
    ) -> WasmEdgeResult<PluginInstance> {
        let plugin = sys::plugin::PluginManager::create_plugin_instance(pname, mname)?;
        Ok(plugin)
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
    #[cfg_attr(
        docsrs,
        doc(cfg(all(
            target_os = "linux",
            feature = "wasmedge_process",
            not(feature = "static")
        )))
    )]
    pub fn init_wasmedge_process(allowed_cmds: Option<Vec<&str>>, allowed: bool) {
        sys::plugin::PluginManager::init_wasmedge_process(allowed_cmds, allowed);
    }

    pub fn auto_detect_plugins() -> WasmEdgeResult<Vec<Instance>> {
        let mut plugin_mods = vec![];
        for plugin_name in PluginManager::names().iter() {
            if let Ok(plugin) = PluginManager::find(plugin_name) {
                for mod_name in plugin.mod_names().iter() {
                    if let Ok(mod_instance) = plugin.mod_instance(mod_name) {
                        plugin_mods.push(mod_instance)
                    }
                }
            }
        }
        Ok(plugin_mods)
    }
}

impl PluginManager {
    pub fn load_plugin_wasi_nn() -> WasmEdgeResult<Instance> {
        Self::create_plugin_instance("wasi_nn", "wasi_nn")
    }

    pub fn load_wasi_crypto_common() -> WasmEdgeResult<Instance> {
        Self::create_plugin_instance("wasi_crypto", "wasi_crypto_common")
    }
    pub fn load_wasi_crypto_asymmetric_common() -> WasmEdgeResult<Instance> {
        Self::create_plugin_instance("wasi_crypto", "wasi_crypto_asymmetric_common")
    }
    pub fn load_wasi_crypto_kx() -> WasmEdgeResult<Instance> {
        Self::create_plugin_instance("wasi_crypto", "wasi_crypto_kx")
    }
    pub fn load_wasi_crypto_signatures() -> WasmEdgeResult<Instance> {
        Self::create_plugin_instance("wasi_crypto", "wasi_crypto_signatures")
    }
    pub fn load_wasi_crypto_symmetric() -> WasmEdgeResult<Instance> {
        Self::create_plugin_instance("wasi_crypto", "wasi_crypto_symmetric")
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
    ///
    /// # Error
    ///
    /// If failed to return the plugin module instance, then return [PluginError::Create](wasmedge_types::error::PluginError::Create) error.
    pub fn mod_instance(&self, name: impl AsRef<str>) -> WasmEdgeResult<PluginInstance> {
        self.inner.mod_instance(name.as_ref())
    }
}

pub type PluginInstance = Instance;
