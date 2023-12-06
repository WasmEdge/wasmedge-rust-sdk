pub mod common;
pub mod env;
pub mod preview_1;

use common::error::Errno;

use self::env::{vfs::WasiFileSys, VFS};

#[derive(Debug)]
pub struct WasiCtx {
    pub args: Vec<String>,
    envs: Vec<String>,
    pub(crate) vfs: VFS,
    pub exit_code: u32,
}
impl Default for WasiCtx {
    fn default() -> Self {
        Self::new()
    }
}
impl WasiCtx {
    pub fn new() -> Self {
        WasiCtx {
            args: vec![],
            envs: vec![],
            vfs: VFS::new(),
            exit_code: 0,
        }
    }

    pub fn create_with_vfs(vfs: VFS) -> Self {
        Self {
            args: vec![],
            envs: vec![],
            vfs,
            exit_code: 0,
        }
    }

    pub fn mount_file_sys(
        &mut self,
        guest_path: &str,
        file_sys: Box<dyn WasiFileSys<Index = usize> + Send + Sync>,
    ) {
        self.vfs.mount_file_sys(guest_path, file_sys)
    }

    pub fn push_arg(&mut self, arg: String) {
        self.args.push(arg);
    }

    pub fn push_args(&mut self, args: Vec<String>) {
        self.args.extend(args);
    }

    /// The format of the `env` argument should be "KEY=VALUE"
    pub fn push_env(&mut self, env: String) {
        self.envs.push(env);
    }

    pub fn push_envs(&mut self, envs: Vec<String>) {
        self.envs.extend(envs);
    }
}

// unsafe impl Send for WasiCtx {}
// unsafe impl Sync for WasiCtx {}
