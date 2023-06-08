use super::{error::Errno, types as wasi_types};
use bitflags::bitflags;
use std::{future::Future, time::Duration};

pub mod sync;

pub use sync::*;

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
