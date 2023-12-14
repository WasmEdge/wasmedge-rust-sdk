use std::fmt::Debug;

use self::vfs::{virtual_sys::StdioSys, WasiDir, WasiFile, WasiFileSys, WasiNode};

pub use super::common::{error::Errno, types as wasi_types, vfs};

#[cfg(all(unix, feature = "async_tokio"))]
pub use super::common::net::async_tokio::AsyncWasiSocket;

#[derive(Debug)]
pub enum VFD {
    Inode {
        dev: usize,
        ino: usize,
    },
    #[cfg(all(unix, feature = "async_tokio"))]
    AsyncSocket(AsyncWasiSocket),
}

impl VFD {
    #[cfg(all(unix, feature = "async_tokio"))]
    pub fn is_socket(&self) -> bool {
        matches!(self, VFD::AsyncSocket(_))
    }

    pub fn is_inode(&self) -> bool {
        matches!(self, VFD::Inode { .. })
    }
}

pub trait AsyncVM: Send + Sync {
    fn yield_now(&mut self) -> Result<(), Errno>;
}

pub struct VFS {
    vfs: slab::Slab<Box<dyn WasiFileSys<Index = usize> + Send + Sync>>,
    preopens: Vec<(String, usize)>,
    fds: slab::Slab<VFD>,
}

impl Debug for VFS {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VFS")
            .field("preopens", &self.preopens)
            .field("fds", &self.fds)
            .finish()
    }
}

impl VFS {
    pub fn new_with_stdio<IN, OUT, ERR>(stdio_sys: StdioSys<IN, OUT, ERR>) -> Self
    where
        IN: std::io::Read + Send + Sync + 'static,
        OUT: std::io::Write + Send + Sync + 'static,
        ERR: std::io::Write + Send + Sync + 'static,
    {
        let mut vfs: slab::Slab<Box<dyn WasiFileSys<Index = usize> + Send + Sync>> =
            slab::Slab::new();
        let dev = vfs.insert(Box::new(stdio_sys));

        let mut fds = slab::Slab::with_capacity(3);
        fds.insert(VFD::Inode { dev, ino: 0 });
        fds.insert(VFD::Inode { dev, ino: 1 });
        fds.insert(VFD::Inode { dev, ino: 2 });

        Self {
            vfs,
            preopens: vec![],
            fds,
        }
    }

    pub fn new() -> Self {
        let stdio_sys = StdioSys::new(std::io::stdin(), std::io::stdout(), std::io::stderr());
        let mut vfs: slab::Slab<Box<dyn WasiFileSys<Index = usize> + Send + Sync>> =
            slab::Slab::new();
        let dev = vfs.insert(Box::new(stdio_sys));

        let mut fds = slab::Slab::with_capacity(3);
        fds.insert(VFD::Inode { dev, ino: 0 });
        fds.insert(VFD::Inode { dev, ino: 1 });
        fds.insert(VFD::Inode { dev, ino: 2 });

        Self {
            vfs,
            preopens: vec![],
            fds,
        }
    }

    pub fn mount_file_sys(
        &mut self,
        path: &str,
        file_sys: Box<dyn WasiFileSys<Index = usize> + Send + Sync>,
    ) {
        let vfs_id = self.vfs.insert(file_sys);
        self.preopens.push((path.to_string(), vfs_id));
        self.fds.insert(VFD::Inode {
            dev: vfs_id,
            ino: 0,
        });
    }
}

impl VFS {
    pub fn path_open(
        &mut self,
        dirfd: usize,
        path: &str,
        oflags: vfs::OFlags,
        fs_rights_base: vfs::WASIRights,
        fs_rights_inheriting: vfs::WASIRights,
        fdflags: vfs::FdFlags,
    ) -> Result<usize, Errno> {
        log::trace!("path_open {dirfd} {path}");
        let (dev, ino) = self.get_inode_index(dirfd)?;
        let vfs = self.vfs.get_mut(dev).ok_or(Errno::__WASI_ERRNO_BADF)?;
        let ino = vfs.path_open(
            ino,
            path,
            oflags,
            fs_rights_base,
            fs_rights_inheriting,
            fdflags,
        )?;
        log::trace!("path_open {dirfd} {path} fd=({dev},{ino})");

        if ino != 0 {
            Ok(self.fds.insert(VFD::Inode { dev, ino }))
        } else {
            Ok(dirfd)
        }
    }

