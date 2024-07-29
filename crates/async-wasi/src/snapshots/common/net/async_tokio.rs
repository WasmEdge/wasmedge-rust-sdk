use super::*;
use crate::snapshots::{
    common::{types as wasi_types, vfs},
    env::Errno,
};
use socket2::{SockAddr, Socket};
use std::{
    ops::DerefMut,
    os::unix::prelude::{AsRawFd, FromRawFd, RawFd},
    sync::atomic::{AtomicBool, AtomicI8, AtomicU8},
};
use tokio::io::{
    unix::{AsyncFd, AsyncFdReadyGuard, TryIoError},
    AsyncReadExt, AsyncWriteExt, Interest,
};

#[derive(Debug)]
pub(crate) enum AsyncWasiSocketInner {
    PreOpen(Socket),
    AsyncFd(AsyncFd<Socket>),
}

impl AsyncWasiSocketInner {
    fn register(&mut self) -> io::Result<()> {
        unsafe {
            let inner = match self {
                AsyncWasiSocketInner::PreOpen(s) => {
                    let mut inner_socket = std::mem::zeroed();
                    std::mem::swap(s, &mut inner_socket);
                    inner_socket
                }
                AsyncWasiSocketInner::AsyncFd(_) => return Ok(()),
            };
            let mut new_self = Self::AsyncFd(AsyncFd::new(inner)?);
            std::mem::swap(self, &mut new_self);
            std::mem::forget(new_self);
            Ok(())
        }
    }

    fn bind(&mut self, addr: &SockAddr) -> io::Result<()> {
        match self {
            AsyncWasiSocketInner::PreOpen(s) => {
                s.set_reuse_address(true)?;
                s.bind(addr)
            }
            AsyncWasiSocketInner::AsyncFd(_) => Err(io::Error::from_raw_os_error(libc::EINVAL)),
        }
    }

    fn bind_device(&mut self, interface: Option<&[u8]>) -> io::Result<()> {
        match self {
            AsyncWasiSocketInner::PreOpen(s) => s.bind_device(interface),
            AsyncWasiSocketInner::AsyncFd(s) => s.get_ref().bind_device(interface),
        }
    }

    fn device(&self) -> io::Result<Option<Vec<u8>>> {
        match self {
            AsyncWasiSocketInner::PreOpen(s) => s.device(),
            AsyncWasiSocketInner::AsyncFd(s) => s.get_ref().device(),
        }
    }

    fn listen(&mut self, backlog: i32) -> io::Result<()> {
        match self {
            AsyncWasiSocketInner::PreOpen(s) => {
                s.listen(backlog)?;
            }
            AsyncWasiSocketInner::AsyncFd(_) => {
                return Err(io::Error::from_raw_os_error(libc::EINVAL))
            }
        }
        self.register()
    }

    async fn accept(&mut self) -> io::Result<(Socket, SockAddr)> {
        match self {
            AsyncWasiSocketInner::PreOpen(s) => Err(io::Error::from_raw_os_error(libc::EINVAL)),
            AsyncWasiSocketInner::AsyncFd(s) => {
                match tokio::time::timeout(
                    std::time::Duration::from_millis(100),
                    s.async_io(Interest::READABLE, |s| s.accept()),
                )
                .await
                {
                    Ok(r) => r,
                    Err(_) => Err(io::Error::from(io::ErrorKind::WouldBlock)),
                }
            }
        }
    }

    fn connect(&mut self, addr: &SockAddr) -> io::Result<()> {
        let r = match self {
            AsyncWasiSocketInner::PreOpen(s) => s.connect(addr),
            AsyncWasiSocketInner::AsyncFd(_) => {
                return Err(io::Error::from_raw_os_error(libc::EINVAL))
            }
        };

        if let Err(e) = r {
            let errno = Errno::from(&e);
            if errno != Errno::__WASI_ERRNO_INPROGRESS {
                Err(e)
            } else {
                self.register()?;
                Err(io::Error::from_raw_os_error(libc::EINPROGRESS))
            }
        } else {
            self.register()?;
            Ok(())
        }
    }

