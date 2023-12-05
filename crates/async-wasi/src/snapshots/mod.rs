pub mod common;
pub mod env;
pub mod preview_1;

use common::error::Errno;
use env::{wasi_types::__wasi_fd_t, VFD};
use std::path::PathBuf;

#[derive(Debug)]
pub struct WasiCtx {
    pub args: Vec<String>,
    envs: Vec<String>,
    vfs: slab::Slab<VFD>,
    closed: Option<__wasi_fd_t>,
    vfs_preopen_limit: usize,
    pub exit_code: u32,
}
impl Default for WasiCtx {
    fn default() -> Self {
        Self::new()
    }
}
impl WasiCtx {
    pub fn new() -> Self {
        let wasi_stdin = VFD::Inode(env::vfs::INode::Stdin(env::vfs::WasiStdin::default()));
        let wasi_stdout = VFD::Inode(env::vfs::INode::Stdout(env::vfs::WasiStdout::default()));
        let wasi_stderr = VFD::Inode(env::vfs::INode::Stderr(env::vfs::WasiStderr::default()));
        let mut vfs = slab::Slab::new();
        vfs.insert(wasi_stdin);
        vfs.insert(wasi_stdout);
        vfs.insert(wasi_stderr);

        WasiCtx {
            args: vec![],
            envs: vec![],
            vfs,
            vfs_preopen_limit: 2,
            closed: None,
            exit_code: 0,
        }
    }

    pub fn push_preopen(&mut self, host_path: PathBuf, guest_path: PathBuf) {
        let preopen = env::vfs::WasiPreOpenDir::new(host_path, guest_path);
        self.vfs
            .insert(VFD::Inode(env::vfs::INode::PreOpenDir(preopen)));
        self.vfs_preopen_limit += 1;
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

    fn remove_closed(&mut self) {
        if let Some(closed) = self.closed.take() {
            let _ = self.remove_vfd(closed);
        };
    }

    pub fn get_mut_vfd(&mut self, fd: __wasi_fd_t) -> Result<&mut env::VFD, Errno> {
        if fd < 0 {
            Err(Errno::__WASI_ERRNO_BADF)
        } else {
            self.remove_closed();
            let vfd = self
                .vfs
                .get_mut(fd as usize)
                .ok_or(Errno::__WASI_ERRNO_BADF)?;
            if let VFD::Closed = vfd {
                let _ = self.closed.insert(fd);
                return Err(Errno::__WASI_ERRNO_BADF);
            }
            Ok(vfd)
        }
    }

    pub fn get_vfd(&self, fd: __wasi_fd_t) -> Result<&env::VFD, Errno> {
        if fd < 0 {
            Err(Errno::__WASI_ERRNO_BADF)
        } else {
            let vfd = self.vfs.get(fd as usize).ok_or(Errno::__WASI_ERRNO_BADF)?;
            if let VFD::Closed = vfd {
                return Err(Errno::__WASI_ERRNO_BADF);
            }
            Ok(vfd)
        }
    }

    pub fn insert_vfd(&mut self, vfd: VFD) -> Result<__wasi_fd_t, Errno> {
        let i = self.vfs.insert(vfd);

        Ok(i as __wasi_fd_t)
    }

    pub fn remove_vfd(&mut self, fd: __wasi_fd_t) -> Result<(), Errno> {
        if fd <= self.vfs_preopen_limit as i32 {
            return Err(Errno::__WASI_ERRNO_NOTSUP);
        }

        self.vfs.remove(fd as usize);

        Ok(())
    }

    pub fn renumber_vfd(&mut self, from: __wasi_fd_t, to: __wasi_fd_t) -> Result<(), Errno> {
        if from < 0 || to < 0 {
            return Err(Errno::__WASI_ERRNO_BADF);
        }

        let to = to as usize;
        let from = from as usize;

        if from <= self.vfs_preopen_limit || to <= self.vfs_preopen_limit {
            return Err(Errno::__WASI_ERRNO_NOTSUP);
        };

        let _ = self.vfs.get(to).ok_or(Errno::__WASI_ERRNO_BADF)?;

        let from_entry = self.vfs.try_remove(from).ok_or(Errno::__WASI_ERRNO_BADF)?;

        let to_entry = self.vfs.get_mut(to).ok_or(Errno::__WASI_ERRNO_BADF)?;

        *to_entry = from_entry;

        Ok(())
    }
}

unsafe impl Send for WasiCtx {}
unsafe impl Sync for WasiCtx {}

#[cfg(test)]
mod vfs_test {
    use super::{env::*, *};
    use std::path::PathBuf;

    #[test]
    fn vfd_opt() {
        // [0,1,2]
        let mut ctx = WasiCtx::new();
        // [0,1,2,3(*)]
        ctx.push_preopen(PathBuf::from("."), PathBuf::from("."));

        assert_eq!(ctx.vfs_preopen_limit, 3, "vfs_preopen_limit");

        fn vfd_stub() -> VFD {
            VFD::Inode(vfs::INode::Stdin(vfs::WasiStdin::default()))
        }

        // [0,1,2,3,4]
        let fd = ctx.insert_vfd(vfd_stub()).unwrap();
        assert_eq!(fd, 4);

        // [0,1,2,3,4,5]
        let fd = ctx.insert_vfd(vfd_stub()).unwrap();
        assert_eq!(fd, 5);

        // [0,1,2,3,none,5]
        ctx.remove_vfd(4).unwrap();

        // [0,1,2,3,4,5]
        let fd = ctx.insert_vfd(vfd_stub()).unwrap();
        assert_eq!(fd, 4);

        // [0,1,2,3,none,5]
        ctx.remove_vfd(4).unwrap();

        // [0,1,2,3,none,none]
        ctx.remove_vfd(5).unwrap();

        // [0,1,2,3,4,none]
        let fd = ctx.insert_vfd(vfd_stub()).unwrap();
        assert_eq!(fd, 5);

        // [0,1,2,3,4,5]
        let fd = ctx.insert_vfd(vfd_stub()).unwrap();
        assert_eq!(fd, 4);

        // [0,1,2,3,4,5,6]
        let fd = ctx.insert_vfd(vfd_stub()).unwrap();
        assert_eq!(fd, 6);

        // [0,1,2,3,4,none,6]
        ctx.remove_vfd(5).unwrap();
        // [0,1,2,3,none,none,6]
        ctx.remove_vfd(4).unwrap();

        // [0,1,2,3,4,none,6]
        let fd = ctx.insert_vfd(vfd_stub()).unwrap();
        assert_eq!(fd, 4);

        // [0,1,2,3,4,5,6]
        let fd = ctx.insert_vfd(vfd_stub()).unwrap();
        assert_eq!(fd, 5);

        // [0,1,2,3,none,5,6]
        ctx.remove_vfd(4).unwrap();

        let v = ctx.vfs.iter().map(|f| f.0).collect::<Vec<usize>>();

        assert_eq!(&v, &[0, 1, 2, 3, 5, 6])
    }
}
