pub mod common;
pub mod env;
pub mod preview_1;

use crate::object_pool::ObjectPool;
use common::error::Errno;
use env::{wasi_types::__wasi_fd_t, VFD};
use std::path::PathBuf;

#[derive(Debug)]
pub struct WasiCtx {
    pub args: Vec<String>,
    envs: Vec<String>,
    vfs: ObjectPool<VFD>,
    closed: Option<__wasi_fd_t>,
    vfs_preopen_limit: usize,
    #[cfg(feature = "serialize")]
    pub io_state: serialize::IoState,
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
        let mut vfs = ObjectPool::new();
        vfs.push(wasi_stdin);
        vfs.push(wasi_stdout);
        vfs.push(wasi_stderr);

        WasiCtx {
            args: vec![],
            envs: vec![],
            vfs,
            vfs_preopen_limit: 2,
            closed: None,
            #[cfg(feature = "serialize")]
            io_state: serialize::IoState::Empty,
            exit_code: 0,
        }
    }

    pub fn push_preopen(&mut self, host_path: PathBuf, guest_path: PathBuf) {
        let preopen = env::vfs::WasiPreOpenDir::new(host_path, guest_path);
        self.vfs
            .push(VFD::Inode(env::vfs::INode::PreOpenDir(preopen)));
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
        let i = self.vfs.push(vfd);

        Ok(i.0 as __wasi_fd_t)
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

        let from_entry = self.vfs.remove(from).ok_or(Errno::__WASI_ERRNO_BADF)?;

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
        assert_eq!(fd, 4);

        // [0,1,2,3,4,5]
        let fd = ctx.insert_vfd(vfd_stub()).unwrap();
        assert_eq!(fd, 5);

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

        // [0,1,2,3,none,none,6]
        ctx.remove_vfd(5).unwrap();

        // [0,1,2,3,none,none,none]
        ctx.remove_vfd(6).unwrap();

        let v = ctx
            .vfs
            .iter()
            .take(7)
            .map(|f| f.is_some())
            .collect::<Vec<bool>>();

        assert_eq!(&v, &[true, true, true, true, false, false, false])
    }
}

