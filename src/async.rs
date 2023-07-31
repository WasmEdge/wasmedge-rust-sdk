/// The state of an asynchronous task.
pub type AsyncState = wasmedge_sys::r#async::fiber::AsyncState;

/// Represents a wasi module instance.
#[derive(Debug, Clone)]
pub struct WasiInstance(pub(crate) wasmedge_sys::r#async::AsyncWasiModule);
impl WasiInstance {
    pub fn exit_code(&self) -> u32 {
        self.0.exit_code()
    }
}

#[derive(Debug)]
pub struct WasiContext {
    pub(crate) inner: async_wasi::WasiCtx,
}
impl WasiContext {
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

    pub fn exit_code(&self) -> u32 {
        self.inner.exit_code
    }
}
impl Default for WasiContext {
    fn default() -> Self {
        Self::new(None, None, None)
    }
}
