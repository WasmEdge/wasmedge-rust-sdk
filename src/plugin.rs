//! Defines plugin related structs.

use crate::{
    error::HostFuncError, instance::Instance, io::WasmValTypeList, CallingFrame, FuncType, Global,
    Memory, Table, WasmEdgeResult, WasmValue,
};
use wasmedge_sys::{self as sys, AsImport};
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
        let preload: Vec<&str> = preload.split(':').collect();
        let (alias, backend, target, path) = (
            preload[0].to_string(),
            preload[1]
                .parse::<GraphEncoding>()
                .map_err(|err| WasmEdgeError::Operation(err.to_string()))?,
            preload[2]
                .parse::<ExecutionTarget>()
                .map_err(|err| WasmEdgeError::Operation(err.to_string()))?,
            std::path::PathBuf::from(preload[3]),
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
        self.inner
            .mod_instance(name.as_ref())
            .map(|i| Instance { inner: i })
    }
}

pub type PluginInstance = Instance;

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
    #[cfg_attr(docsrs, doc(cfg(feature = "ffi")))]
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
pub struct PluginModuleBuilder<T: ?Sized + Send + Sync + Clone> {
    funcs: Vec<(String, sys::Function)>,
    globals: Vec<(String, sys::Global)>,
    memories: Vec<(String, sys::Memory)>,
    tables: Vec<(String, sys::Table)>,
    host_data: Option<Box<T>>,
}
impl<T: ?Sized + Send + Sync + Clone> PluginModuleBuilder<T> {
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

    /// Adds a [host function](crate::Func) to the [crate::ImportObject] to create.
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
    #[cfg_attr(docsrs, doc(cfg(all(feature = "async", target_os = "linux"))))]
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
    pub fn build(self, name: impl AsRef<str>) -> WasmEdgeResult<PluginModule> {
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
pub struct PluginModule(pub(crate) sys::plugin::PluginModule);
impl PluginModule {
    /// Returns the name of the plugin module instance.
    pub fn name(&self) -> &str {
        self.0.name()
    }

    /// Returns the raw pointer to the inner `WasmEdge_ModuleInstanceContext`.
    #[cfg(feature = "ffi")]
    #[cfg_attr(docsrs, doc(cfg(feature = "ffi")))]
    pub fn as_raw_ptr(&self) -> *const sys::ffi::WasmEdge_ModuleInstanceContext {
        self.0.as_raw_ptr()
    }
}