    fn get_ref(&self) -> io::Result<&Socket> {
        match self {
            AsyncWasiSocketInner::PreOpen(_) => Err(io::Error::from_raw_os_error(libc::ENOTCONN)),
            AsyncWasiSocketInner::AsyncFd(s) => Ok(s.get_ref()),
        }
    }

    fn get_async_socket(&self) -> io::Result<&AsyncFd<Socket>> {
        match self {
            AsyncWasiSocketInner::PreOpen(_) => Err(io::Error::from_raw_os_error(libc::ENOTCONN)),
            AsyncWasiSocketInner::AsyncFd(s) => Ok(s),
        }
    }

    fn mut_async_socket(&mut self) -> io::Result<&mut AsyncFd<Socket>> {
        match self {
            AsyncWasiSocketInner::PreOpen(_) => Err(io::Error::from_raw_os_error(libc::ENOTCONN)),
            AsyncWasiSocketInner::AsyncFd(s) => Ok(s),
        }
    }

    pub(crate) async fn readable(&self) -> io::Result<AsyncFdReadyGuard<Socket>> {
        match self {
            AsyncWasiSocketInner::PreOpen(_) => Err(io::Error::from_raw_os_error(libc::ENOTCONN)),
            AsyncWasiSocketInner::AsyncFd(s) => Ok(s.readable().await?),
        }
    }

    pub(crate) async fn writable(&self) -> io::Result<AsyncFdReadyGuard<Socket>> {
        match self {
            AsyncWasiSocketInner::PreOpen(_) => Err(io::Error::from_raw_os_error(libc::ENOTCONN)),
            AsyncWasiSocketInner::AsyncFd(s) => Ok(s.writable().await?),
        }
    }
}

#[derive(Debug)]
pub(crate) struct SocketWritable(pub(crate) AtomicI8);
impl SocketWritable {
    pub(crate) async fn writable(&self) {
        let b = self.0.fetch_sub(1, std::sync::atomic::Ordering::Acquire);
        tokio::time::timeout(Duration::from_secs(10), SocketWritableFuture(b)).await;
    }

    pub(crate) fn set_writable(&self) {
        self.0.store(5, std::sync::atomic::Ordering::Release)
    }
}
impl Default for SocketWritable {
    fn default() -> Self {
        Self(AtomicI8::new(5))
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct SocketWritableFuture(i8);

impl Future for SocketWritableFuture {
    type Output = ();

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        log::trace!("SocketWritableFuture self.0={}", self.0);
        if self.0 >= 0 {
            std::task::Poll::Ready(())
        } else {
            std::task::Poll::Pending
        }
    }
}

#[derive(Debug)]
pub struct AsyncWasiSocket {
    pub(crate) inner: AsyncWasiSocketInner,
    pub state: Box<WasiSocketState>,
    pub(crate) writable: SocketWritable,
}

impl AsyncWasiSocket {
    pub(crate) async fn readable(&self) -> std::io::Result<()> {
        self.inner.readable().await.map(|x| ())
    }

    pub(crate) async fn writable(&self) -> std::io::Result<()> {
        self.writable.writable().await;
        self.inner.writable().await?;
        Ok(())
    }
}

#[inline]
fn handle_timeout_result<T>(
    result: Result<io::Result<T>, tokio::time::error::Elapsed>,
) -> io::Result<T> {
    if let Ok(r) = result {
        r
    } else {
        Err(io::Error::from_raw_os_error(libc::EWOULDBLOCK))
    }
}

impl AsyncWasiSocket {
    pub fn fd_fdstat_get(&self) -> Result<FdStat, Errno> {
        let mut filetype = match self.state.sock_type.1 {
            SocketType::Datagram => FileType::SOCKET_DGRAM,
            SocketType::Stream => FileType::SOCKET_STREAM,
        };
        let flags = if self.state.nonblocking {
            FdFlags::NONBLOCK
        } else {
            FdFlags::empty()
        };

        Ok(FdStat {
            filetype,
            fs_rights_base: self.state.fs_rights.clone(),
            fs_rights_inheriting: WASIRights::empty(),
            flags,
        })
    }
}

impl AsyncWasiSocket {
    pub fn from_tcplistener(
        listener: std::net::TcpListener,
        state: WasiSocketState,
    ) -> io::Result<Self> {
        let socket = Socket::from(listener);
        socket.set_nonblocking(true)?;
        Ok(Self {
            inner: AsyncWasiSocketInner::AsyncFd(AsyncFd::new(socket)?),
            state: Box::new(state),
            writable: Default::default(),
        })
    }

