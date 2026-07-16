use super::*;
use crate::snapshots::{
    common::{types as wasi_types, vfs},
    env::Errno,
};
use socket2::{MaybeUninitSlice, SockAddr, Socket};
use std::{
    ops::DerefMut,
    os::unix::prelude::{AsRawFd, FromRawFd, RawFd},
    sync::atomic::{AtomicBool, AtomicI8, AtomicU8},
};
use tokio::io::{
    AsyncReadExt, AsyncWriteExt, Interest,
    unix::{AsyncFd, AsyncFdReadyGuard, TryIoError},
};

/// Reinterprets an initialized `IoSliceMut` buffer view as a `MaybeUninitSlice`
/// for socket2's vectored recv APIs (`recv_vectored_with_flags`,
/// `recv_from_vectored_with_flags`). `IoSliceMut` and `MaybeUninitSlice` are
/// both `repr(transparent)` wrappers around a single platform `iovec` (this
/// module only builds on unix, so `WSABUF` doesn't apply), so reinterpreting
/// the slice type is layout-safe; this is the same cast socket2 performs
/// internally in its own `impl Read for Socket::read_vectored`. The recv
/// calls promise never to write uninitialised bytes into the buffer, which is
/// what makes it sound to hand back a `&mut [IoSliceMut]` view afterwards.
fn as_maybe_uninit_slices<'r, 'a>(
    bufs: &'r mut [io::IoSliceMut<'a>],
) -> &'r mut [MaybeUninitSlice<'a>] {
    unsafe { &mut *(bufs as *mut [io::IoSliceMut<'_>] as *mut [MaybeUninitSlice<'_>]) }
}

/// `SO_BINDTODEVICE`-style "bind this socket to a named network interface",
/// and the matching getter. socket2 only implements these on the Linux
/// family (they map straight onto the Linux-only `SO_BINDTODEVICE` sockopt);
/// macOS/BSD expose interface binding via `IP_BOUND_IF`/`bind_device_by_index`
/// instead, which takes an interface *index*, not a name, so it isn't a
/// drop-in replacement here. Other Unix targets get an honest `ENOTSUP`
/// rather than failing to compile.
#[cfg(any(target_os = "linux", target_os = "android", target_os = "fuchsia"))]
fn socket_bind_device(s: &Socket, interface: Option<&[u8]>) -> io::Result<()> {
    s.bind_device(interface)
}

#[cfg(not(any(target_os = "linux", target_os = "android", target_os = "fuchsia")))]
fn socket_bind_device(_s: &Socket, _interface: Option<&[u8]>) -> io::Result<()> {
    Err(io::Error::from_raw_os_error(libc::ENOTSUP))
}

#[cfg(any(target_os = "linux", target_os = "android", target_os = "fuchsia"))]
fn socket_device(s: &Socket) -> io::Result<Option<Vec<u8>>> {
    s.device()
}

#[cfg(not(any(target_os = "linux", target_os = "android", target_os = "fuchsia")))]
fn socket_device(_s: &Socket) -> io::Result<Option<Vec<u8>>> {
    Err(io::Error::from_raw_os_error(libc::ENOTSUP))
}

/// `SO_ACCEPTCONN` ("has `listen(2)` been called on this socket") is gated by
/// socket2 to Linux/Android/FreeBSD/Fuchsia/AIX/Cygwin; macOS is not in that
/// list. Same honest-`ENOTSUP` fallback as above.
#[cfg(any(target_os = "linux", target_os = "android", target_os = "fuchsia"))]
fn socket_is_listener(s: &Socket) -> io::Result<bool> {
    s.is_listener()
}

#[cfg(not(any(target_os = "linux", target_os = "android", target_os = "fuchsia")))]
fn socket_is_listener(_s: &Socket) -> io::Result<bool> {
    Err(io::Error::from_raw_os_error(libc::ENOTSUP))
}

#[derive(Debug)]
pub(crate) enum AsyncWasiSocketInner {
    PreOpen(Option<Socket>),
    AsyncFd(AsyncFd<Socket>),
}

impl AsyncWasiSocketInner {
    fn register(&mut self) -> io::Result<()> {
        let socket = match self {
            AsyncWasiSocketInner::PreOpen(slot) => match slot.take() {
                Some(socket) => socket,
                None => return Err(io::Error::from_raw_os_error(libc::EINVAL)),
            },
            AsyncWasiSocketInner::AsyncFd(_) => return Ok(()),
        };
        *self = AsyncWasiSocketInner::AsyncFd(AsyncFd::new(socket)?);
        Ok(())
    }

