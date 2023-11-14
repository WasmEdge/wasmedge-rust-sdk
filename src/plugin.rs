//! Defines plugin related structs.

use crate::{instance::Instance, WasmEdgeResult};
use wasmedge_sys::{self as sys};

#[cfg(feature = "wasi_nn")]
use wasmedge_types::error::WasmEdgeError;

/// Defines low-level types used in Plugin development.
pub mod ffi {
    pub use wasmedge_sys::ffi::{
        WasmEdge_ModuleDescriptor, WasmEdge_ModuleInstanceContext, WasmEdge_PluginDescriptor,
    };
}

/// Preload config for initializing the wasi_nn plug-in.
#[cfg(feature = "wasi_nn")]
#[cfg_attr(docsrs, doc(cfg(feature = "wasi_nn")))]
#[derive(Debug)]
pub struct NNPreload {
    /// The alias of the model in the WASI-NN environment.
    alias: String,
    /// The inference backend.
    backend: GraphEncoding,
    /// The execution target, on which the inference runs.
    target: ExecutionTarget,
    /// The path to the model file. Note that the path is the guest path instead of the host path.
    path: std::path::PathBuf,
}
#[cfg(feature = "wasi_nn")]
#[cfg_attr(docsrs, doc(cfg(feature = "wasi_nn")))]
impl NNPreload {
    /// Creates a new preload config.
    ///
    /// # Arguments
    ///
    /// * `alias` - The alias of the model in the WASI-NN environment.
    ///
    /// * `backend` - The inference backend.
    ///
    /// * `target` - The execution target, on which the inference runs.
    ///
    /// * `path` - The path to the model file. Note that the path is the guest path instead of the host path.
    ///
    pub fn new(
        alias: impl AsRef<str>,
        backend: GraphEncoding,
        target: ExecutionTarget,
        path: impl AsRef<std::path::Path>,
    ) -> Self {
        Self {
            alias: alias.as_ref().to_owned(),
            backend,
            target,
            path: path.as_ref().to_owned(),
        }
    }
}
#[cfg(feature = "wasi_nn")]
#[cfg_attr(docsrs, doc(cfg(feature = "wasi_nn")))]
impl std::fmt::Display for NNPreload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{alias}:{backend}:{target}:{path}",
            alias = self.alias,
            backend = self.backend,
            target = self.target,
            path = self.path.to_string_lossy().into_owned()
        )
    }
}
#[cfg(feature = "wasi_nn")]
#[cfg_attr(docsrs, doc(cfg(feature = "wasi_nn")))]
impl std::str::FromStr for NNPreload {
    type Err = WasmEdgeError;

    fn from_str(preload: &str) -> std::result::Result<Self, Self::Err> {
        let nn_preload: Vec<&str> = preload.split(':').collect();
        if nn_preload.len() != 4 {
            return Err(WasmEdgeError::Operation(format!(
                "Failed to convert to NNPreload value. Invalid preload string: {}. The correct format is: 'alias:backend:target:path'",
                preload
            )));
        }
        let (alias, backend, target, path) = (
            nn_preload[0].to_string(),
            nn_preload[1]
                .parse::<GraphEncoding>()
                .map_err(|err| WasmEdgeError::Operation(err.to_string()))?,
            nn_preload[2]
                .parse::<ExecutionTarget>()
                .map_err(|err| WasmEdgeError::Operation(err.to_string()))?,
            std::path::PathBuf::from(nn_preload[3]),
        );

        Ok(Self::new(alias, backend, target, path))
    }
}

#[cfg(feature = "wasi_nn")]
#[test]
fn test_generate_nnpreload_from_str() {
    use std::str::FromStr;

    // valid preload string
    let preload = "default:GGML:CPU:llama-2-7b-chat.Q5_K_M.gguf";
    let result = NNPreload::from_str(preload);
    assert!(result.is_ok());
    let nnpreload = result.unwrap();
    assert_eq!(nnpreload.alias, "default");
    assert_eq!(nnpreload.backend, GraphEncoding::GGML);
    assert_eq!(nnpreload.target, ExecutionTarget::CPU);
    assert_eq!(
        nnpreload.path,
        std::path::PathBuf::from("llama-2-7b-chat.Q5_K_M.gguf")
    );

    // invalid preload string
    let preload = "default:CPU:GGML:llama-2-7b-chat.Q5_K_M.gguf";
    let result = NNPreload::from_str(preload);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(
        WasmEdgeError::Operation(
            "Failed to convert to NNBackend value. Unknown NNBackend type: CPU".to_string()
        ),
        err
    );

    // invalid preload string: unsupported target
    let preload = "default:GGML:NPU:llama-2-7b-chat.Q5_K_M.gguf";
    let result = NNPreload::from_str(preload);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(
        WasmEdgeError::Operation(
            "Failed to convert to ExecutionTarget value. Unknown ExecutionTarget type: NPU"
                .to_string()
        ),
        err
    );

    // invalid preload string: invalid format
    let preload = "default:GGML:CPU";
    let result = NNPreload::from_str(preload);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(
        WasmEdgeError::Operation(
            "Failed to convert to NNPreload value. Invalid preload string: default:GGML:CPU. The correct format is: 'alias:backend:target:path'"
                .to_string()
        ),
        err
    );
}

