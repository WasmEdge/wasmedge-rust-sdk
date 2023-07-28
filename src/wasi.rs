//! Defines wasi module instance types, including WasiInstance, WasiNnInstance, wasi-crypto instances.

use crate::WasmEdgeResult;

/// Represents a wasi module instance.
#[derive(Debug, Clone)]
pub struct WasiInstance {
    pub(crate) inner: wasmedge_sys::WasiModule,
}
impl WasiInstance {
    /// Returns the name of this exported [module instance](crate::Instance).
    ///
    /// If this [module instance](crate::Instance) is an active [instance](crate::Instance), return None.
    pub fn name(&self) -> &str {
        self.inner.name()
    }

    /// Initializes the WASI host module with the given parameters.
    ///
    /// # Arguments
    ///
    /// * `args` - The commandline arguments. The first argument is the program name.
    ///
    /// * `envs` - The environment variables in the format `ENV_VAR_NAME=VALUE`.
    ///
    /// * `preopens` - The directories to pre-open. The required format is `DIR1:DIR2`.
    pub fn initialize(
        &mut self,
        args: Option<Vec<&str>>,
        envs: Option<Vec<&str>>,
        preopens: Option<Vec<&str>>,
    ) {
        self.inner.init_wasi(args, envs, preopens);
    }

    /// Returns the WASI exit code.
    ///
    /// The WASI exit code can be accessed after running the "_start" function of a `wasm32-wasi` program.
    pub fn exit_code(&self) -> u32 {
        self.inner.exit_code()
    }

    /// Returns the native handler from the mapped FD/Handler.
    ///
    /// # Argument
    ///
    /// * `fd` - The WASI mapped Fd.
    ///
    /// # Error
    ///
    /// If fail to get the native handler, then an error is returned.
    pub fn native_handler(&self, fd: i32) -> WasmEdgeResult<u64> {
        self.inner.get_native_handler(fd)
    }
}
