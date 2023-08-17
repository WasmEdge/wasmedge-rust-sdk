//! Defines wasi module instance.

/// Represents a wasi module instance.
#[cfg(not(feature = "async"))]
#[derive(Debug, Clone)]
pub struct WasiInstance {
    pub(crate) inner: wasmedge_sys::WasiModule,
}
#[cfg(not(feature = "async"))]
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
    pub fn native_handler(&self, fd: i32) -> crate::WasmEdgeResult<u64> {
        self.inner.get_native_handler(fd)
    }
}

#[cfg(all(feature = "async", target_os = "linux"))]
#[cfg_attr(docsrs, doc(cfg(all(feature = "async", target_os = "linux"))))]
pub mod r#async {

    /// The state of an asynchronous task.
    #[cfg_attr(docsrs, doc(cfg(all(feature = "async", target_os = "linux"))))]
    pub type AsyncState = wasmedge_sys::r#async::fiber::AsyncState;

    /// Represents a wasi module instance for the `async` scenarios.
    #[cfg_attr(docsrs, doc(cfg(all(feature = "async", target_os = "linux"))))]
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
    #[cfg_attr(docsrs, doc(cfg(all(feature = "async", target_os = "linux"))))]
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
}
