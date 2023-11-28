use super::*;
use std::{
    fmt::Debug,
    fs, io,
    io::{Read, Seek, Write},
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
    time::{Duration, SystemTime},
};

fn systimespec(
    set: bool,
    ts: wasi_types::__wasi_timestamp_t,
    now: bool,
) -> Result<Option<SystemTimeSpec>, Errno> {
    if set && now {
        Err(Errno::__WASI_ERRNO_INVAL)
    } else if set {
        Ok(Some(SystemTimeSpec::Absolute(Duration::from_nanos(ts))))
    } else if now {
        Ok(Some(SystemTimeSpec::SymbolicNow))
    } else {
        Ok(None)
    }
}

pub struct WasiStdin(Box<dyn Read + Send>);
impl Debug for WasiStdin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WasiStdin").finish()
    }
}
impl Default for WasiStdin {
    fn default() -> Self {
        WasiStdin(Box::new(std::io::stdin()))
    }
}
impl From<Box<dyn Read + Send>> for WasiStdin {
    fn from(value: Box<dyn Read + Send>) -> Self {
        Self(value)
    }
}
impl From<WasiStdin> for INode {
    fn from(value: WasiStdin) -> Self {
        INode::Stdin(value)
    }
}
impl WasiVirtualNode for WasiStdin {
    fn fd_fdstat_get(&self) -> Result<FdStat, Errno> {
        Ok(FdStat {
            filetype: FileType::CHARACTER_DEVICE,
            fs_rights_base: WASIRights::FD_READ | WASIRights::POLL_FD_READWRITE,
            fs_rights_inheriting: WASIRights::empty(),
            flags: FdFlags::empty(),
        })
    }

    fn fd_fdstat_set_flags(&mut self, flags: FdFlags) -> Result<(), Errno> {
        Err(Errno::__WASI_ERRNO_BADF)
    }

    fn fd_fdstat_set_rights(
        &mut self,
        fs_rights_base: WASIRights,
        _fs_rights_inheriting: WASIRights,
    ) -> Result<(), Errno> {
        Err(Errno::__WASI_ERRNO_BADF)
    }

    fn fd_filestat_get(&mut self) -> Result<Filestat, Errno> {
        Ok(Filestat {
            filetype: FileType::CHARACTER_DEVICE,
            nlink: 0,
            inode: 0,
            size: 0,
            atim: None,
            mtim: None,
            ctim: None,
        })
    }

    fn fd_filestat_set_size(&mut self, size: wasi_types::__wasi_filesize_t) -> Result<(), Errno> {
        Err(Errno::__WASI_ERRNO_BADF)
    }

    fn fd_filestat_set_times(
        &mut self,
        atim: wasi_types::__wasi_timestamp_t,
        mtim: wasi_types::__wasi_timestamp_t,
        fst_flags: wasi_types::__wasi_fstflags_t::Type,
    ) -> Result<(), Errno> {
        Err(Errno::__WASI_ERRNO_BADF)
    }
}
impl WasiVirtualFile for WasiStdin {
    fn fd_advise(
        &mut self,
        offset: wasi_types::__wasi_filesize_t,
        len: wasi_types::__wasi_filesize_t,
        advice: Advice,
    ) -> Result<(), Errno> {
        Ok(())
    }

    fn fd_allocate(
        &mut self,
        offset: wasi_types::__wasi_filesize_t,
        len: wasi_types::__wasi_filesize_t,
    ) -> Result<(), Errno> {
        Err(Errno::__WASI_ERRNO_BADF)
    }

    fn fd_datasync(&mut self) -> Result<(), Errno> {
        Ok(())
    }

    fn fd_sync(&mut self) -> Result<(), Errno> {
        Ok(())
    }

