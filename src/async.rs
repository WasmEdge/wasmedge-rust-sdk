//! Defines the types used in the `async` scenarios.

/// The state of an asynchronous task.
pub type AsyncState = wasmedge_sys::r#async::fiber::AsyncState;

/// Represents a wasi module instance for the `async` scenarios.
#[derive(Debug, Clone)]
pub struct WasiInstance(pub(crate) wasmedge_sys::r#async::AsyncWasiModule);
impl WasiInstance {
    /// Returns the WASI exit code.
    ///
    /// The WASI exit code can be accessed after running the "_start" function of a `wasm32-wasi` program.
    pub fn exit_code(&self) -> u32 {
        self.0.exit_code()
    }
}

/// Represents a wasi context for the `async` scenarios.
#[derive(Debug)]
pub struct WasiContext {
    pub(crate) inner: async_wasi::WasiCtx,
}
impl WasiContext {
    /// Creates a wasi context with the specified argumentes, environment variables, and preopened directories.
    ///
    /// # Arguments
    ///
    /// * `args` - The commandline arguments. The first argument is the program name.
    ///
    /// * `envs` - The environment variables to use.
    ///
    /// * `preopens` - The directories to pre-open. The first element of the pair is the host directory, while the second is the guest directory.
    pub fn new(
        args: Option<Vec<&str>>,
        envs: Option<Vec<(&str, &str)>>,
        preopens: Option<Vec<(&str, &str)>>,
    ) -> Self {
        let mut inner = async_wasi::WasiCtx::new();

        if let Some(args) = args {
            inner.push_args(args.iter().map(|x| x.to_string()).collect());
        }
        if let Some(envs) = envs {
            inner.push_envs(envs.iter().map(|(k, v)| format!("{}={}", k, v)).collect());
        }
        if let Some(preopens) = preopens {
            for (host_dir, guest_dir) in preopens {
                inner.push_preopen(
                    std::path::PathBuf::from(host_dir),
                    std::path::PathBuf::from(guest_dir),
                )
            }
        }

        Self { inner }
    }

    /// Creates a wasi context with the specified argumentes, environment variables, and preopened directories.
    ///
    /// # Arguments
    ///
    /// * `args` - The commandline arguments. The first argument is the program name.
    ///
    /// * `envs` - The environment variables to use.
    ///
    /// * `preopens` - The directories to pre-open. The first element of the pair is the host directory, while the second is the guest directory.
    pub fn generate<S: Into<String>>(
        args: Option<Vec<S>>,
        envs: Option<Vec<(S, S)>>,
        preopens: Option<Vec<(S, S)>>,
    ) -> Self {
        let mut inner = async_wasi::WasiCtx::new();

        if let Some(args) = args {
            inner.push_args(args.into_iter().map(|x| x.into()).collect());
        }
        if let Some(envs) = envs {
            inner.push_envs(
                envs.into_iter()
                    .map(|(k, v)| format!("{}={}", k.into(), v.into()))
                    .collect(),
            );
        }
        if let Some(preopens) = preopens {
            for (host_dir, guest_dir) in preopens {
                inner.push_preopen(
                    std::path::PathBuf::from(host_dir.into()),
                    std::path::PathBuf::from(guest_dir.into()),
                )
            }
        }

        Self { inner }
    }

    /// Returns the WASI exit code.
    ///
    /// The WASI exit code can be accessed after running the "_start" function of a `wasm32-wasi` program.
    pub fn exit_code(&self) -> u32 {
        self.inner.exit_code
    }
}
impl Default for WasiContext {
    fn default() -> Self {
        Self::new(None, None, None)
    }
}