    pub fn from_udpsocket(socket: std::net::UdpSocket, state: WasiSocketState) -> io::Result<Self> {
        let socket = Socket::from(socket);
        socket.set_nonblocking(true)?;
        Ok(Self {
            inner: AsyncWasiSocketInner::AsyncFd(AsyncFd::new(socket)?),
            state: Box::new(state),
            writable: Default::default(),
        })
    }
}

impl AsyncWasiSocket {
    pub fn open(mut state: WasiSocketState) -> io::Result<Self> {
        use socket2::{Domain, Protocol, Type};
        match state.sock_type.1 {
            SocketType::Stream => {
                state.fs_rights = WASIRights::SOCK_BIND
                    | WASIRights::SOCK_CLOSE
                    | WASIRights::SOCK_RECV
                    | WASIRights::SOCK_SEND
                    | WASIRights::SOCK_SHUTDOWN
                    | WASIRights::POLL_FD_READWRITE;
            }
            SocketType::Datagram => {
                state.fs_rights = WASIRights::SOCK_BIND
                    | WASIRights::SOCK_CLOSE
                    | WASIRights::SOCK_RECV_FROM
                    | WASIRights::SOCK_SEND_TO
                    | WASIRights::SOCK_SHUTDOWN
                    | WASIRights::POLL_FD_READWRITE;
            }
        }
        let inner = match state.sock_type {
            (AddressFamily::Inet4, SocketType::Datagram) => {
                Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?
            }
            (AddressFamily::Inet4, SocketType::Stream) => {
                Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP))?
            }
            (AddressFamily::Inet6, SocketType::Datagram) => {
                Socket::new(Domain::IPV6, Type::DGRAM, Some(Protocol::UDP))?
            }
            (AddressFamily::Inet6, SocketType::Stream) => {
                Socket::new(Domain::IPV6, Type::STREAM, Some(Protocol::TCP))?
            }
        };
        inner.set_nonblocking(true)?;
        if !state.bind_device.is_empty() {
            inner.bind_device(Some(&state.bind_device))?;
        }
        Ok(AsyncWasiSocket {
            inner: AsyncWasiSocketInner::PreOpen(inner),
            state: Box::new(state),
            writable: Default::default(),
        })
    }

    pub fn bind(&mut self, addr: net::SocketAddr) -> io::Result<()> {
        use socket2::SockAddr;
        let sock_addr = SockAddr::from(addr);
        self.inner.bind(&sock_addr)?;
        if let SocketType::Datagram = self.state.sock_type.1 {
            self.inner.register()?;
        }
        self.state.local_addr = Some(addr);
        Ok(())
    }

    pub fn device(&self) -> io::Result<Option<Vec<u8>>> {
        if self.state.bind_device.is_empty() {
            self.inner.device()
        } else {
            Ok(Some(self.state.bind_device.clone()))
        }
    }

    pub fn bind_device(&mut self, interface: Option<&[u8]>) -> io::Result<()> {
        self.inner.bind_device(interface)?;
        self.state.bind_device = match interface {
            Some(interface) => interface.to_vec(),
            None => vec![],
        };
        Ok(())
    }

    pub fn listen(&mut self, backlog: u32) -> io::Result<()> {
        self.inner.listen(backlog as i32)?;
        self.state.backlog = backlog;
        self.state.so_conn_state = ConnectState::Listening;
        Ok(())
    }

    pub async fn accept(&mut self) -> io::Result<Self> {
        let mut new_state = WasiSocketState {
            nonblocking: self.state.nonblocking,
            so_conn_state: ConnectState::Connected,
            ..Default::default()
        };

        log::trace!("accept nonblocking={}", self.state.nonblocking);

        let (cs, _) = if self.state.nonblocking {
            let s = self
                .inner
                .get_async_socket()?
                .async_io(Interest::READABLE, |s| s.accept());
            tokio::time::timeout(std::time::Duration::from_millis(50), s)
                .await
                .map_err(|_| io::Error::from(io::ErrorKind::WouldBlock))?
        } else {
            self.inner
                .get_async_socket()?
                .async_io(Interest::READABLE, |s| s.accept())
                .await
        }?;

        cs.set_nonblocking(true)?;
        new_state.peer_addr = cs.peer_addr().ok().and_then(|addr| addr.as_socket());
        new_state.local_addr = cs.local_addr().ok().and_then(|addr| addr.as_socket());

        Ok(AsyncWasiSocket {
            inner: AsyncWasiSocketInner::AsyncFd(AsyncFd::new(cs)?),
            state: Box::new(new_state),
            writable: Default::default(),
        })
    }

    pub async fn connect(&mut self, addr: net::SocketAddr) -> io::Result<()> {
        let address = SockAddr::from(addr);
        self.state.so_conn_state = ConnectState::Connected;
        self.state.peer_addr = Some(addr);

        match (self.state.nonblocking, self.state.so_send_timeout) {
            (true, None) => {
                let r = self.inner.connect(&address);
                if r.is_err() {
                    self.state.so_conn_state = ConnectState::Connecting;
                }
                r?;
                Ok(())
            }
            (false, None) => {
                if let Err(e) = self.inner.connect(&address) {
                    match e.raw_os_error() {
                        Some(libc::EINPROGRESS) => {}
                        _ => return Err(e),
                    }
                    let s = self.inner.writable().await?;
                    let e = s.get_inner().take_error()?;
                    if let Some(e) = e {
                        return Err(e);
                    }
                }
                Ok(())
            }
            (_, Some(timeout)) => {
                if let Err(e) = self.inner.connect(&address) {
                    match e.raw_os_error() {
                        Some(libc::EINPROGRESS) => {}
                        _ => return Err(e),
                    }
                    match tokio::time::timeout(timeout, self.inner.writable()).await {
                        Ok(r) => {
                            let s = r?;
                            let e = s.get_inner().take_error()?;
                            if let Some(e) = e {
                                return Err(e);
                            }
                            Ok(())
                        }
                        Err(e) => Err(io::Error::from_raw_os_error(libc::EWOULDBLOCK)),
                    }
                } else {
                    Ok(())
                }
            }
        }
    }

    pub async fn recv<'a>(
        &self,
        bufs: &mut [io::IoSliceMut<'a>],
        flags: libc::c_int,
    ) -> io::Result<(usize, bool)> {
        use socket2::MaybeUninitSlice;

        let (n, f) = match (self.state.nonblocking, self.state.so_recv_timeout) {
            (true, None) => {
                let f = self
                    .inner
                    .get_async_socket()?
                    .async_io(Interest::READABLE, |s| {
                        let bufs = unsafe {
                            &mut *(bufs as *mut [io::IoSliceMut<'_>]
                                as *mut [MaybeUninitSlice<'_>])
                        };
                        s.recv_vectored_with_flags(bufs, flags)
                    });

                tokio::time::timeout(std::time::Duration::from_millis(50), f)
                    .await
                    .map_err(|_| io::Error::from(io::ErrorKind::WouldBlock))??
            }
            (false, None) => {
                self.inner
                    .get_async_socket()?
                    .async_io(Interest::READABLE, |s| {
                        let bufs = unsafe {
                            &mut *(bufs as *mut [io::IoSliceMut<'_>]
                                as *mut [MaybeUninitSlice<'_>])
                        };
                        s.recv_vectored_with_flags(bufs, flags)
                    })
                    .await?
            }
            (_, Some(timeout)) => {
                let f = self
                    .inner
                    .get_async_socket()?
                    .async_io(Interest::READABLE, |s| {
                        let bufs = unsafe {
                            &mut *(bufs as *mut [io::IoSliceMut<'_>]
                                as *mut [MaybeUninitSlice<'_>])
                        };
                        s.recv_vectored_with_flags(bufs, flags)
                    });

                tokio::time::timeout(timeout, f)
                    .await
                    .map_err(|_| io::Error::from(io::ErrorKind::WouldBlock))??
            }
        };

        Ok((n, f.is_truncated()))
    }

    pub async fn recv_from<'a>(
        &self,
        bufs: &mut [io::IoSliceMut<'a>],
        flags: libc::c_int,
    ) -> io::Result<(usize, bool, Option<net::SocketAddr>)> {
        use socket2::MaybeUninitSlice;

        let (n, f, addr) = match (self.state.nonblocking, self.state.so_recv_timeout) {
            (true, None) => {
                let f = self
                    .inner
                    .get_async_socket()?
                    .async_io(Interest::READABLE, |s| {
                        let bufs = unsafe {
                            &mut *(bufs as *mut [io::IoSliceMut<'_>]
                                as *mut [MaybeUninitSlice<'_>])
                        };
                        s.recv_from_vectored_with_flags(bufs, flags)
                    });

                tokio::time::timeout(std::time::Duration::from_millis(50), f)
                    .await
                    .map_err(|_| io::Error::from(io::ErrorKind::WouldBlock))??
            }
            (false, None) => {
                let f = self
                    .inner
                    .get_async_socket()?
                    .async_io(Interest::READABLE, |s| {
                        let bufs = unsafe {
                            &mut *(bufs as *mut [io::IoSliceMut<'_>]
                                as *mut [MaybeUninitSlice<'_>])
                        };
                        s.recv_from_vectored_with_flags(bufs, flags)
                    });

                f.await?
            }
            (_, Some(timeout)) => {
                let f = self
                    .inner
                    .get_async_socket()?
                    .async_io(Interest::READABLE, |s| {
                        let bufs = unsafe {
                            &mut *(bufs as *mut [io::IoSliceMut<'_>]
                                as *mut [MaybeUninitSlice<'_>])
                        };
                        s.recv_from_vectored_with_flags(bufs, flags)
                    });

                tokio::time::timeout(timeout, f)
                    .await
                    .map_err(|_| io::Error::from(io::ErrorKind::WouldBlock))??
            }
        };
        Ok((n, f.is_truncated(), addr.as_socket()))
    }

    pub async fn send<'a>(
        &self,
        bufs: &[io::IoSlice<'a>],
        flags: libc::c_int,
    ) -> io::Result<usize> {
        let n = match (self.state.nonblocking, self.state.so_send_timeout) {
            (true, None) => {
                let f = self
                    .inner
                    .get_async_socket()?
                    .async_io(Interest::WRITABLE, |s| {
                        s.send_vectored_with_flags(bufs, flags)
                    });

                tokio::time::timeout(std::time::Duration::from_millis(50), f)
                    .await
                    .map_err(|_| io::Error::from(io::ErrorKind::WouldBlock))??
            }
            (false, None) => {
                let f = self
                    .inner
                    .get_async_socket()?
                    .async_io(Interest::WRITABLE, |s| {
                        s.send_vectored_with_flags(bufs, flags)
                    });

                f.await?
            }
            (_, Some(timeout)) => {
                let f = self
                    .inner
                    .get_async_socket()?
                    .async_io(Interest::WRITABLE, |s| {
                        s.send_vectored_with_flags(bufs, flags)
                    });

                tokio::time::timeout(timeout, f)
                    .await
                    .map_err(|_| io::Error::from(io::ErrorKind::WouldBlock))??
            }
        };

        Ok(n)
    }

    pub async fn send_to<'a>(
        &self,
        bufs: &[io::IoSlice<'a>],
        addr: net::SocketAddr,
        flags: libc::c_int,
    ) -> io::Result<usize> {
        use socket2::{MaybeUninitSlice, SockAddr};
        let address = SockAddr::from(addr);

        let n = match (self.state.nonblocking, self.state.so_send_timeout) {
            (true, None) => {
                let f = self
                    .inner
                    .get_async_socket()?
                    .async_io(Interest::WRITABLE, |s| {
                        s.send_to_vectored_with_flags(bufs, &address, flags)
                    });

                tokio::time::timeout(std::time::Duration::from_millis(50), f)
                    .await
                    .map_err(|_| io::Error::from(io::ErrorKind::WouldBlock))??
            }
            (false, None) => {
                let f = self
                    .inner
                    .get_async_socket()?
                    .async_io(Interest::WRITABLE, |s| {
                        s.send_to_vectored_with_flags(bufs, &address, flags)
                    });

                f.await?
            }
            (_, Some(timeout)) => {
                let f = self
                    .inner
                    .get_async_socket()?
                    .async_io(Interest::WRITABLE, |s| {
                        s.send_to_vectored_with_flags(bufs, &address, flags)
                    });

                tokio::time::timeout(timeout, f)
                    .await
                    .map_err(|_| io::Error::from(io::ErrorKind::WouldBlock))??
            }
        };

        Ok(n)
    }

    pub fn shutdown(&mut self, how: net::Shutdown) -> io::Result<()> {
        self.inner.get_ref()?.shutdown(how)?;
        self.state.shutdown.insert(how);
        Ok(())
    }

    pub fn get_peer(&mut self) -> io::Result<net::SocketAddr> {
        if let Some(addr) = self.state.peer_addr {
            Ok(addr)
        } else {
            let addr = self.inner.get_ref()?.peer_addr()?.as_socket().unwrap();
            self.state.peer_addr = Some(addr);
            Ok(addr)
        }
    }

    pub fn get_local(&mut self) -> io::Result<net::SocketAddr> {
        if let Some(addr) = self.state.local_addr {
            Ok(addr)
        } else {
            let addr = self.inner.get_ref()?.local_addr()?.as_socket().unwrap();
            self.state.local_addr = Some(addr);
            Ok(addr)
        }
    }

    pub fn set_nonblocking(&mut self, nonblocking: bool) -> io::Result<()> {
        self.state.nonblocking = nonblocking;
        Ok(())
    }

    pub fn get_nonblocking(&self) -> bool {
        self.state.nonblocking
    }

    pub fn get_so_type(&self) -> (AddressFamily, SocketType) {
        self.state.sock_type
    }

    pub fn get_so_accept_conn(&self) -> io::Result<bool> {
        self.inner.get_ref()?.is_listener()
    }

    pub fn sync_conn_state(&mut self) {
        if self.state.so_conn_state == ConnectState::Connecting {
            self.state.so_conn_state = ConnectState::Connected;
        }
    }

    pub fn set_so_reuseaddr(&mut self, reuseaddr: bool) -> io::Result<()> {
        self.state.so_reuseaddr = reuseaddr;
        Ok(())
    }

    pub fn get_so_reuseaddr(&self) -> bool {
        self.state.so_reuseaddr
    }

    pub fn set_so_recv_buf_size(&mut self, buf_size: usize) -> io::Result<()> {
        self.state.so_recv_buf_size = buf_size;
        Ok(())
    }

    pub fn get_so_recv_buf_size(&self) -> usize {
        self.state.so_recv_buf_size
    }

    pub fn set_so_send_buf_size(&mut self, buf_size: usize) -> io::Result<()> {
        self.state.so_send_buf_size = buf_size;
        Ok(())
    }

    pub fn get_so_send_buf_size(&mut self) -> usize {
        self.state.so_send_buf_size
    }

    pub fn set_so_recv_timeout(&mut self, timeout: Option<Duration>) -> io::Result<()> {
        self.state.so_recv_timeout = timeout;
        self.state.nonblocking = true;
        Ok(())
    }

    pub fn get_so_recv_timeout(&mut self) -> Option<Duration> {
        self.state.so_recv_timeout
    }

    pub fn set_so_send_timeout(&mut self, timeout: Option<Duration>) -> io::Result<()> {
        self.state.so_send_timeout = timeout;
        self.state.nonblocking = true;
        Ok(())
    }

    pub fn get_so_send_timeout(&mut self) -> Option<Duration> {
        self.state.so_send_timeout
    }

    pub fn get_so_error(&mut self) -> io::Result<Option<io::Error>> {
        self.inner.get_ref()?.take_error()
    }
}