    pub fn path_rename(
        &mut self,
        old_dir_fd: usize,
        old_path: &str,
        new_dir_fd: usize,
        new_path: &str,
    ) -> Result<(), Errno> {
        log::trace!(
            "path_rename {:?} {:?}",
            (old_dir_fd, old_path),
            (new_dir_fd, new_path)
        );

        let (dev0, ino0, dev1, ino1) = if old_dir_fd == new_dir_fd {
            if let VFD::Inode { dev, ino } =
                self.fds.get(old_dir_fd).ok_or(Errno::__WASI_ERRNO_BADF)?
            {
                (*dev, *ino, *dev, *ino)
            } else {
                return Err(Errno::__WASI_ERRNO_BADF);
            }
        } else if let (
            VFD::Inode {
                dev: dev0,
                ino: ino0,
            },
            VFD::Inode {
                dev: dev1,
                ino: ino1,
            },
        ) = self
            .fds
            .get2_mut(old_dir_fd, new_dir_fd)
            .ok_or(Errno::__WASI_ERRNO_BADF)?
        {
            (*dev0, *ino0, *dev1, *ino1)
        } else {
            return Err(Errno::__WASI_ERRNO_BADF);
        };

        if dev0 != dev1 {
            return Err(Errno::__WASI_ERRNO_XDEV);
        }

        if ino0 != 0 || ino1 != 0 {
            return Err(Errno::__WASI_ERRNO_ACCES);
        }

        let vfs = self.vfs.get_mut(dev0).ok_or(Errno::__WASI_ERRNO_BADF)?;
        vfs.path_rename(ino0, old_path, ino1, new_path)
    }

    pub fn fd_preopen_get(&mut self, fd: usize) -> Result<String, Errno> {
        log::trace!("fd_preopen_get({fd})");
        if fd < 3 {
            return Err(Errno::__WASI_ERRNO_BADF);
        }
        let fd = fd - 3;
        Ok(self
            .preopens
            .get(fd)
            .ok_or(Errno::__WASI_ERRNO_BADF)?
            .0
            .clone())
    }

    pub fn fd_renumber(&mut self, _from: usize, _to: usize) -> Result<(), Errno> {
        Err(Errno::__WASI_ERRNO_NOTSUP)
    }

    pub fn fd_advise(
        &mut self,
        _fd: usize,
        _offset: wasi_types::__wasi_filesize_t,
        _len: wasi_types::__wasi_filesize_t,
        _advice: wasi_types::__wasi_advice_t::Type,
    ) -> Result<(), Errno> {
        Ok(())
    }

    pub fn fd_close(&mut self, fd: usize) -> Result<(), Errno> {
        match self.fds.get(fd) {
            Some(VFD::Inode { dev, ino }) => {
                if *ino == 0 {
                    return Err(Errno::__WASI_ERRNO_NOTSUP);
                }
                if let Some(vfs) = self.vfs.get_mut(*dev) {
                    log::trace!("fclose fd=({},{})", *dev, *ino);
                    vfs.fclose(*ino)?;
                }
                self.fds.remove(fd);
            }
            Some(VFD::AsyncSocket(_)) => {
                self.fds.remove(fd);
            }
            None => {}
        }

        Ok(())
    }

    pub fn path_filestat_get(
        &self,
        dir_fd: usize,
        path: &str,
        follow_symlinks: bool,
    ) -> Result<(u64, vfs::Filestat), Errno> {
        let (dev, ino) = self.get_inode_index(dir_fd)?;
        let vfs = self.vfs.get(dev).ok_or(Errno::__WASI_ERRNO_BADF)?;
        Ok((
            dev as u64,
            vfs.path_filestat_get(ino, path, follow_symlinks)?,
        ))
    }

    pub fn path_create_directory(&mut self, dir_fd: usize, path: &str) -> Result<(), Errno> {
        let (dev, ino) = self.get_inode_index(dir_fd)?;
        let vfs = self.vfs.get_mut(dev).ok_or(Errno::__WASI_ERRNO_BADF)?;
        vfs.path_create_directory(ino, path)
    }

    pub fn path_remove_directory(&mut self, dir_fd: usize, path: &str) -> Result<(), Errno> {
        let (dev, ino) = self.get_inode_index(dir_fd)?;
        let vfs = self.vfs.get_mut(dev).ok_or(Errno::__WASI_ERRNO_BADF)?;
        vfs.path_remove_directory(ino, path)
    }

    pub fn path_unlink_file(&mut self, dir_fd: usize, path: &str) -> Result<(), Errno> {
        let (dev, ino) = self.get_inode_index(dir_fd)?;
        let vfs = self.vfs.get_mut(dev).ok_or(Errno::__WASI_ERRNO_BADF)?;
        vfs.path_unlink_file(ino, path)
    }

    fn get_inode_index(&self, fd: usize) -> Result<(usize, usize), Errno> {
        if let VFD::Inode { dev, ino } = self.fds.get(fd).ok_or(Errno::__WASI_ERRNO_BADF)? {
            Ok((*dev, *ino))
        } else {
            Err(Errno::__WASI_ERRNO_NOTDIR)
        }
    }