    fn bind(&mut self, addr: &SockAddr) -> io::Result<()> {
        match self {
            AsyncWasiSocketInner::PreOpen(Some(s)) => {
                s.set_reuse_address(true)?;
                s.bind(addr)
            }
            _ => Err(io::Error::from_raw_os_error(libc::EINVAL)),
        }
    }

    fn bind_device(&mut self, interface: Option<&[u8]>) -> io::Result<()> {
        match self {
            AsyncWasiSocketInner::PreOpen(Some(s)) => socket_bind_device(s, interface),
            AsyncWasiSocketInner::AsyncFd(s) => socket_bind_device(s.get_ref(), interface),
            AsyncWasiSocketInner::PreOpen(None) => Err(io::Error::from_raw_os_error(libc::EINVAL)),
        }
    }

    fn device(&self) -> io::Result<Option<Vec<u8>>> {
        match self {
            AsyncWasiSocketInner::PreOpen(Some(s)) => socket_device(s),
            AsyncWasiSocketInner::AsyncFd(s) => socket_device(s.get_ref()),
            AsyncWasiSocketInner::PreOpen(None) => Err(io::Error::from_raw_os_error(libc::EINVAL)),
        }
    }

    fn listen(&mut self, backlog: i32) -> io::Result<()> {
        match self {
            AsyncWasiSocketInner::PreOpen(Some(s)) => {
                s.listen(backlog)?;
            }
            _ => {
                return Err(io::Error::from_raw_os_error(libc::EINVAL));
            }
        }
        self.register()
    }

    async fn accept(&mut self) -> io::Result<(Socket, SockAddr)> {
        match self {
            AsyncWasiSocketInner::PreOpen(_) => Err(io::Error::from_raw_os_error(libc::EINVAL)),
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
            AsyncWasiSocketInner::PreOpen(Some(s)) => s.connect(addr),
            _ => {
                return Err(io::Error::from_raw_os_error(libc::EINVAL));
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

    pub(crate) async fn readable(&self) -> io::Result<AsyncFdReadyGuard<'_, Socket>> {
        match self {
            AsyncWasiSocketInner::PreOpen(_) => Err(io::Error::from_raw_os_error(libc::ENOTCONN)),
            AsyncWasiSocketInner::AsyncFd(s) => Ok(s.readable().await?),
        }
    }

    pub(crate) async fn writable(&self) -> io::Result<AsyncFdReadyGuard<'_, Socket>> {
        match self {
            AsyncWasiSocketInner::PreOpen(_) => Err(io::Error::from_raw_os_error(libc::ENOTCONN)),
            AsyncWasiSocketInner::AsyncFd(s) => Ok(s.writable().await?),
        }
    }
}

#[derive(Debug)]
pub(crate) struct SocketWritable {
    count: AtomicI8,
    notify: tokio::sync::Notify,
}
impl SocketWritable {
    pub(crate) async fn writable(&self) {
        // Consume one write-budget unit; proceed while the pre-decrement value stays >= 0.
        let b = self
            .count
            .fetch_sub(1, std::sync::atomic::Ordering::Acquire);
        if b >= 0 {
            return;
        }
        // Budget exhausted: wait for `set_writable()` (10s cap) — `Notify` delivers a real wakeup, and the timeout result is inspected.
        if tokio::time::timeout(Duration::from_secs(10), self.notify.notified())
            .await
            .is_err()
        {
            log::trace!("SocketWritable::writable timed out waiting for writability");
        }
    }

