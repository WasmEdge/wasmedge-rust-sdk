#[cfg(all(unix, feature = "async_tokio"))]
pub mod async_tokio;

pub use super::vfs::*;

use std::future::Future;
use std::io::{self, Read, Write};
use std::net;
use std::time::{Duration, SystemTime};

#[derive(Debug, Clone, Copy)]
pub enum AddressFamily {
    Inet4,
    Inet6,
}

impl Default for AddressFamily {
    fn default() -> Self {
        AddressFamily::Inet4
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SocketType {
    Datagram,
    Stream,
}

impl Default for SocketType {
    fn default() -> Self {
        SocketType::Stream
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ConnectState {
    Empty,
    Listening,
    Connect,
}

impl Default for ConnectState {
    fn default() -> Self {
        Self::Empty
    }
}

#[derive(Debug, Clone, Default)]
pub struct WasiSocketState {
    pub sock_type: (AddressFamily, SocketType),
    pub local_addr: Option<net::SocketAddr>,
    pub peer_addr: Option<net::SocketAddr>,
    pub bind_device: Vec<u8>,
    pub backlog: u32,
    pub shutdown: Option<net::Shutdown>,
    pub nonblocking: bool,
    pub so_reuseaddr: bool,
    pub so_conn_state: ConnectState,
    pub so_recv_buf_size: usize,
    pub so_send_buf_size: usize,
    pub so_recv_timeout: Option<Duration>,
    pub so_send_timeout: Option<Duration>,
    pub fs_rights: WASIRights,
}

use super::error::Errno;
use super::types::{self as wasi_types, __wasi_subscription_t};

#[derive(Debug, Clone, Copy)]
pub enum SubscriptionFdType {
    Read(wasi_types::__wasi_userdata_t),
    Write(wasi_types::__wasi_userdata_t),
    Both {
        read: wasi_types::__wasi_userdata_t,
        write: wasi_types::__wasi_userdata_t,
    },
}

#[derive(Debug, Clone, Copy)]
pub struct SubscriptionFd {
    pub fd: wasi_types::__wasi_fd_t,
    pub type_: SubscriptionFdType,
}

impl SubscriptionFd {
    pub fn set_write(&mut self, userdata: wasi_types::__wasi_userdata_t) {
        let read_userdata = match &mut self.type_ {
            SubscriptionFdType::Read(v) => *v,
            SubscriptionFdType::Write(v) => {
                *v = userdata;
                return;
            }
            SubscriptionFdType::Both { read, write } => {
                *write = userdata;
                return;
            }
        };
        self.type_ = SubscriptionFdType::Both {
            read: read_userdata,
            write: userdata,
        };
    }

    pub fn set_read(&mut self, userdata: wasi_types::__wasi_userdata_t) {
        let write_userdata = match &mut self.type_ {
            SubscriptionFdType::Write(v) => *v,
            SubscriptionFdType::Read(v) => {
                *v = userdata;
                return;
            }
            SubscriptionFdType::Both { read, write } => {
                *read = userdata;
                return;
            }
        };
        self.type_ = SubscriptionFdType::Both {
            read: userdata,
            write: write_userdata,
        };
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SubscriptionClock {
    pub timeout: Option<SystemTime>,
    pub userdata: wasi_types::__wasi_userdata_t,
    pub err: Option<Errno>,
}

#[derive(Debug, Clone, Copy)]
pub enum Subscription {
    FD(SubscriptionFd),
    RealClock(SubscriptionClock),
}

impl Subscription {
    pub fn from(s: &__wasi_subscription_t) -> Result<Subscription, Errno> {
        use wasi_types::__wasi_clockid_t::__WASI_CLOCKID_MONOTONIC as CLOCKID_MONOTONIC;
        use wasi_types::__wasi_clockid_t::__WASI_CLOCKID_REALTIME as CLOCKID_REALTIME;
        use wasi_types::__wasi_eventtype_t::{
            __WASI_EVENTTYPE_CLOCK as CLOCK, __WASI_EVENTTYPE_FD_READ as RD,
            __WASI_EVENTTYPE_FD_WRITE as WR,
        };

        let userdata = s.userdata;
        match s.u.tag {
            CLOCK => {
                let clock = unsafe { s.u.u.clock };
                match clock.id {
                    CLOCKID_REALTIME | CLOCKID_MONOTONIC => {
                        if clock.flags == 1 {
                            if let Some(ddl) = std::time::UNIX_EPOCH
                                .checked_add(Duration::from_nanos(clock.timeout + clock.precision))
                            {
                                Ok(Subscription::RealClock(SubscriptionClock {
                                    timeout: Some(ddl),
                                    userdata,
                                    err: None,
                                }))
                            } else {
                                Ok(Subscription::RealClock(SubscriptionClock {
                                    timeout: None,
                                    userdata,
                                    err: Some(Errno::__WASI_ERRNO_INVAL),
                                }))
                            }
                        } else {
                            if clock.timeout == 0 {
                                Ok(Subscription::RealClock(SubscriptionClock {
                                    timeout: None,
                                    userdata,
                                    err: None,
                                }))
                            } else {
                                let duration =
                                    Duration::from_nanos(clock.timeout + clock.precision);

                                let timeout = std::time::SystemTime::now().checked_add(duration);

                                Ok(Subscription::RealClock(SubscriptionClock {
                                    timeout,
                                    userdata,
                                    err: None,
                                }))
                            }
                        }
                    }

                    _ => Ok(Subscription::RealClock(SubscriptionClock {
                        timeout: None,
                        userdata,
                        err: Some(Errno::__WASI_ERRNO_NODEV),
                    })),
                }
            }
            RD => {
                let fd_read = unsafe { s.u.u.fd_read };
                Ok(Subscription::FD(SubscriptionFd {
                    fd: fd_read.file_descriptor,
                    type_: SubscriptionFdType::Read(userdata),
                }))
            }
            WR => {
                let fd_read = unsafe { s.u.u.fd_read };
                Ok(Subscription::FD(SubscriptionFd {
                    fd: fd_read.file_descriptor,
                    type_: SubscriptionFdType::Write(userdata),
                }))
            }
            _ => Err(Errno::__WASI_ERRNO_INVAL),
        }
    }
}

pub enum PrePoll {
    OnlyFd(Vec<SubscriptionFd>),
    OnlyClock(SubscriptionClock),
    ClockAndFd(SubscriptionClock, Vec<SubscriptionFd>),
}

impl PrePoll {
    pub fn from_wasi_subscription(
        subs: &[wasi_types::__wasi_subscription_t],
    ) -> Result<Self, Errno> {
        use std::collections::HashMap;
        let mut fds = HashMap::with_capacity(subs.len());

        let mut timeout: Option<SubscriptionClock> = None;
        for s in subs {
            let s = Subscription::from(s)?;
            match s {
                Subscription::FD(fd) => {
                    let type_ = fd.type_;

                    fds.entry(fd.fd)
                        .and_modify(|e: &mut SubscriptionFd| match type_ {
                            SubscriptionFdType::Read(data) => e.set_read(data),
                            SubscriptionFdType::Write(data) => e.set_write(data),
                            SubscriptionFdType::Both { read, write } => {
                                e.type_ = SubscriptionFdType::Both { read, write };
                            }
                        })
                        .or_insert(fd);
                }
                Subscription::RealClock(clock) => {
                    if clock.err.is_some() {
                        return Ok(PrePoll::OnlyClock(clock));
                    }
                    if clock.timeout.is_none() {
                        return Ok(PrePoll::OnlyClock(clock));
                    }

                    if let Some(old_clock) = &mut timeout {
                        let new_timeout = clock.timeout.unwrap();
                        let old_timeout = old_clock.timeout.unwrap();

                        if new_timeout < old_timeout {
                            *old_clock = clock
                        }
                    } else {
                        timeout = Some(clock)
                    }
                }
            }
        }

        let fd_vec: Vec<SubscriptionFd> = fds.into_values().collect();

        if let Some(clock) = timeout {
            if fd_vec.is_empty() {
                Ok(PrePoll::OnlyClock(clock))
            } else {
                Ok(PrePoll::ClockAndFd(clock, fd_vec))
            }
        } else {
            Ok(PrePoll::OnlyFd(fd_vec))
        }
    }
}
