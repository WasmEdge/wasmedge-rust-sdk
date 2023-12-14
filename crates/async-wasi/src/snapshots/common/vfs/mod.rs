use super::{
    error::Errno,
    types::{self as wasi_types},
};
use bitflags::bitflags;
use std::{fmt::Debug, future::Future, io, path::Path, time::Duration};

pub mod impls;
pub mod virtual_sys;

pub enum SystemTimeSpec {
    SymbolicNow,
    Absolute(Duration),
}

pub struct ReaddirEntity {
    pub next: u64,
    pub inode: u64,
    pub name: String,
    pub filetype: FileType,
}

impl From<&ReaddirEntity> for wasi_types::__wasi_dirent_t {
    fn from(ent: &ReaddirEntity) -> Self {
        wasi_types::__wasi_dirent_t {
            d_next: ent.next.to_le(),
            d_ino: ent.inode.to_le(),
            d_namlen: (ent.name.len() as u32).to_le(),
            d_type: ent.filetype.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FdStat {
    pub filetype: FileType,
    pub fs_rights_base: WASIRights,
    pub fs_rights_inheriting: WASIRights,
    pub flags: FdFlags,
}

impl From<&FdStat> for wasi_types::__wasi_fdstat_t {
    fn from(fdstat: &FdStat) -> wasi_types::__wasi_fdstat_t {
        use wasi_types::__wasi_fdstat_t;
        __wasi_fdstat_t {
            fs_filetype: fdstat.filetype.0,
            fs_rights_base: fdstat.fs_rights_base.bits(),
            fs_rights_inheriting: fdstat.fs_rights_inheriting.bits(),
            fs_flags: fdstat.flags.bits(),
        }
    }
}

impl From<FdStat> for wasi_types::__wasi_fdstat_t {
    fn from(fdstat: FdStat) -> wasi_types::__wasi_fdstat_t {
        use wasi_types::__wasi_fdstat_t;
        __wasi_fdstat_t::from(&fdstat)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Filestat {
    pub filetype: FileType,
    pub inode: u64,
    pub nlink: u64,
    pub size: u64, // this is a read field, the rest are file fields
    pub atim: Option<std::time::SystemTime>,
    pub mtim: Option<std::time::SystemTime>,
    pub ctim: Option<std::time::SystemTime>,
}

impl From<Filestat> for wasi_types::__wasi_filestat_t {
    fn from(stat: Filestat) -> wasi_types::__wasi_filestat_t {
        wasi_types::__wasi_filestat_t {
            dev: 3,
            ino: stat.inode,
            filetype: stat.filetype.0,
            nlink: stat.nlink,
            size: stat.size,
            atim: stat
                .atim
                .map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos() as u64)
                .unwrap_or(0),
            mtim: stat
                .mtim
                .map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos() as u64)
                .unwrap_or(0),
            ctim: stat
                .ctim
                .map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos() as u64)
                .unwrap_or(0),
        }
    }
}

impl From<(u64, Filestat)> for wasi_types::__wasi_filestat_t {
    fn from((dev, stat): (u64, Filestat)) -> Self {
        let mut stat: Self = stat.into();
        stat.dev = dev;
        stat
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct FileType(pub wasi_types::__wasi_filetype_t::Type);
impl FileType {
    pub const UNKNOWN: FileType = FileType(0);
    pub const BLOCK_DEVICE: FileType = FileType(1);
    pub const CHARACTER_DEVICE: FileType = FileType(2);
    pub const DIRECTORY: FileType = FileType(3);
    pub const REGULAR_FILE: FileType = FileType(4);
    pub const SOCKET_DGRAM: FileType = FileType(5);
    pub const SOCKET_STREAM: FileType = FileType(6);
    pub const SYMBOLIC_LINK: FileType = FileType(7);
}

bitflags! {
    #[derive(Debug, Clone)]
    pub struct FdFlags: wasi_types::__wasi_fdflags_t::Type {
        const APPEND   = wasi_types::__wasi_fdflags_t::__WASI_FDFLAGS_APPEND; // 0b1
        const DSYNC    = wasi_types::__wasi_fdflags_t::__WASI_FDFLAGS_DSYNC; // 0b10
        const NONBLOCK = wasi_types::__wasi_fdflags_t::__WASI_FDFLAGS_NONBLOCK; // 0b100
        const RSYNC    = wasi_types::__wasi_fdflags_t::__WASI_FDFLAGS_RSYNC; // 0b1000
        const SYNC     = wasi_types::__wasi_fdflags_t::__WASI_FDFLAGS_SYNC; // 0b10000
    }
}

bitflags! {
    #[derive(PartialEq)]
    pub struct SdFlags: wasi_types::__wasi_sdflags_t::Type {
        const RD = wasi_types::__wasi_sdflags_t::__WASI_SDFLAGS_RD;
        const WR = wasi_types::__wasi_sdflags_t::__WASI_SDFLAGS_WR;
    }
}

impl From<SdFlags> for std::net::Shutdown {
    fn from(val: SdFlags) -> Self {
        use std::net::Shutdown;
        if val == SdFlags::RD {
            Shutdown::Read
        } else if val == SdFlags::WR {
            Shutdown::Write
        } else {
            Shutdown::Both
        }
    }
}

bitflags! {
    pub struct SiFlags: wasi_types::__wasi_siflags_t {
    }
}

bitflags! {
    pub struct RiFlags: wasi_types::__wasi_riflags_t::Type {
        const RECV_PEEK    = wasi_types::__wasi_riflags_t::__WASI_RIFLAGS_RECV_PEEK;
        const RECV_WAITALL = wasi_types::__wasi_riflags_t::__WASI_RIFLAGS_RECV_WAITALL;
    }
}

bitflags! {
    pub struct RoFlags: wasi_types::__wasi_roflags_t::Type {
        const RECV_DATA_TRUNCATED = wasi_types::__wasi_roflags_t::__WASI_ROFLAGS_RECV_DATA_TRUNCATED;
    }
}

bitflags! {
    #[derive(Debug, Clone)]
    pub struct OFlags: wasi_types::__wasi_oflags_t::Type {
        const CREATE    = wasi_types::__wasi_oflags_t::__WASI_OFLAGS_CREAT;
        const DIRECTORY = wasi_types::__wasi_oflags_t::__WASI_OFLAGS_DIRECTORY;
        const EXCLUSIVE = wasi_types::__wasi_oflags_t::__WASI_OFLAGS_EXCL;
        const TRUNCATE  = wasi_types::__wasi_oflags_t::__WASI_OFLAGS_TRUNC;
    }
}

bitflags! {
    #[derive(Debug, Clone)]
    pub struct WASIRights : wasi_types::__wasi_rights_t::Type {
        const  FD_DATASYNC= 1;
        const  FD_READ  = 2;
        const  FD_SEEK  = 4;
        const  FD_FDSTAT_SET_FLAGS  = 8;
        const  FD_SYNC  = 16;
        const  FD_TELL  = 32;
        const  FD_WRITE  = 64;
        const  FD_ADVISE  = 128;
        const  FD_ALLOCATE  = 256;
        const  PATH_CREATE_DIRECTORY  = 512;
        const  PATH_CREATE_FILE  = 1024;
        const  PATH_LINK_SOURCE  = 2048;
        const  PATH_LINK_TARGET  = 4096;
        const  PATH_OPEN  = 8192;
        const  FD_READDIR  = 16384;
        const  PATH_READLINK  = 32768;
        const  PATH_RENAME_SOURCE  = 65536;
        const  PATH_RENAME_TARGET  = 131072;
        const  PATH_FILESTAT_GET  = 262144;
        const  PATH_FILESTAT_SET_SIZE  = 524288;
        const  PATH_FILESTAT_SET_TIMES  = 1048576;
        const  FD_FILESTAT_GET  = 2097152;
        const  FD_FILESTAT_SET_SIZE  = 4194304;
        const  FD_FILESTAT_SET_TIMES  = 8388608;
        const  PATH_SYMLINK  = 16777216;
        const  PATH_REMOVE_DIRECTORY  = 33554432;
        const  PATH_UNLINK_FILE  = 67108864;
        const  POLL_FD_READWRITE  = 134217728;
        const  SOCK_SHUTDOWN  = 268435456;
        const  SOCK_OPEN  = 536870912;
        const  SOCK_CLOSE  = 1073741824;
        const  SOCK_BIND  = 2147483648;
        const  SOCK_RECV  = 4294967296;
        const  SOCK_RECV_FROM  = 8589934592;
        const  SOCK_SEND  = 17179869184;
        const  SOCK_SEND_TO  = 34359738368;
    }
}

impl Default for WASIRights {
    fn default() -> Self {
        Self::empty()
    }
}

impl WASIRights {
    #[inline]
    pub fn fd_all() -> Self {
        WASIRights::FD_ADVISE
            | WASIRights::FD_ALLOCATE
            | WASIRights::FD_DATASYNC
            | WASIRights::FD_SYNC
            | WASIRights::FD_TELL
            | WASIRights::FD_SEEK
            | WASIRights::FD_READ
            | WASIRights::FD_WRITE
            | WASIRights::FD_FDSTAT_SET_FLAGS
            | WASIRights::FD_FILESTAT_GET
            | WASIRights::FD_FILESTAT_SET_SIZE
            | WASIRights::FD_FILESTAT_SET_TIMES
    }

    #[inline]
    pub fn dir_all() -> Self {
        WASIRights::PATH_CREATE_DIRECTORY
            | WASIRights::PATH_CREATE_FILE
            | WASIRights::PATH_LINK_SOURCE
            | WASIRights::PATH_LINK_TARGET
            | WASIRights::PATH_OPEN
            | WASIRights::FD_READDIR
            | WASIRights::PATH_READLINK
            | WASIRights::PATH_RENAME_SOURCE
            | WASIRights::PATH_RENAME_TARGET
            | WASIRights::PATH_SYMLINK
            | WASIRights::PATH_REMOVE_DIRECTORY
            | WASIRights::PATH_UNLINK_FILE
            | WASIRights::PATH_FILESTAT_GET
            | WASIRights::PATH_FILESTAT_SET_TIMES
            | WASIRights::FD_FILESTAT_GET
            | WASIRights::FD_FILESTAT_SET_TIMES
    }

    pub fn can(&self, other: Self) -> Result<(), Errno> {
        if self.contains(other) {
            Ok(())
        } else {
            Err(Errno::__WASI_ERRNO_NOTCAPABLE)
        }
    }
}

bitflags! {
    pub struct Lookupflags: wasi_types::__wasi_lookupflags_t::Type {
        const SYMLINK_FOLLOW = wasi_types::__wasi_lookupflags_t::__WASI_LOOKUPFLAGS_SYMLINK_FOLLOW;
    }
}

#[derive(Debug, Clone)]
pub enum Advice {
    Normal,
    Sequential,
    Random,
    WillNeed,
    DontNeed,
    NoReuse,
}

pub trait WasiNode {
    fn fd_fdstat_get(&self) -> Result<FdStat, Errno>;

    fn fd_fdstat_set_flags(&mut self, flags: FdFlags) -> Result<(), Errno> {
        Err(Errno::__WASI_ERRNO_BADF)
    }

    fn fd_fdstat_set_rights(
        &mut self,
        fs_rights_base: WASIRights,
        fs_rights_inheriting: WASIRights,
    ) -> Result<(), Errno> {
        Ok(())
    }

    fn fd_filestat_get(&self) -> Result<Filestat, Errno>;

    fn fd_filestat_set_size(&mut self, size: wasi_types::__wasi_filesize_t) -> Result<(), Errno>;

    fn fd_filestat_set_times(
        &mut self,
        atim: wasi_types::__wasi_timestamp_t,
        mtim: wasi_types::__wasi_timestamp_t,
        fst_flags: wasi_types::__wasi_fstflags_t::Type,
    ) -> Result<(), Errno>;
}

pub trait WasiFile: WasiNode {
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

pub trait WasiDir: WasiNode {
    fn get_readdir(&self, start: u64) -> Result<Vec<(String, u64, FileType)>, Errno>;

    fn fd_readdir(&self, cursor: usize, write_buf: &mut [u8]) -> Result<usize, Errno> {
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

pub trait WasiFileSys {
    type Index: Sized;

    fn path_open(
        &mut self,
        dir_ino: Self::Index,
        path: &str,
        oflags: OFlags,
        fs_rights_base: WASIRights,
        fs_rights_inheriting: WASIRights,
        fdflags: FdFlags,
    ) -> Result<Self::Index, Errno>;
    fn path_rename(
        &mut self,
        old_dir: Self::Index,
        old_path: &str,
        new_dir: Self::Index,
        new_path: &str,
    ) -> Result<(), Errno>;
    fn path_create_directory(&mut self, dir_ino: Self::Index, path: &str) -> Result<(), Errno>;
    fn path_remove_directory(&mut self, dir_ino: Self::Index, path: &str) -> Result<(), Errno>;
    fn path_unlink_file(&mut self, dir_ino: Self::Index, path: &str) -> Result<(), Errno>;
    fn path_link_file(
        &mut self,
        old_dir: Self::Index,
        old_path: &str,
        new_dir: Self::Index,
        new_path: &str,
    ) -> Result<(), Errno>;
    fn path_filestat_get(
        &self,
        dir_ino: Self::Index,
        path: &str,
        follow_symlinks: bool,
    ) -> Result<Filestat, Errno>;

    fn fclose(&mut self, ino: Self::Index) -> Result<(), Errno> {
        Ok(())
    }

    fn get_mut_inode(&mut self, ino: usize) -> Result<&mut dyn WasiNode, Errno>;
    fn get_inode(&self, ino: usize) -> Result<&dyn WasiNode, Errno>;

    fn get_mut_file(&mut self, ino: usize) -> Result<&mut dyn WasiFile, Errno>;
    fn get_file(&self, ino: usize) -> Result<&dyn WasiFile, Errno>;

    fn get_mut_dir(&mut self, ino: usize) -> Result<&mut dyn WasiDir, Errno>;
    fn get_dir(&self, ino: usize) -> Result<&dyn WasiDir, Errno>;
}