#[cfg(feature = "serialize")]
pub mod serialize {
    use super::{
        common::{
            net::{
                async_tokio::AsyncWasiSocket, AddressFamily, ConnectState, SocketType,
                WasiSocketState,
            },
            vfs::{self, INode, WASIRights},
        },
        env::vfs::WasiPreOpenDir,
        VFD,
    };
    use crate::object_pool::SerialObjectPool;
    use serde::{Deserialize, Serialize};
    use std::{net::SocketAddr, path::PathBuf, time::SystemTime};

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub enum PollFdState {
        UdpSocket {
            fd: i32,
            socket_type: SerialSocketType,
            local_addr: Option<SocketAddr>,
            peer_addr: Option<SocketAddr>,
            poll_read: bool,
            poll_write: bool,
        },
        TcpListener {
            fd: i32,
            socket_type: SerialSocketType,
            local_addr: Option<SocketAddr>,
            peer_addr: Option<SocketAddr>,
            poll_read: bool,
            poll_write: bool,
        },
        TcpStream {
            fd: i32,
            socket_type: SerialSocketType,
            local_addr: Option<SocketAddr>,
            peer_addr: Option<SocketAddr>,
            poll_read: bool,
            poll_write: bool,
        },
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub enum IoState {
        Empty,
        Accept {
            bind: SocketAddr,
        },
        Sleep {
            ddl: SystemTime,
        },
        Poll {
            fds: Vec<PollFdState>,
            ddl: Option<SystemTime>,
        },
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct SerialWasiCtx {
        pub args: Vec<String>,
        pub envs: Vec<String>,
        pub vfs: SerialObjectPool<SerialVFD>,
        pub vfs_preopen_limit: usize,
        pub io_state: IoState,
        pub exit_code: u32,
    }

    impl From<&super::WasiCtx> for SerialWasiCtx {
        fn from(ctx: &super::WasiCtx) -> Self {
            let vfs = SerialObjectPool::from_ref(&ctx.vfs, |fd| SerialVFD::from(fd));
            Self {
                args: ctx.args.clone(),
                envs: ctx.envs.clone(),
                vfs,
                vfs_preopen_limit: ctx.vfs_preopen_limit,
                io_state: ctx.io_state.clone(),
                exit_code: ctx.exit_code,
            }
        }
    }

    impl SerialWasiCtx {
        pub fn resume(self, f: impl FnMut(SerialVFD) -> VFD) -> super::WasiCtx {
            let Self {
                args,
                envs,
                vfs_preopen_limit,
                io_state,
                exit_code,
                vfs,
            } = self;

            let vfs = vfs.into(f);

            super::WasiCtx {
                args,
                envs,
                vfs,
                vfs_preopen_limit,
                closed: None,
                io_state,
                exit_code,
            }
        }
    }

    #[derive(Debug, Clone, Copy, Deserialize, Serialize)]
    pub enum SerialSocketType {
        TCP4,
        TCP6,
        UDP4,
        UDP6,
    }

    impl From<(AddressFamily, SocketType)> for SerialSocketType {
        fn from(sock_type: (AddressFamily, SocketType)) -> Self {
            match sock_type {
                (AddressFamily::Inet4, SocketType::Datagram) => SerialSocketType::UDP4,
                (AddressFamily::Inet4, SocketType::Stream) => SerialSocketType::TCP4,
                (AddressFamily::Inet6, SocketType::Datagram) => SerialSocketType::UDP6,
                (AddressFamily::Inet6, SocketType::Stream) => SerialSocketType::TCP6,
            }
        }
    }

    impl From<SerialSocketType> for (AddressFamily, SocketType) {
        fn from(val: SerialSocketType) -> Self {
            match val {
                SerialSocketType::TCP4 => (AddressFamily::Inet4, SocketType::Stream),
                SerialSocketType::TCP6 => (AddressFamily::Inet6, SocketType::Stream),
                SerialSocketType::UDP4 => (AddressFamily::Inet4, SocketType::Datagram),
                SerialSocketType::UDP6 => (AddressFamily::Inet6, SocketType::Datagram),
            }
        }
    }

    #[derive(Debug, Clone, Copy, Deserialize, Serialize)]
    pub enum SerialConnectState {
        Empty,
        Listening,
        Connected,
        Connecting,
    }

    impl From<ConnectState> for SerialConnectState {
        fn from(s: ConnectState) -> Self {
            match s {
                ConnectState::Empty => Self::Empty,
                ConnectState::Listening => Self::Listening,
                ConnectState::Connected => Self::Connected,
                ConnectState::Connecting => Self::Connecting,
            }
        }
    }

    impl From<SerialConnectState> for ConnectState {
        fn from(val: SerialConnectState) -> Self {
            match val {
                SerialConnectState::Empty => ConnectState::Empty,
                SerialConnectState::Listening => ConnectState::Listening,
                SerialConnectState::Connected => ConnectState::Connected,
                SerialConnectState::Connecting => ConnectState::Connecting,
            }
        }
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct SerialWasiSocketState {
        pub sock_type: SerialSocketType,
        pub local_addr: Option<SocketAddr>,
        pub peer_addr: Option<SocketAddr>,
        pub bind_device: Vec<u8>,
        pub backlog: u32,
        pub nonblocking: bool,
        pub so_reuseaddr: bool,
        pub so_conn_state: SerialConnectState,
        pub so_recv_buf_size: usize,
        pub so_send_buf_size: usize,
        pub so_recv_timeout: Option<u64>, // nano_sec
        pub so_send_timeout: Option<u64>, // nano_sec,
        pub fs_rights: u64,
    }

    impl From<&WasiSocketState> for SerialWasiSocketState {
        fn from(state: &WasiSocketState) -> Self {
            SerialWasiSocketState {
                sock_type: state.sock_type.into(),
                local_addr: state.local_addr,
                peer_addr: state.peer_addr,
                bind_device: state.bind_device.clone(),
                backlog: state.backlog,
                nonblocking: state.nonblocking,
                so_reuseaddr: state.so_reuseaddr,
                so_conn_state: state.so_conn_state.into(),
                so_recv_buf_size: state.so_recv_buf_size,
                so_send_buf_size: state.so_send_buf_size,
                so_recv_timeout: state.so_recv_timeout.map(|d| d.as_nanos() as u64),
                so_send_timeout: state.so_send_timeout.map(|d| d.as_nanos() as u64),
                fs_rights: state.fs_rights.bits(),
            }
        }
    }

    impl From<SerialWasiSocketState> for WasiSocketState {
        fn from(val: SerialWasiSocketState) -> Self {
            WasiSocketState {
                sock_type: val.sock_type.into(),
                local_addr: val.local_addr,
                peer_addr: val.peer_addr,
                bind_device: val.bind_device,
                backlog: val.backlog,
                shutdown: None,
                nonblocking: val.nonblocking,
                so_reuseaddr: val.so_reuseaddr,
                so_conn_state: val.so_conn_state.into(),
                so_recv_buf_size: val.so_recv_buf_size,
                so_send_buf_size: val.so_send_buf_size,
                so_recv_timeout: val.so_recv_timeout.map(std::time::Duration::from_nanos),
                so_send_timeout: val.so_send_timeout.map(std::time::Duration::from_nanos),
                fs_rights: WASIRights::from_bits_truncate(val.fs_rights),
            }
        }
    }

    impl From<&SerialWasiSocketState> for WasiSocketState {
        fn from(val: &SerialWasiSocketState) -> Self {
            WasiSocketState {
                sock_type: val.sock_type.into(),
                local_addr: val.local_addr,
                peer_addr: val.peer_addr,
                bind_device: val.bind_device.clone(),
                backlog: val.backlog,
                shutdown: None,
                nonblocking: val.nonblocking,
                so_reuseaddr: val.so_reuseaddr,
                so_conn_state: val.so_conn_state.into(),
                so_recv_buf_size: val.so_recv_buf_size,
                so_send_buf_size: val.so_send_buf_size,
                so_recv_timeout: val.so_recv_timeout.map(std::time::Duration::from_nanos),
                so_send_timeout: val.so_send_timeout.map(std::time::Duration::from_nanos),
                fs_rights: WASIRights::from_bits_truncate(val.fs_rights),
            }
        }
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct SerialStdin;
    impl From<SerialStdin> for VFD {
        fn from(_: SerialStdin) -> Self {
            VFD::Inode(INode::Stdin(vfs::WasiStdin))
        }
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct SerialStdout;
    impl From<SerialStdout> for VFD {
        fn from(_: SerialStdout) -> Self {
            VFD::Inode(INode::Stdout(vfs::WasiStdout))
        }
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct SerialStderr;
    impl From<SerialStderr> for VFD {
        fn from(_: SerialStderr) -> Self {
            VFD::Inode(INode::Stderr(vfs::WasiStderr))
        }
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct SerialWasiDir;
    impl From<SerialWasiDir> for VFD {
        fn from(_: SerialWasiDir) -> Self {
            VFD::Closed
        }
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct SerialWasiFile;
    impl From<SerialWasiFile> for VFD {
        fn from(_: SerialWasiFile) -> Self {
            VFD::Closed
        }
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct SerialTcpServer {
        pub state: SerialWasiSocketState,
    }

    impl SerialTcpServer {
        pub fn default_to_async_socket(self) -> std::io::Result<VFD> {
            let state: WasiSocketState = self.state.into();
            let addr = state
                .local_addr
                .ok_or(std::io::Error::from(std::io::ErrorKind::AddrNotAvailable))?;
            let backlog = state.backlog.clamp(128, state.backlog);
            let mut s = AsyncWasiSocket::open(state)?;
            s.bind(addr)?;
            s.listen(backlog)?;
            Ok(VFD::AsyncSocket(s))
        }

        pub fn to_async_socket_with_std(
            self,
            listener: std::net::TcpListener,
        ) -> std::io::Result<VFD> {
            let state: WasiSocketState = self.state.into();
            Ok(VFD::AsyncSocket(AsyncWasiSocket::from_tcplistener(
                listener, state,
            )?))
        }
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct SerialUdpSocket {
        pub state: SerialWasiSocketState,
    }

    impl SerialUdpSocket {
        pub fn default_to_async_socket(self) -> std::io::Result<VFD> {
            let state: WasiSocketState = self.state.into();
            let addr = state
                .local_addr
                .ok_or(std::io::Error::from(std::io::ErrorKind::AddrNotAvailable))?;
            let mut s = AsyncWasiSocket::open(state)?;
            s.bind(addr)?;
            Ok(VFD::AsyncSocket(s))
        }

        pub fn to_async_socket_with_std(self, socket: std::net::UdpSocket) -> std::io::Result<VFD> {
            let state: WasiSocketState = self.state.into();
            Ok(VFD::AsyncSocket(AsyncWasiSocket::from_udpsocket(
                socket, state,
            )?))
        }
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct SerialPreOpen {
        pub guest_path: String,
        pub dir_rights: u64,
        pub file_rights: u64,
    }

    impl SerialPreOpen {
        pub fn to_vfd(self, host_path: PathBuf) -> VFD {
            let mut preopen = WasiPreOpenDir::new(host_path, PathBuf::from(self.guest_path));
            preopen.dir_rights = WASIRights::from_bits_truncate(self.dir_rights);
            preopen.file_rights = WASIRights::from_bits_truncate(self.file_rights);
            VFD::Inode(INode::PreOpenDir(preopen))
        }
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    #[serde(tag = "type")]
    pub enum SerialVFD {
        Stdin(SerialStdin),
        Stdout(SerialStdout),
        Stderr(SerialStderr),
        PreOpenDir(SerialPreOpen),
        WasiDir(SerialWasiDir),
        WasiFile(SerialWasiFile),
        Closed,
        TcpServer(SerialTcpServer),
        UdpSocket(SerialUdpSocket),
    }

    impl From<&VFD> for SerialVFD {
        fn from(vfd: &VFD) -> Self {
            match vfd {
                VFD::Closed => Self::Closed,
                VFD::Inode(INode::Dir(_)) => Self::WasiDir(SerialWasiDir),
                VFD::Inode(INode::File(_)) => Self::WasiFile(SerialWasiFile),
                VFD::Inode(INode::PreOpenDir(pre_open)) => {
                    let guest_path = format!("{}", pre_open.guest_path.display());
                    Self::PreOpenDir(SerialPreOpen {
                        guest_path,
                        dir_rights: pre_open.dir_rights.bits(),
                        file_rights: pre_open.file_rights.bits(),
                    })
                }
                VFD::Inode(INode::Stdin(_)) => Self::Stdin(SerialStdin),
                VFD::Inode(INode::Stdout(_)) => Self::Stdout(SerialStdout),
                VFD::Inode(INode::Stderr(_)) => Self::Stderr(SerialStderr),
                VFD::AsyncSocket(AsyncWasiSocket { inner, state, .. }) => match inner {
                    super::common::net::async_tokio::AsyncWasiSocketInner::PreOpen(_) => {
                        Self::Closed
                    }
                    super::common::net::async_tokio::AsyncWasiSocketInner::AsyncFd(_) => {
                        if state.shutdown.is_some() {
                            Self::Closed
                        } else {
                            let state: SerialWasiSocketState = state.as_ref().into();
                            match state.sock_type {
                                SerialSocketType::TCP4 | SerialSocketType::TCP6 => {
                                    if matches!(state.so_conn_state, SerialConnectState::Listening)
                                    {
                                        Self::TcpServer(SerialTcpServer { state })
                                    } else {
                                        Self::Closed
                                    }
                                }
                                SerialSocketType::UDP4 | SerialSocketType::UDP6 => {
                                    Self::UdpSocket(SerialUdpSocket { state })
                                }
                            }
                        }
                    }
                },
            }
        }
    }

    #[tokio::test]
    async fn test_json_serial() {
        use super::common::net;
        let mut wasi_ctx = super::WasiCtx::new();
        wasi_ctx.push_arg("abc".to_string());
        wasi_ctx.push_env("a=1".to_string());
        wasi_ctx.push_preopen(PathBuf::from("."), PathBuf::from("."));

        // tcp4
        let state = net::WasiSocketState::default();
        let mut s = net::async_tokio::AsyncWasiSocket::open(state).unwrap();
        s.bind("0.0.0.0:1234".parse().unwrap()).unwrap();
        s.listen(128).unwrap();
        wasi_ctx.insert_vfd(VFD::AsyncSocket(s)).unwrap();

        let state = net::WasiSocketState::default();
        let s = net::async_tokio::AsyncWasiSocket::open(state).unwrap();
        wasi_ctx.insert_vfd(VFD::AsyncSocket(s)).unwrap();

        let serial: SerialWasiCtx = (&wasi_ctx).into();

        drop(wasi_ctx);

        let s = serde_json::to_string_pretty(&serial).unwrap();

        println!("{s}");

        let new_wasi_ctx = serial.resume(|vfs| {
            let fd = match vfs {
                SerialVFD::Stdin(s) => s.into(),
                SerialVFD::Stdout(s) => s.into(),
                SerialVFD::Stderr(s) => s.into(),

                SerialVFD::PreOpenDir(dir) => match dir.guest_path.as_str() {
                    "." => dir.clone().to_vfd(PathBuf::from(".")),
                    _ => unreachable!(),
                },
                SerialVFD::TcpServer(s) => s.default_to_async_socket().unwrap(),
                SerialVFD::UdpSocket(s) => s.default_to_async_socket().unwrap(),
                _ => VFD::Closed,
            };
            fd
        });

        assert!(matches!(
            new_wasi_ctx.get_vfd(0),
            Ok(VFD::Inode(INode::Stdin(..)))
        ));
        assert!(matches!(
            new_wasi_ctx.get_vfd(1),
            Ok(VFD::Inode(INode::Stdout(..)))
        ));
        assert!(matches!(
            new_wasi_ctx.get_vfd(2),
            Ok(VFD::Inode(INode::Stderr(..)))
        ));
        assert!(matches!(
            new_wasi_ctx.get_vfd(3),
            Ok(VFD::Inode(INode::PreOpenDir(..)))
        ));
        assert!(matches!(
            new_wasi_ctx.get_vfd(4),
            Ok(VFD::AsyncSocket(AsyncWasiSocket { .. }))
        ));
        assert!(matches!(new_wasi_ctx.get_vfd(5), Err(..)));
        assert!(&new_wasi_ctx.args == &["abc"]);
        assert!(&new_wasi_ctx.envs == &["a=1"]);
        assert_eq!(new_wasi_ctx.vfs_preopen_limit, 3);
    }
}
