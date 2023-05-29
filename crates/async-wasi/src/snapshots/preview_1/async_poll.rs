use std::time::Duration;

use futures::{stream::FuturesUnordered, StreamExt};
use tokio::io::unix::AsyncFdReadyGuard;

use crate::snapshots::common::memory::{Memory, WasmPtr};
use crate::snapshots::common::net::{self, SubscriptionClock};
use crate::snapshots::common::types::*;
use crate::snapshots::env::VFD;
use crate::snapshots::Errno;
use crate::snapshots::WasiCtx;
use net::async_tokio::AsyncWasiSocket;
use net::{PrePoll, SubscriptionFd, SubscriptionFdType};

fn handle_event_err(type_: SubscriptionFdType, errno: Errno) -> __wasi_event_t {
    let mut r = __wasi_event_t {
        userdata: 0,
        error: errno.0,
        type_: 0,
        fd_readwrite: __wasi_event_fd_readwrite_t {
            nbytes: 0,
            flags: 0,
        },
    };
    match type_ {
        SubscriptionFdType::Read(userdata) => {
            r.userdata = userdata;
            r.type_ = __wasi_eventtype_t::__WASI_EVENTTYPE_FD_READ;
        }
        SubscriptionFdType::Write(userdata) => {
            r.userdata = userdata;
            r.type_ = __wasi_eventtype_t::__WASI_EVENTTYPE_FD_WRITE;
        }
        SubscriptionFdType::Both { read: userdata, .. } => {
            r.userdata = userdata;
            r.type_ = __wasi_eventtype_t::__WASI_EVENTTYPE_FD_READ;
        }
    }
    r
}

async fn wait_fd(fd: &AsyncWasiSocket, type_: SubscriptionFdType) -> Result<__wasi_event_t, Errno> {
    let handler =
        |r: Result<AsyncFdReadyGuard<socket2::Socket>, std::io::Error>, userdata, type_| match r {
            Ok(mut s) => {
                s.clear_ready();
                __wasi_event_t {
                    userdata,
                    error: 0,
                    type_,
                    fd_readwrite: __wasi_event_fd_readwrite_t {
                        nbytes: 0,
                        flags: 0,
                    },
                }
            }
            Err(e) => __wasi_event_t {
                userdata,
                error: Errno::from(e).0,
                type_,
                fd_readwrite: __wasi_event_fd_readwrite_t {
                    nbytes: 0,
                    flags: __wasi_eventrwflags_t::__WASI_EVENTRWFLAGS_FD_READWRITE_HANGUP,
                },
            },
        };

    match type_ {
        SubscriptionFdType::Write(userdata) => Ok(handler(
            fd.inner.writable().await,
            userdata,
            __wasi_eventtype_t::__WASI_EVENTTYPE_FD_WRITE,
        )),
        SubscriptionFdType::Read(userdata) => Ok(handler(
            fd.inner.readable().await,
            userdata,
            __wasi_eventtype_t::__WASI_EVENTTYPE_FD_READ,
        )),
        SubscriptionFdType::Both { read, write } => {
            tokio::select! {
                read_result=fd.inner.readable()=>{
                    Ok(handler(
                        read_result,
                        read,
                        __wasi_eventtype_t::__WASI_EVENTTYPE_FD_READ,
                    ))
                }
                write_result=fd.inner.writable()=>{
                    Ok(handler(
                        write_result,
                        write,
                        __wasi_eventtype_t::__WASI_EVENTTYPE_FD_WRITE,
                    ))
                }
            }
        }
    }
}

async fn poll_only_fd<M: Memory>(
    ctx: &mut WasiCtx,
    mem: &mut M,
    out_ptr: WasmPtr<__wasi_event_t>,
    nsubscriptions: usize,
    revents_num_ptr: WasmPtr<__wasi_size_t>,
    fd_vec: Vec<SubscriptionFd>,
) -> Result<(), Errno> {
    if fd_vec.is_empty() {
        mem.write_data(revents_num_ptr, 0)?;
    } else {
        let r_events = mem.mut_slice(out_ptr, nsubscriptions)?;
        let mut wait = FuturesUnordered::new();

        let mut i = 0;

        for SubscriptionFd { fd, type_ } in fd_vec {
            match ctx.get_vfd(fd) {
                Ok(VFD::AsyncSocket(s)) => {
                    wait.push(wait_fd(s, type_));
                }
                Ok(VFD::Closed) => {
                    r_events[i] = handle_event_err(type_, Errno::__WASI_ERRNO_IO);
                    i += 1;
                }
                _ => {
                    r_events[i] = handle_event_err(type_, Errno::__WASI_ERRNO_NOTSOCK);
                    i += 1;
                }
            }
        }

        if i == 0 {
            let v = wait.select_next_some().await?;
            r_events[i] = v;
            i += 1;

            'wait_poll: loop {
                if i >= nsubscriptions {
                    break 'wait_poll;
                }
                println!("poll {}", wait.len());
                futures::select! {
                    v = wait.next() => {
                        if let Some(v) = v {
                            r_events[i] = v?;
                            i += 1;
                        } else {
                            break 'wait_poll;
                        }
                    }
                    default => {
                        break 'wait_poll;
                    }
                };
            }
        }

        mem.write_data(revents_num_ptr, i as u32)?;
    }
    Ok(())
}