/// Describes the encoding of the graph.
#[cfg(feature = "wasi_nn")]
#[cfg_attr(docsrs, doc(cfg(feature = "wasi_nn")))]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[allow(non_camel_case_types)]
pub enum GraphEncoding {
    OpenVINO,
    ONNX,
    TensorFlow,
    PyTorch,
    TensorFlowLite,
    Autodetect,
    GGML,
}
#[cfg(feature = "wasi_nn")]
#[cfg_attr(docsrs, doc(cfg(feature = "wasi_nn")))]
impl std::fmt::Display for GraphEncoding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GraphEncoding::PyTorch => write!(f, "PyTorch"),
            GraphEncoding::TensorFlowLite => write!(f, "TensorflowLite"),
            GraphEncoding::TensorFlow => write!(f, "Tensorflow"),
            GraphEncoding::OpenVINO => write!(f, "OpenVINO"),
            GraphEncoding::GGML => write!(f, "GGML"),
            GraphEncoding::ONNX => write!(f, "ONNX"),
            GraphEncoding::Autodetect => write!(f, "Autodetect"),
        }
    }
}
#[cfg(feature = "wasi_nn")]
#[cfg_attr(docsrs, doc(cfg(feature = "wasi_nn")))]
impl std::str::FromStr for GraphEncoding {
    type Err = WasmEdgeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "openvino" => Ok(GraphEncoding::OpenVINO),
            "onnx" => Ok(GraphEncoding::ONNX),
            "tensorflow" => Ok(GraphEncoding::TensorFlow),
            "pytorch" => Ok(GraphEncoding::PyTorch),
            "tensorflowlite" => Ok(GraphEncoding::TensorFlowLite),
            "autodetect" => Ok(GraphEncoding::Autodetect),
            "ggml" => Ok(GraphEncoding::GGML),
            _ => Err(WasmEdgeError::Operation(format!(
                "Failed to convert to NNBackend value. Unknown NNBackend type: {}",
                s
            ))),
        }
    }
}

/// Define where the graph should be executed.
#[cfg(feature = "wasi_nn")]
#[cfg_attr(docsrs, doc(cfg(feature = "wasi_nn")))]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[allow(non_camel_case_types)]
pub enum ExecutionTarget {
    CPU,
    GPU,
    TPU,
    AUTO,
}
#[cfg(feature = "wasi_nn")]
#[cfg_attr(docsrs, doc(cfg(feature = "wasi_nn")))]
impl std::fmt::Display for ExecutionTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecutionTarget::CPU => write!(f, "CPU"),
            ExecutionTarget::GPU => write!(f, "GPU"),
            ExecutionTarget::TPU => write!(f, "TPU"),
            ExecutionTarget::AUTO => write!(f, "AUTO"),
        }
    }
}
#[cfg(feature = "wasi_nn")]
#[cfg_attr(docsrs, doc(cfg(feature = "wasi_nn")))]
impl std::str::FromStr for ExecutionTarget {
    type Err = WasmEdgeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "CPU" => Ok(ExecutionTarget::CPU),
            "GPU" => Ok(ExecutionTarget::GPU),
            "TPU" => Ok(ExecutionTarget::TPU),
            "AUTO" => Ok(ExecutionTarget::AUTO),
            _ => Err(WasmEdgeError::Operation(format!(
                "Failed to convert to ExecutionTarget value. Unknown ExecutionTarget type: {}",
                s
            ))),
        }
    }
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

    /// Initialize the wasi_nn plug-in with the preloads.
    ///
    /// Note that this function is only available after loading the wasi_nn plug-in and before creating, and before creating the module instance from the plug-in.
    ///
    /// # Argument
    ///
    /// * `preloads` - The preload list.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // load wasinn-pytorch-plugin from the default plugin directory: /usr/local/lib/wasmedge
    /// PluginManager::load(None)?;
    /// // preload named model
    /// PluginManager::nn_preload(vec![NNPreload::new(
    ///     "default",
    ///     GraphEncoding::GGML,
    ///     ExecutionTarget::CPU,
    ///     "llama-2-7b-chat.Q5_K_M.gguf",
    /// )]);
    /// ```
    ///
    /// If a preload is string, then use `NNPreload::from_str` to create a `NNPreload` instance:
    ///
    /// ```ignore
    /// use std::str::FromStr;
    ///
    /// // load wasinn-pytorch-plugin from the default plugin directory: /usr/local/lib/wasmedge
    /// PluginManager::load(None)?;
    /// // preload named model
    /// PluginManager::nn_preload(vec![NNPreload::from_str("default:GGML:CPU:llama-2-7b-chat.Q5_K_M.gguf")?]);
    ///
    /// ```
    #[cfg(feature = "wasi_nn")]
    #[cfg_attr(docsrs, doc(cfg(feature = "wasi_nn")))]
    pub fn nn_preload(preloads: Vec<NNPreload>) {
        let mut nn_preloads = Vec::new();
        for preload in preloads {
            nn_preloads.push(preload.to_string());
        }

        let nn_preloads_str: Vec<&str> = nn_preloads.iter().map(|s| s.as_str()).collect();

        sys::plugin::PluginManager::nn_preload(nn_preloads_str);
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