    fn fd_read(&mut self, bufs: &mut [io::IoSliceMut<'_>]) -> Result<usize, Errno> {
        Ok(self.0.read_vectored(bufs)?)
    }

    fn fd_pread(
        &mut self,
        bufs: &mut [io::IoSliceMut<'_>],
        offset: wasi_types::__wasi_filesize_t,
    ) -> Result<usize, Errno> {
        Err(Errno::__WASI_ERRNO_SPIPE)
    }

    fn fd_write(&mut self, bufs: &[io::IoSlice<'_>]) -> Result<usize, Errno> {
        Err(Errno::__WASI_ERRNO_BADF)
    }

    fn fd_pwrite(
        &mut self,
        bufs: &[io::IoSlice<'_>],
        offset: wasi_types::__wasi_filesize_t,
    ) -> Result<usize, Errno> {
        Err(Errno::__WASI_ERRNO_BADF)
    }

    fn fd_seek(
        &mut self,
        offset: wasi_types::__wasi_filedelta_t,
        whence: wasi_types::__wasi_whence_t::Type,
    ) -> Result<wasi_types::__wasi_filesize_t, Errno> {
        Err(Errno::__WASI_ERRNO_SPIPE)
    }

    fn fd_tell(&mut self) -> Result<wasi_types::__wasi_filesize_t, Errno> {
        Err(Errno::__WASI_ERRNO_SPIPE)
    }
}

pub struct WasiStdout(Box<dyn Write + Send>);
impl Debug for WasiStdout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WasiStdout").finish()
    }
}
impl Default for WasiStdout {
    fn default() -> Self {
        WasiStdout(Box::new(std::io::stdout()))
    }
}
impl From<Box<dyn Write + Send>> for WasiStdout {
    fn from(value: Box<dyn Write + Send>) -> Self {
        Self(value)
    }
}
impl From<WasiStdout> for INode {
    fn from(value: WasiStdout) -> Self {
        INode::Stdout(value)
    }
}
impl WasiVirtualFile for WasiStdout {
    fn fd_advise(
        &mut self,
        offset: wasi_types::__wasi_filesize_t,
        len: wasi_types::__wasi_filesize_t,
        advice: Advice,
    ) -> Result<(), Errno> {
        Ok(())
    }

    fn fd_allocate(
        &mut self,
        offset: wasi_types::__wasi_filesize_t,
        len: wasi_types::__wasi_filesize_t,
    ) -> Result<(), Errno> {
        Err(Errno::__WASI_ERRNO_BADF)
    }

    fn fd_datasync(&mut self) -> Result<(), Errno> {
        self.0.flush()?;
        Ok(())
    }

    fn fd_sync(&mut self) -> Result<(), Errno> {
        self.0.flush()?;
        Ok(())
    }

    fn fd_read(&mut self, bufs: &mut [io::IoSliceMut<'_>]) -> Result<usize, Errno> {
        Err(Errno::__WASI_ERRNO_BADF)
    }

    fn fd_pread(
        &mut self,
        bufs: &mut [io::IoSliceMut<'_>],
        offset: wasi_types::__wasi_filesize_t,
    ) -> Result<usize, Errno> {
        Err(Errno::__WASI_ERRNO_BADF)
    }

    fn fd_write(&mut self, bufs: &[io::IoSlice<'_>]) -> Result<usize, Errno> {
        Ok(self.0.write_vectored(bufs)?)
    }

    fn fd_pwrite(
        &mut self,
        bufs: &[io::IoSlice<'_>],
        offset: wasi_types::__wasi_filesize_t,
    ) -> Result<usize, Errno> {
        Err(Errno::__WASI_ERRNO_SPIPE)
    }

    fn fd_seek(
        &mut self,
        offset: wasi_types::__wasi_filedelta_t,
        whence: wasi_types::__wasi_whence_t::Type,
    ) -> Result<wasi_types::__wasi_filesize_t, Errno> {
        Err(Errno::__WASI_ERRNO_SPIPE)
    }

    fn fd_tell(&mut self) -> Result<wasi_types::__wasi_filesize_t, Errno> {
        Err(Errno::__WASI_ERRNO_SPIPE)
    }
}
impl WasiVirtualNode for WasiStdout {
    fn fd_fdstat_get(&self) -> Result<FdStat, Errno> {
        Ok(FdStat {
            filetype: FileType::CHARACTER_DEVICE,
            fs_rights_base: WASIRights::FD_WRITE | WASIRights::POLL_FD_READWRITE,
            fs_rights_inheriting: WASIRights::empty(),
            flags: FdFlags::APPEND,
        })
    }

    fn fd_fdstat_set_flags(&mut self, flags: FdFlags) -> Result<(), Errno> {
        Err(Errno::__WASI_ERRNO_BADF)
    }

    fn fd_fdstat_set_rights(
        &mut self,
        fs_rights_base: WASIRights,
        _fs_rights_inheriting: WASIRights,
    ) -> Result<(), Errno> {
        Err(Errno::__WASI_ERRNO_BADF)
    }

    fn fd_filestat_get(&mut self) -> Result<Filestat, Errno> {
        Ok(Filestat {
            filetype: FileType::CHARACTER_DEVICE,
            nlink: 0,
            inode: 0,
            size: 0,
            atim: None,
            mtim: None,
            ctim: None,
        })
    }

    fn fd_filestat_set_size(&mut self, size: wasi_types::__wasi_filesize_t) -> Result<(), Errno> {
        Err(Errno::__WASI_ERRNO_BADF)
    }

    fn fd_filestat_set_times(
        &mut self,
        atim: wasi_types::__wasi_timestamp_t,
        mtim: wasi_types::__wasi_timestamp_t,
        fst_flags: wasi_types::__wasi_fstflags_t::Type,
    ) -> Result<(), Errno> {
        Err(Errno::__WASI_ERRNO_BADF)
    }
}

pub struct WasiStderr(Box<dyn Write + Send>);
impl Debug for WasiStderr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WasiStderr").finish()
    }
}
impl Default for WasiStderr {
    fn default() -> Self {
        WasiStderr(Box::new(std::io::stderr()))
    }
}
impl From<Box<dyn Write + Send>> for WasiStderr {
    fn from(value: Box<dyn Write + Send>) -> Self {
        Self(value)
    }
}
impl From<WasiStderr> for INode {
    fn from(value: WasiStderr) -> INode {
        INode::Stderr(value)
    }
}
impl WasiVirtualNode for WasiStderr {
    fn fd_fdstat_get(&self) -> Result<FdStat, Errno> {
        Ok(FdStat {
            filetype: FileType::CHARACTER_DEVICE,
            fs_rights_base: WASIRights::FD_WRITE | WASIRights::POLL_FD_READWRITE,
            fs_rights_inheriting: WASIRights::empty(),
            flags: FdFlags::APPEND,
        })
    }

    fn fd_fdstat_set_flags(&mut self, flags: FdFlags) -> Result<(), Errno> {
        Err(Errno::__WASI_ERRNO_BADF)
    }

    fn fd_fdstat_set_rights(
        &mut self,
        fs_rights_base: WASIRights,
        _fs_rights_inheriting: WASIRights,
    ) -> Result<(), Errno> {
        Err(Errno::__WASI_ERRNO_BADF)
    }

    fn fd_filestat_get(&mut self) -> Result<Filestat, Errno> {
        Ok(Filestat {
            filetype: FileType::CHARACTER_DEVICE,
            nlink: 0,
            inode: 0,
            size: 0,
            atim: None,
            mtim: None,
            ctim: None,
        })
    }

    fn fd_filestat_set_size(&mut self, size: wasi_types::__wasi_filesize_t) -> Result<(), Errno> {
        Err(Errno::__WASI_ERRNO_BADF)
    }

    fn fd_filestat_set_times(
        &mut self,
        atim: wasi_types::__wasi_timestamp_t,
        mtim: wasi_types::__wasi_timestamp_t,
        fst_flags: wasi_types::__wasi_fstflags_t::Type,
    ) -> Result<(), Errno> {
        Err(Errno::__WASI_ERRNO_BADF)
    }
}
impl WasiVirtualFile for WasiStderr {
    fn fd_advise(
        &mut self,
        offset: wasi_types::__wasi_filesize_t,
        len: wasi_types::__wasi_filesize_t,
        advice: Advice,
    ) -> Result<(), Errno> {
        Ok(())
    }

    fn fd_allocate(
        &mut self,
        offset: wasi_types::__wasi_filesize_t,
        len: wasi_types::__wasi_filesize_t,
    ) -> Result<(), Errno> {
        Err(Errno::__WASI_ERRNO_BADF)
    }

    fn fd_datasync(&mut self) -> Result<(), Errno> {
        Ok(())
    }

    fn fd_sync(&mut self) -> Result<(), Errno> {
        Ok(())
    }

    fn fd_read(&mut self, bufs: &mut [io::IoSliceMut<'_>]) -> Result<usize, Errno> {
        Err(Errno::__WASI_ERRNO_BADF)
    }

    fn fd_pread(
        &mut self,
        bufs: &mut [io::IoSliceMut<'_>],
        offset: wasi_types::__wasi_filesize_t,
    ) -> Result<usize, Errno> {
        Err(Errno::__WASI_ERRNO_BADF)
    }

    fn fd_write(&mut self, bufs: &[io::IoSlice<'_>]) -> Result<usize, Errno> {
        Ok(self.0.write_vectored(bufs)?)
    }

    fn fd_pwrite(
        &mut self,
        bufs: &[io::IoSlice<'_>],
        offset: wasi_types::__wasi_filesize_t,
    ) -> Result<usize, Errno> {
        Err(Errno::__WASI_ERRNO_SPIPE)
    }

    fn fd_seek(
        &mut self,
        offset: wasi_types::__wasi_filedelta_t,
        whence: wasi_types::__wasi_whence_t::Type,
    ) -> Result<wasi_types::__wasi_filesize_t, Errno> {
        Err(Errno::__WASI_ERRNO_SPIPE)
    }

    fn fd_tell(&mut self) -> Result<wasi_types::__wasi_filesize_t, Errno> {
        Err(Errno::__WASI_ERRNO_SPIPE)
    }
}

pub trait WasiVirtualNode {
    fn fd_fdstat_get(&self) -> Result<FdStat, Errno>;

    fn fd_fdstat_set_flags(&mut self, flags: FdFlags) -> Result<(), Errno>;

    fn fd_fdstat_set_rights(
        &mut self,
        fs_rights_base: WASIRights,
        fs_rights_inheriting: WASIRights,
    ) -> Result<(), Errno>;

    fn fd_filestat_get(&mut self) -> Result<Filestat, Errno>;

    fn fd_filestat_set_size(&mut self, size: wasi_types::__wasi_filesize_t) -> Result<(), Errno>;

    fn fd_filestat_set_times(
        &mut self,
        atim: wasi_types::__wasi_timestamp_t,
        mtim: wasi_types::__wasi_timestamp_t,
        fst_flags: wasi_types::__wasi_fstflags_t::Type,
    ) -> Result<(), Errno>;
}

pub trait WasiVirtualFile: WasiVirtualNode {
    fn fd_advise(
        &mut self,
        offset: wasi_types::__wasi_filesize_t,
        len: wasi_types::__wasi_filesize_t,
        advice: Advice,
    ) -> Result<(), Errno> {
        Ok(())
    }

    fn fd_allocate(
        &mut self,
        offset: wasi_types::__wasi_filesize_t,
        len: wasi_types::__wasi_filesize_t,
    ) -> Result<(), Errno> {
        Err(Errno::__WASI_ERRNO_BADF)
    }

    fn fd_datasync(&mut self) -> Result<(), Errno> {
        Ok(())
    }

    fn fd_sync(&mut self) -> Result<(), Errno> {
        Ok(())
    }

    fn fd_read(&mut self, bufs: &mut [io::IoSliceMut<'_>]) -> Result<usize, Errno>;

    fn fd_pread(
        &mut self,
        bufs: &mut [io::IoSliceMut<'_>],
        offset: wasi_types::__wasi_filesize_t,
    ) -> Result<usize, Errno>;

    fn fd_write(&mut self, bufs: &[io::IoSlice<'_>]) -> Result<usize, Errno>;

    fn fd_pwrite(
        &mut self,
        bufs: &[io::IoSlice<'_>],
        offset: wasi_types::__wasi_filesize_t,
    ) -> Result<usize, Errno>;

    fn fd_seek(
        &mut self,
        offset: wasi_types::__wasi_filedelta_t,
        whence: wasi_types::__wasi_whence_t::Type,
    ) -> Result<wasi_types::__wasi_filesize_t, Errno>;

    fn fd_tell(&mut self) -> Result<wasi_types::__wasi_filesize_t, Errno>;
}

pub trait WasiVirtualDir: WasiVirtualNode {
    fn get_readdir(&self, start: u64) -> Result<Vec<(String, u64, FileType)>, Errno>;

    fn fd_readdir(&self, cursor: usize, write_buf: &mut [u8]) -> Result<usize, Errno> {
        let buflen = write_buf.len();

        let mut bufused = 0;
        let mut next = cursor as u64;

        for (name, inode, filetype) in self.get_readdir(next)? {
            next += 1;
            let entity = ReaddirEntity {
                next,
                inode,
                name,
                filetype,
            };

            let n = write_dirent(&entity, &mut write_buf[bufused..]);
            bufused += n;
            if bufused == buflen {
                return Ok(bufused);
            }
        }

        Ok(bufused)
    }
}

#[derive(Debug)]
pub struct WasiFile {
    pub fd: fs::File,
    pub flags: FdFlags,
    pub right: WASIRights,
}

impl WasiVirtualNode for WasiFile {
    fn fd_fdstat_get(&self) -> Result<FdStat, Errno> {
        let meta = self.fd.metadata()?;
        let fd_flags = FdStat {
            filetype: if meta.is_symlink() {
                FileType::SYMBOLIC_LINK
            } else {
                FileType::REGULAR_FILE
            },
            fs_rights_base: self.right.clone(),
            fs_rights_inheriting: WASIRights::empty(),
            flags: self.flags.clone(),
        };
        Ok(fd_flags)
    }

    fn fd_fdstat_set_flags(&mut self, flags: FdFlags) -> Result<(), Errno> {
        self.right.can(WASIRights::FD_FDSTAT_SET_FLAGS)?;
        if flags.contains(FdFlags::NONBLOCK)
            && flags.intersects(FdFlags::DSYNC | FdFlags::SYNC | FdFlags::RSYNC)
        {
            return Err(Errno::__WASI_ERRNO_INVAL);
        }
        if flags.contains(FdFlags::APPEND) {
            return Err(Errno::__WASI_ERRNO_NOSYS);
        }
        self.flags = flags;
        Ok(())
    }

    fn fd_fdstat_set_rights(
        &mut self,
        fs_rights_base: WASIRights,
        _fs_rights_inheriting: WASIRights,
    ) -> Result<(), Errno> {
        self.right.can(fs_rights_base.clone())?;
        self.right = fs_rights_base;
        Ok(())
    }

    fn fd_filestat_get(&mut self) -> Result<Filestat, Errno> {
        self.right.can(WASIRights::FD_FILESTAT_GET)?;
        let meta = self.fd.metadata()?;
        let filetype = if meta.is_symlink() {
            FileType::SYMBOLIC_LINK
        } else {
            FileType::REGULAR_FILE
        };

        let nlink = get_file_nlink(&meta);
        let inode = get_file_ino(&meta);

        Ok(Filestat {
            filetype,
            nlink,
            inode,
            size: meta.len(),
            atim: meta.accessed().ok(),
            mtim: meta.modified().ok(),
            ctim: meta.created().ok(),
        })
    }

    fn fd_filestat_set_size(&mut self, size: wasi_types::__wasi_filesize_t) -> Result<(), Errno> {
        self.right.can(WASIRights::FD_FILESTAT_SET_SIZE)?;
        self.fd.set_len(size)?;
        Ok(())
    }

    fn fd_filestat_set_times(
        &mut self,
        atim: wasi_types::__wasi_timestamp_t,
        mtim: wasi_types::__wasi_timestamp_t,
        fst_flags: wasi_types::__wasi_fstflags_t::Type,
    ) -> Result<(), Errno> {
        use wasi_types::__wasi_fstflags_t;

        self.right.can(WASIRights::FD_FILESTAT_SET_TIMES)?;

        let set_atim = (fst_flags & __wasi_fstflags_t::__WASI_FSTFLAGS_ATIM) > 0;
        let set_atim_now = (fst_flags & __wasi_fstflags_t::__WASI_FSTFLAGS_ATIM_NOW) > 0;
        let set_mtim = (fst_flags & __wasi_fstflags_t::__WASI_FSTFLAGS_MTIM) > 0;
        let set_mtim_now = (fst_flags & __wasi_fstflags_t::__WASI_FSTFLAGS_MTIM_NOW) > 0;

        let atim = systimespec(set_atim, atim, set_atim_now)?;
        let mtim = systimespec(set_mtim, mtim, set_mtim_now)?;

        #[cfg(unix)]
        {
            use std::os::unix::prelude::AsRawFd;
            let fd = self.fd.as_raw_fd();
            let times = [
                {
                    match atim {
                        Some(SystemTimeSpec::Absolute(atim)) => libc::timespec {
                            tv_sec: atim.as_secs() as i64,
                            tv_nsec: atim.subsec_nanos() as i64,
                        },
                        Some(SystemTimeSpec::SymbolicNow) => libc::timespec {
                            tv_sec: 0,
                            tv_nsec: libc::UTIME_NOW,
                        },
                        None => libc::timespec {
                            tv_sec: 0,
                            tv_nsec: libc::UTIME_OMIT,
                        },
                    }
                },
                {
                    match mtim {
                        Some(SystemTimeSpec::Absolute(mtim)) => libc::timespec {
                            tv_sec: mtim.as_secs() as i64,
                            tv_nsec: mtim.subsec_nanos() as i64,
                        },
                        Some(SystemTimeSpec::SymbolicNow) => libc::timespec {
                            tv_sec: 0,
                            tv_nsec: libc::UTIME_NOW,
                        },
                        None => libc::timespec {
                            tv_sec: 0,
                            tv_nsec: libc::UTIME_OMIT,
                        },
                    }
                },
            ];
            if unsafe { libc::futimens(fd, times.as_ptr()) } < 0 {
                Err(std::io::Error::last_os_error())?;
            }
            Ok(())
        }
        #[cfg(not(unix))]
        {
            Err(Errno::__WASI_ERRNO_NOSYS)
        }
    }
}

impl WasiVirtualFile for WasiFile {
    fn fd_advise(
        &mut self,
        offset: wasi_types::__wasi_filesize_t,
        len: wasi_types::__wasi_filesize_t,
        advice: Advice,
    ) -> Result<(), Errno> {
        Ok(())
    }

    fn fd_allocate(
        &mut self,
        offset: wasi_types::__wasi_filesize_t,
        len: wasi_types::__wasi_filesize_t,
    ) -> Result<(), Errno> {
        self.right.can(WASIRights::FD_ALLOCATE)?;
        let f = &mut self.fd;
        let metadata = f.metadata()?;
        let file_len = metadata.len();
        let new_len = offset + len;
        if new_len > file_len {
            let old_seek = f.stream_position()?;
            f.set_len(new_len)?;
            f.seek(io::SeekFrom::Start(old_seek))?;
        }
        Ok(())
    }

    fn fd_datasync(&mut self) -> Result<(), Errno> {
        self.right.can(WASIRights::FD_DATASYNC)?;
        self.fd.sync_data()?;
        Ok(())
    }

    fn fd_sync(&mut self) -> Result<(), Errno> {
        self.right.can(WASIRights::FD_SYNC)?;
        self.fd.sync_all()?;
        Ok(())
    }

    fn fd_read(&mut self, bufs: &mut [io::IoSliceMut<'_>]) -> Result<usize, Errno> {
        self.right.can(WASIRights::FD_READ)?;
        Ok(self.fd.read_vectored(bufs)?)
    }

    fn fd_pread(
        &mut self,
        bufs: &mut [io::IoSliceMut<'_>],
        offset: wasi_types::__wasi_filesize_t,
    ) -> Result<usize, Errno> {
        use std::io::SeekFrom;

        self.right.can(WASIRights::FD_READ | WASIRights::FD_SEEK)?;

        let old_seek = self.fd.stream_position()?;
        self.fd.seek(SeekFrom::Start(offset))?;
        let r = self.fd.read_vectored(bufs);
        self.fd.seek(SeekFrom::Start(old_seek))?;
        Ok(r?)
    }

    fn fd_write(&mut self, bufs: &[io::IoSlice<'_>]) -> Result<usize, Errno> {
        self.right.can(WASIRights::FD_WRITE)?;
        Ok(self.fd.write_vectored(bufs)?)
    }

    fn fd_pwrite(
        &mut self,
        bufs: &[io::IoSlice<'_>],
        offset: wasi_types::__wasi_filesize_t,
    ) -> Result<usize, Errno> {
        use std::io::SeekFrom;

        self.right.can(WASIRights::FD_WRITE | WASIRights::FD_SEEK)?;

        let old_seek = self.fd.stream_position()?;
        self.fd.seek(SeekFrom::Start(offset))?;
        let r = self.fd.write_vectored(bufs);
        self.fd.seek(SeekFrom::Start(old_seek))?;
        Ok(r?)
    }

    fn fd_seek(
        &mut self,
        offset: wasi_types::__wasi_filedelta_t,
        whence: wasi_types::__wasi_whence_t::Type,
    ) -> Result<wasi_types::__wasi_filesize_t, Errno> {
        use std::io::SeekFrom;

        let required_rigth =
            if offset == 0 && whence == wasi_types::__wasi_whence_t::__WASI_WHENCE_CUR {
                WASIRights::FD_TELL
            } else {
                WASIRights::FD_TELL | WASIRights::FD_SEEK
            };

        self.right.can(required_rigth)?;

        let pos = match whence {
            wasi_types::__wasi_whence_t::__WASI_WHENCE_CUR => SeekFrom::Current(offset),
            wasi_types::__wasi_whence_t::__WASI_WHENCE_END => SeekFrom::End(offset),
            wasi_types::__wasi_whence_t::__WASI_WHENCE_SET => SeekFrom::Start(offset as u64),
            _ => return Err(Errno::__WASI_ERRNO_INVAL),
        };

        Ok(self.fd.seek(pos)?)
    }

    fn fd_tell(&mut self) -> Result<wasi_types::__wasi_filesize_t, Errno> {
        use std::io::SeekFrom;
        self.right.can(WASIRights::FD_TELL)?;
        Ok(self.fd.stream_position()?)
    }
}

#[derive(Debug)]
pub struct WasiPreOpenDir {
    pub guest_path: PathBuf,
    wasidir: WasiDir,
}

impl Deref for WasiPreOpenDir {
    type Target = WasiDir;
    fn deref(&self) -> &Self::Target {
        &self.wasidir
    }
}

impl DerefMut for WasiPreOpenDir {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.wasidir
    }
}

impl WasiPreOpenDir {
    pub fn new(host_path: PathBuf, guest_path: PathBuf) -> Self {
        WasiPreOpenDir {
            guest_path,
            wasidir: WasiDir {
                real_path: host_path,
                dir_rights: WASIRights::dir_all(),
                file_rights: WASIRights::fd_all(),
            },
        }
    }

    pub fn get_absolutize_path<P: AsRef<Path>>(&self, sub_path: &P) -> Result<PathBuf, Errno> {
        use path_absolutize::*;
        let new_path = self.real_path.join(sub_path);
        let absolutize = new_path
            .absolutize_virtually(&self.real_path)
            .or(Err(Errno::__WASI_ERRNO_NOENT))?;
        Ok(absolutize.to_path_buf())
    }

    pub fn path_open_file<P: AsRef<Path>>(
        &mut self,
        path: &P,
        oflags: OFlags,
        fs_rights_base: WASIRights,
        fdflags: FdFlags,
    ) -> Result<WasiFile, Errno> {
        let mut required_rights = WASIRights::PATH_OPEN;
        if oflags.contains(OFlags::CREATE) {
            required_rights |= WASIRights::PATH_CREATE_FILE;
        }
        self.dir_rights.can(required_rights)?;

        let path = self.get_absolutize_path(path)?;

        let read = fs_rights_base.contains(WASIRights::FD_READ);
        let write = fs_rights_base.contains(WASIRights::FD_WRITE)
            || fs_rights_base.contains(WASIRights::FD_ALLOCATE)
            || fs_rights_base.contains(WASIRights::FD_FILESTAT_SET_SIZE);

        let mut opts = fs::OpenOptions::new();
        if oflags.contains(OFlags::CREATE | OFlags::EXCLUSIVE) {
            opts.create_new(true);
            opts.write(true);
        } else if oflags.contains(OFlags::CREATE) {
            opts.create(true);
            opts.write(true);
        }

        if oflags.contains(OFlags::TRUNCATE) {
            opts.truncate(true);
        }
        if read {
            opts.read(true);
        }

        if write {
            opts.write(true);
        } else {
            opts.read(true);
        }

        if fdflags.contains(FdFlags::APPEND) {
            opts.append(true);
        }

        if fdflags.intersects(FdFlags::DSYNC | FdFlags::SYNC | FdFlags::RSYNC) {
            return Err(Errno::__WASI_ERRNO_NOSYS);
        }

        let fd = opts.open(path)?;

        Ok(WasiFile {
            fd,
            flags: fdflags,
            right: fs_rights_base,
        })
    }

    pub fn path_open_dir<P: AsRef<Path>>(
        &mut self,
        path: &P,
        oflags: OFlags,
        fs_rights_base: WASIRights,
        fs_rights_inheriting: WASIRights,
        fdflags: FdFlags,
    ) -> Result<WasiDir, Errno> {
        let path = self.get_absolutize_path(path)?;

        if oflags.contains(OFlags::CREATE)
            || oflags.contains(OFlags::EXCLUSIVE)
            || oflags.contains(OFlags::TRUNCATE)
        {
            return Err(Errno::__WASI_ERRNO_INVAL);
        }

        let dir_rights = self.dir_rights.clone() & fs_rights_base;
        let file_rights = self.file_rights.clone() & fs_rights_inheriting;
        let meta = fs::metadata(&path)?;
        if !meta.is_dir() {
            return Err(Errno::__WASI_ERRNO_NOTDIR);
        }

        Ok(WasiDir {
            real_path: path,
            dir_rights,
            file_rights,
        })
    }

    pub fn path_create_directory(&self, path: &str) -> Result<(), Errno> {
        self.dir_rights.can(WASIRights::PATH_CREATE_DIRECTORY)?;
        let new_path = self.get_absolutize_path(&path)?;
        fs::DirBuilder::new().recursive(true).create(new_path)?;
        Ok(())
    }

    pub fn path_remove_directory(&self, path: &str) -> Result<(), Errno> {
        self.dir_rights.can(WASIRights::PATH_REMOVE_DIRECTORY)?;
        let new_path = self.get_absolutize_path(&path)?;
        fs::remove_dir(path)?;
        Ok(())
    }

    pub fn path_unlink_file(&self, path: &str) -> Result<(), Errno> {
        self.dir_rights.can(WASIRights::PATH_REMOVE_DIRECTORY)?;
        let new_path = self.get_absolutize_path(&path)?;
        fs::remove_file(new_path)?;
        Ok(())
    }

    pub fn path_filestat_get(&self, path: &str, follow_symlinks: bool) -> Result<Filestat, Errno> {
        self.dir_rights.can(WASIRights::PATH_FILESTAT_GET)?;
        let new_path = self.get_absolutize_path(&path)?;

        let meta = if follow_symlinks {
            fs::metadata(new_path)?
        } else {
            fs::symlink_metadata(new_path)?
        };

        let filetype = if meta.is_symlink() {
            FileType::SYMBOLIC_LINK
        } else if meta.is_dir() {
            FileType::DIRECTORY
        } else {
            FileType::REGULAR_FILE
        };

        let nlink = get_file_nlink(&meta);
        let inode = get_file_ino(&meta);

        Ok(Filestat {
            filetype,
            inode,
            nlink,
            size: meta.len(),
            atim: meta.accessed().ok(),
            mtim: meta.modified().ok(),
            ctim: meta.created().ok(),
        })
    }
}

#[derive(Debug)]
pub struct WasiDir {
    // absolutize
    pub real_path: PathBuf,
    pub dir_rights: WASIRights,
    pub file_rights: WASIRights,
}

fn get_file_ino(metadata: &fs::Metadata) -> u64 {
    #[cfg(unix)]
    {
        use std::os::unix::prelude::MetadataExt;
        metadata.ino()
    }
    #[cfg(not(unix))]
    {
        0
    }
}

fn get_file_nlink(metadata: &fs::Metadata) -> u64 {
    #[cfg(unix)]
    {
        use std::os::unix::prelude::MetadataExt;
        metadata.nlink()
    }
    #[cfg(not(unix))]
    {
        1
    }
}

fn write_dirent(entity: &ReaddirEntity, write_buf: &mut [u8]) -> usize {
    unsafe {
        use wasi_types::__wasi_dirent_t;
        const __wasi_dirent_t_size: usize = std::mem::size_of::<__wasi_dirent_t>();
        let ent = __wasi_dirent_t::from(entity);
        let ent_bytes_ptr = (&ent) as *const __wasi_dirent_t;
        let ent_bytes =
            std::slice::from_raw_parts(ent_bytes_ptr as *const u8, __wasi_dirent_t_size);
        let dirent_copy_len = write_buf.len().min(__wasi_dirent_t_size);
        write_buf[..dirent_copy_len].copy_from_slice(&ent_bytes[..dirent_copy_len]);
        if dirent_copy_len < __wasi_dirent_t_size {
            return dirent_copy_len;
        }

        let name_bytes = entity.name.as_bytes();
        let name_len = name_bytes.len();
        let name_copy_len = (write_buf.len() - dirent_copy_len).min(name_len);
        write_buf[dirent_copy_len..dirent_copy_len + name_copy_len]
            .copy_from_slice(&name_bytes[..name_copy_len]);

        dirent_copy_len + name_copy_len
    }
}

impl WasiVirtualNode for WasiDir {
    fn fd_fdstat_get(&self) -> Result<FdStat, Errno> {
        Ok(FdStat {
            filetype: FileType::DIRECTORY,
            fs_rights_base: self.dir_rights.clone(),
            fs_rights_inheriting: self.file_rights.clone(),
            flags: FdFlags::empty(),
        })
    }

    fn fd_fdstat_set_flags(&mut self, flags: FdFlags) -> Result<(), Errno> {
        Err(Errno::__WASI_ERRNO_BADF)
    }

    fn fd_fdstat_set_rights(
        &mut self,
        fs_rights_base: WASIRights,
        fs_rights_inheriting: WASIRights,
    ) -> Result<(), Errno> {
        self.dir_rights.can(fs_rights_base.clone())?;
        self.file_rights.can(fs_rights_inheriting.clone())?;

        self.dir_rights = fs_rights_base;
        self.file_rights = fs_rights_inheriting;

        Ok(())
    }

    fn fd_filestat_get(&mut self) -> Result<Filestat, Errno> {
        self.dir_rights.can(WASIRights::FD_FILESTAT_GET)?;
        let meta = fs::metadata(&self.real_path)?;
        let filetype = if meta.is_symlink() {
            FileType::SYMBOLIC_LINK
        } else {
            FileType::DIRECTORY
        };

        let nlink = get_file_nlink(&meta);
        let inode = get_file_ino(&meta);

        Ok(Filestat {
            filetype,
            nlink,
            inode,
            size: meta.len(),
            atim: meta.accessed().ok(),
            mtim: meta.modified().ok(),
            ctim: meta.created().ok(),
        })
    }

    fn fd_filestat_set_size(&mut self, size: wasi_types::__wasi_filesize_t) -> Result<(), Errno> {
        Err(Errno::__WASI_ERRNO_BADF)
    }

    fn fd_filestat_set_times(
        &mut self,
        atim: wasi_types::__wasi_timestamp_t,
        mtim: wasi_types::__wasi_timestamp_t,
        fst_flags: wasi_types::__wasi_fstflags_t::Type,
    ) -> Result<(), Errno> {
        use wasi_types::__wasi_fstflags_t;
        self.dir_rights.can(WASIRights::FD_FILESTAT_SET_TIMES)?;
        Err(Errno::__WASI_ERRNO_NOSYS)
    }
}

impl WasiVirtualDir for WasiDir {
    fn get_readdir(&self, mut index: u64) -> Result<Vec<(String, u64, FileType)>, Errno> {
        self.dir_rights.can(WASIRights::FD_READDIR)?;

        let mut dirs = vec![];
        if index == 0 {
            let dir_meta = fs::metadata(&self.real_path)?;
            let dir_ino = get_file_ino(&dir_meta);
            dirs.push((".".to_string(), dir_ino, FileType::DIRECTORY));
            index += 1;
        }

        if index == 1 {
            let dir_ino = if let Some(parent) = self.real_path.parent() {
                let dir_meta = fs::metadata(parent)?;
                get_file_ino(&dir_meta)
            } else {
                0
            };
            dirs.push(("..".to_string(), dir_ino, FileType::DIRECTORY));
            index += 1;
        }

        let read_dir = self.real_path.read_dir()?;

        for dir_entity in read_dir.into_iter().skip((index - 2) as usize) {
            let dir_entity = dir_entity?;
            let name = dir_entity
                .file_name()
                .into_string()
                .map_err(|_| Errno::__WASI_ERRNO_ILSEQ)?;
            let metadata = dir_entity.metadata()?;
            let inode = get_file_ino(&metadata);

            let filetype = if metadata.is_dir() {
                FileType::DIRECTORY
            } else if metadata.is_symlink() {
                FileType::SYMBOLIC_LINK
            } else {
                FileType::REGULAR_FILE
            };

            dirs.push((name, inode, filetype));
        }

        Ok(dirs)
    }
}

#[derive(Debug)]
pub enum INode {
    PreOpenDir(WasiPreOpenDir),
    Dir(WasiDir),
    File(WasiFile),
    Stdin(WasiStdin),
    Stdout(WasiStdout),
    Stderr(WasiStderr),
}