async fn poll_fd_timeout<M: Memory>(
    ctx: &mut WasiCtx,
    mem: &mut M,
    out_ptr: WasmPtr<__wasi_event_t>,
    nsubscriptions: usize,
    revents_num_ptr: WasmPtr<__wasi_size_t>,
    clock: SubscriptionClock,
    fd_vec: Vec<SubscriptionFd>,
) -> Result<(), Errno> {
    let r_events = mem.mut_slice(out_ptr, nsubscriptions)?;
    let mut wait = FuturesUnordered::new();

    let mut i = 0;

    for SubscriptionFd { fd, type_ } in fd_vec {
        match ctx.get_vfd(fd) {
            Ok(VFD::AsyncSocket(s)) => {
                wait.push(wait_fd(s, type_));
            }
            Ok(VFD::Closed) => {
                r_events[i] = handle_event_err(type_, Errno::__WASI_ERRNO_IO);
                i += 1;
            }
            _ => {
                r_events[i] = handle_event_err(type_, Errno::__WASI_ERRNO_NOTSOCK);
                i += 1;
            }
        }
    }

    if i == 0 {
        let ddl = clock.timeout.unwrap();
        let now = std::time::SystemTime::now();
        let timeout = ddl.duration_since(now).unwrap_or(Duration::from_secs(0));
        let sleep = tokio::time::timeout(timeout, wait.select_next_some()).await;
        if sleep.is_err() {
            let r_event = &mut r_events[0];
            r_event.userdata = clock.userdata;
            r_event.type_ = __wasi_eventtype_t::__WASI_EVENTTYPE_CLOCK;
            mem.write_data(revents_num_ptr, 1)?;
            return Ok(());
        }

        let first = sleep.unwrap()?;
        r_events[i] = first;
        i += 1;

        'wait: loop {
            if i >= nsubscriptions {
                break 'wait;
            }
            futures::select! {
                v = wait.next() => {
                    if let Some(v) = v {
                        r_events[i] = v?;
                        i += 1;
                    } else {
                        break 'wait;
                    }
                }
                default => {
                    break 'wait;
                }
            };
        }
    }

    mem.write_data(revents_num_ptr, i as u32)?;
    Ok(())
}

#[cfg(feature = "serialize")]
use crate::snapshots::serialize::IoState;
#[cfg(feature = "serialize")]
fn record_state(
    ctx: &mut WasiCtx,
    ddl: Option<std::time::SystemTime>,
    fds: &[SubscriptionFd],
) -> IoState {
    use crate::snapshots::serialize::PollFdState;
    let mut save_fds = vec![];
    for fd in fds {
        let poll_read;
        let poll_write;

        match fd.type_ {
            SubscriptionFdType::Read(_) => {
                poll_read = true;
                poll_write = false;
            }
            SubscriptionFdType::Write(_) => {
                poll_read = false;
                poll_write = true;
            }
            SubscriptionFdType::Both { .. } => {
                poll_read = true;
                poll_write = true;
            }
        }

        if let Ok(VFD::AsyncSocket(s)) = ctx.get_mut_vfd(fd.fd) {
            match s.state.sock_type.1 {
                net::SocketType::Datagram => {
                    // save
                    save_fds.push(PollFdState::UdpSocket {
                        fd: fd.fd,
                        socket_type: s.state.sock_type.into(),
                        local_addr: s.get_local().ok(),
                        peer_addr: s.get_peer().ok(),
                        poll_read,
                        poll_write,
                    })
                }
                net::SocketType::Stream if s.state.shutdown.is_none() => {
                    // save
                    match s.state.so_conn_state {
                        net::ConnectState::Empty => {}
                        net::ConnectState::Listening => save_fds.push(PollFdState::TcpListener {
                            fd: fd.fd,
                            socket_type: s.state.sock_type.into(),
                            local_addr: s.get_local().ok(),
                            peer_addr: s.get_peer().ok(),
                            poll_read,
                            poll_write,
                        }),
                        net::ConnectState::Connect => save_fds.push(PollFdState::TcpStream {
                            fd: fd.fd,
                            socket_type: s.state.sock_type.into(),
                            local_addr: s.get_local().ok(),
                            peer_addr: s.get_peer().ok(),
                            poll_read,
                            poll_write,
                        }),
                    }
                }
                _ => {}
            }
        }
    }

    IoState::Poll { fds: save_fds, ddl }
}