    pub(crate) fn set_writable(&self) {
        self.count.store(5, std::sync::atomic::Ordering::Release);
        // notify_one() stores a permit when no waiter is parked, so a signal that
        // races ahead of writable().await is not lost.
        self.notify.notify_one();
    }
}
impl Default for SocketWritable {
    fn default() -> Self {
        Self {
            count: AtomicI8::new(5),
            notify: tokio::sync::Notify::new(),
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
            socket_bind_device(&inner, Some(&state.bind_device))?;
        }
        Ok(AsyncWasiSocket {
            inner: AsyncWasiSocketInner::PreOpen(Some(inner)),
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
        let (n, f) = match (self.state.nonblocking, self.state.so_recv_timeout) {
            (true, None) => {
                let f = self
                    .inner
                    .get_async_socket()?
                    .async_io(Interest::READABLE, |s| {
                        s.recv_vectored_with_flags(as_maybe_uninit_slices(bufs), flags)
                    });

                tokio::time::timeout(std::time::Duration::from_millis(50), f)
                    .await
                    .map_err(|_| io::Error::from(io::ErrorKind::WouldBlock))??
            }
            (false, None) => {
                self.inner
                    .get_async_socket()?
                    .async_io(Interest::READABLE, |s| {
                        s.recv_vectored_with_flags(as_maybe_uninit_slices(bufs), flags)
                    })
                    .await?
            }
            (_, Some(timeout)) => {
                let f = self
                    .inner
                    .get_async_socket()?
                    .async_io(Interest::READABLE, |s| {
                        s.recv_vectored_with_flags(as_maybe_uninit_slices(bufs), flags)
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
        let (n, f, addr) = match (self.state.nonblocking, self.state.so_recv_timeout) {
            (true, None) => {
                let f = self
                    .inner
                    .get_async_socket()?
                    .async_io(Interest::READABLE, |s| {
                        s.recv_from_vectored_with_flags(as_maybe_uninit_slices(bufs), flags)
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
                        s.recv_from_vectored_with_flags(as_maybe_uninit_slices(bufs), flags)
                    });

                f.await?
            }
            (_, Some(timeout)) => {
                let f = self
                    .inner
                    .get_async_socket()?
                    .async_io(Interest::READABLE, |s| {
                        s.recv_from_vectored_with_flags(as_maybe_uninit_slices(bufs), flags)
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
        use socket2::SockAddr;
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
        socket_is_listener(self.inner.get_ref()?)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::snapshots::common::net::{AddressFamily, SocketType, WasiSocketState};
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    fn tcp_socket() -> AsyncWasiSocket {
        let state = WasiSocketState {
            sock_type: (AddressFamily::Inet4, SocketType::Stream),
            ..Default::default()
        };
        AsyncWasiSocket::open(state).unwrap()
    }

    // Drive the `Option`-based register() state machine end to end: `listen` and `connect` both move PreOpen -> AsyncFd, and the loopback must carry data.
    #[tokio::test]
    async fn register_tcp_loopback_roundtrip() {
        let loopback = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0);

        let mut server = tcp_socket();
        server.bind(loopback).unwrap();
        server.listen(128).unwrap(); // register() on the server side
        // `bind()` cached port 0, so read the real OS-assigned port from the registered socket.
        let addr = server
            .inner
            .get_ref()
            .unwrap()
            .local_addr()
            .unwrap()
            .as_socket()
            .unwrap();
        assert!(addr.port() > 0);

        let mut client = tcp_socket();
        let (accepted, connected) = tokio::join!(server.accept(), client.connect(addr));
        let accepted = accepted.unwrap();
        connected.unwrap(); // register() on the client side

        client
            .send(&[std::io::IoSlice::new(b"ping")], 0)
            .await
            .unwrap();

        let mut buf = [0u8; 8];
        let (n, _) = accepted
            .recv(&mut [std::io::IoSliceMut::new(&mut buf)], 0)
            .await
            .unwrap();
        assert_eq!(&buf[..n], b"ping");
    }

    // Virtual time makes this deterministic: if `set_writable()` failed to wake the waiter, only the 10s timeout would release it, so the 1s bound below would elapse first.
    #[tokio::test(start_paused = true)]
    async fn set_writable_wakes_blocked_writable() {
        use std::sync::Arc;

        let sw = Arc::new(SocketWritable::default());
        // Drain the initial write budget of 5.
        for _ in 0..6 {
            sw.writable().await;
        }

        // The next call has no budget left and must block until set_writable().
        let waiter = {
            let sw = sw.clone();
            tokio::spawn(async move { sw.writable().await })
        };
        tokio::task::yield_now().await;
        assert!(
            !waiter.is_finished(),
            "writable() should block once the budget is exhausted"
        );

        sw.set_writable();
        tokio::time::timeout(Duration::from_secs(1), waiter)
            .await
            .expect("set_writable() did not wake writable() within 1s")
            .unwrap();
    }
}