    pub fn get_mut_inode(&mut self, fd: usize) -> Result<&mut dyn WasiNode, Errno> {
        log::trace!("get_mut_inode {fd}");

        let (dev, ino) =
            if let VFD::Inode { dev, ino } = self.fds.get(fd).ok_or(Errno::__WASI_ERRNO_BADF)? {
                (*dev, *ino)
            } else {
                return Err(Errno::__WASI_ERRNO_BADF);
            };
        let vfs = self.vfs.get_mut(dev).ok_or(Errno::__WASI_ERRNO_BADF)?;
        vfs.get_mut_inode(ino)
    }
    pub fn get_inode(&self, fd: usize) -> Result<&dyn WasiNode, Errno> {
        log::trace!("get_inode {fd}");

        let (dev, ino) =
            if let VFD::Inode { dev, ino } = self.fds.get(fd).ok_or(Errno::__WASI_ERRNO_BADF)? {
                (*dev, *ino)
            } else {
                return Err(Errno::__WASI_ERRNO_BADF);
            };
        let vfs = self.vfs.get(dev).ok_or(Errno::__WASI_ERRNO_BADF)?;
        vfs.get_inode(ino)
    }

    pub fn get_mut_file(&mut self, fd: usize) -> Result<&mut dyn WasiFile, Errno> {
        log::trace!("get_mut_file {fd}");

        let (dev, ino) =
            if let VFD::Inode { dev, ino } = self.fds.get(fd).ok_or(Errno::__WASI_ERRNO_BADF)? {
                (*dev, *ino)
            } else {
                return Err(Errno::__WASI_ERRNO_BADF);
            };
        let vfs = self.vfs.get_mut(dev).ok_or(Errno::__WASI_ERRNO_BADF)?;
        vfs.get_mut_file(ino)
    }
    pub fn get_file(&self, fd: usize) -> Result<&dyn WasiFile, Errno> {
        log::trace!("get_file {fd}");

        let (dev, ino) =
            if let VFD::Inode { dev, ino } = self.fds.get(fd).ok_or(Errno::__WASI_ERRNO_BADF)? {
                (*dev, *ino)
            } else {
                return Err(Errno::__WASI_ERRNO_BADF);
            };
        let vfs = self.vfs.get(dev).ok_or(Errno::__WASI_ERRNO_BADF)?;
        vfs.get_file(ino)
    }

    pub fn get_mut_dir(&mut self, fd: usize) -> Result<&mut dyn WasiDir, Errno> {
        log::trace!("get_mut_dir {fd}");

        let (dev, ino) =
            if let VFD::Inode { dev, ino } = self.fds.get(fd).ok_or(Errno::__WASI_ERRNO_BADF)? {
                (*dev, *ino)
            } else {
                return Err(Errno::__WASI_ERRNO_BADF);
            };
        let vfs = self.vfs.get_mut(dev).ok_or(Errno::__WASI_ERRNO_BADF)?;
        vfs.get_mut_dir(ino)
    }
    pub fn get_dir(&self, fd: usize) -> Result<&dyn WasiDir, Errno> {
        log::trace!("get_dir {fd}");

        let (dev, ino) =
            if let VFD::Inode { dev, ino } = self.fds.get(fd).ok_or(Errno::__WASI_ERRNO_BADF)? {
                (*dev, *ino)
            } else {
                return Err(Errno::__WASI_ERRNO_BADF);
            };
        let vfs = self.vfs.get(dev).ok_or(Errno::__WASI_ERRNO_BADF)?;
        vfs.get_dir(ino)
    }

    #[cfg(all(unix, feature = "async_tokio"))]
    pub fn get_mut_socket(&mut self, fd: usize) -> Result<&mut AsyncWasiSocket, Errno> {
        if let VFD::AsyncSocket(s) = self.fds.get_mut(fd).ok_or(Errno::__WASI_ERRNO_BADF)? {
            Ok(s)
        } else {
            Err(Errno::__WASI_ERRNO_NOTSOCK)
        }
    }
    #[cfg(all(unix, feature = "async_tokio"))]
    pub fn get_socket(&self, fd: usize) -> Result<&AsyncWasiSocket, Errno> {
        if let VFD::AsyncSocket(s) = self.fds.get(fd).ok_or(Errno::__WASI_ERRNO_BADF)? {
            Ok(s)
        } else {
            Err(Errno::__WASI_ERRNO_NOTSOCK)
        }
    }
    #[cfg(all(unix, feature = "async_tokio"))]
    pub fn insert_socket(&mut self, s: AsyncWasiSocket) -> Result<usize, Errno> {
        Ok(self.fds.insert(VFD::AsyncSocket(s)))
    }
}