pub async fn poll_oneoff<M: Memory>(
    ctx: &mut WasiCtx,
    mem: &mut M,
    in_ptr: WasmPtr<__wasi_subscription_t>,
    out_ptr: WasmPtr<__wasi_event_t>,
    nsubscriptions: __wasi_size_t,
    revents_num_ptr: WasmPtr<__wasi_size_t>,
) -> Result<(), Errno> {
    let r = poll_oneoff_impl(ctx, mem, in_ptr, out_ptr, nsubscriptions, revents_num_ptr).await;
    #[cfg(feature = "serialize")]
    {
        ctx.io_state = IoState::Empty;
    }
    r
}

async fn poll_oneoff_impl<M: Memory>(
    ctx: &mut WasiCtx,
    mem: &mut M,
    in_ptr: WasmPtr<__wasi_subscription_t>,
    out_ptr: WasmPtr<__wasi_event_t>,
    nsubscriptions: __wasi_size_t,
    revents_num_ptr: WasmPtr<__wasi_size_t>,
) -> Result<(), Errno> {
    if nsubscriptions <= 0 {
        return Ok(());
    }

    let nsubscriptions = nsubscriptions as usize;

    let subs = mem.get_slice(in_ptr, nsubscriptions)?;
    let prepoll = PrePoll::from_wasi_subscription(subs)?;

    match prepoll {
        PrePoll::OnlyFd(fd_vec) => {
            #[cfg(feature = "serialize")]
            {
                if let IoState::Empty = ctx.io_state {
                    ctx.io_state = record_state(ctx, None, &fd_vec);
                }
            }
            poll_only_fd(ctx, mem, out_ptr, nsubscriptions, revents_num_ptr, fd_vec).await?;
        }
        PrePoll::ClockAndFd(clock, fd_vec) => {
            #[cfg(feature = "serialize")]
            let clock = {
                // resume
                if let IoState::Poll { ddl, .. } = ctx.io_state {
                    let mut clock_clone = clock.clone();
                    clock_clone.timeout = ddl;
                    clock_clone
                } else {
                    ctx.io_state = record_state(ctx, clock.timeout, &fd_vec);
                    clock
                }
            };
            poll_fd_timeout(
                ctx,
                mem,
                out_ptr,
                nsubscriptions,
                revents_num_ptr,
                clock,
                fd_vec,
            )
            .await?;
        }
        PrePoll::OnlyClock(clock) => {
            if let Some(e) = clock.err {
                let r_event = mem.mut_data(out_ptr)?;
                r_event.userdata = clock.userdata;
                r_event.type_ = __wasi_eventtype_t::__WASI_EVENTTYPE_CLOCK;
                r_event.error = Errno::from(e).0;
                mem.write_data(revents_num_ptr, 1)?;
                return Ok(());
            }
            if let Some(ddl) = clock.timeout {
                #[cfg(feature = "serialize")]
                let ddl = {
                    // resume
                    if let IoState::Sleep { ddl } = ctx.io_state {
                        ddl
                    } else {
                        ctx.io_state = IoState::Sleep { ddl };
                        ddl
                    }
                };
                let now = std::time::SystemTime::now();
                let dur = ddl.duration_since(now).unwrap_or(Duration::from_secs(0));
                tokio::time::sleep(dur).await;
                let r_event = mem.mut_data(out_ptr)?;
                r_event.userdata = clock.userdata;
                r_event.type_ = __wasi_eventtype_t::__WASI_EVENTTYPE_CLOCK;
                mem.write_data(revents_num_ptr, 1)?;
                return Ok(());
            }
        }
    }
    Ok(())
}
