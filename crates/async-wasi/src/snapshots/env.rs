pub use super::common::{error::Errno, types as wasi_types, vfs};

#[cfg(all(unix, feature = "async_tokio"))]
pub use super::common::net::async_tokio::AsyncWasiSocket;

#[derive(Debug)]
pub enum VFD {
    Closed,
    Inode(vfs::INode),
    #[cfg(all(unix, feature = "async_tokio"))]
    AsyncSocket(AsyncWasiSocket),
}

impl VFD {
    #[cfg(all(unix, feature = "async_tokio"))]
    pub fn is_socket(&self) -> bool {
        if let VFD::AsyncSocket(_) = self {
            true
        } else {
            false
        }
    }

    pub fn is_inode(&self) -> bool {
        if let VFD::Inode(_) = self {
            true
        } else {
            false
        }
    }
}

pub trait AsyncVM: Send + Sync {
    fn yield_now(&mut self) -> Result<(), Errno>;
}
