use crate::snapshots::{
    common::{
        memory::{Memory, WasmPtr},
        net::{self, ConnectState, SubscriptionClock},
        types::*,
    },
    Errno, WasiCtx,
};
use futures::{stream::FuturesUnordered, StreamExt};
use net::{async_tokio::AsyncWasiSocket, PrePoll, SubscriptionFd, SubscriptionFdType};
use std::time::Duration;

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

async fn wait_fd(
    fd_index: usize,
    socket: &AsyncWasiSocket,
    type_: SubscriptionFdType,
) -> Result<(__wasi_event_t, Option<usize>), Errno> {
    let connecting = ConnectState::Connecting == socket.state.so_conn_state;

    let handler = |r: Result<(), std::io::Error>, userdata, type_| {
        log::trace!("wait_fd {fd_index} {r:?}");
        match r {
            Ok(_) => (
                __wasi_event_t {
                    userdata,
                    error: 0,
                    type_,
                    fd_readwrite: __wasi_event_fd_readwrite_t {
                        nbytes: 0,
                        flags: 0,
                    },
                },
                if connecting { Some(fd_index) } else { None },
            ),
            Err(e) => (
                __wasi_event_t {
                    userdata,
                    error: Errno::from(e).0,
                    type_,
                    fd_readwrite: __wasi_event_fd_readwrite_t {
                        nbytes: 0,
                        flags: __wasi_eventrwflags_t::__WASI_EVENTRWFLAGS_FD_READWRITE_HANGUP,
                    },
                },
                None,
            ),
        }
    };

    match type_ {
        SubscriptionFdType::Write(userdata) => {
            let write_result = socket.writable().await;
            log::trace!("wait_fd {fd_index} writeable");

            Ok(handler(
                write_result,
                userdata,
                __wasi_eventtype_t::__WASI_EVENTTYPE_FD_WRITE,
            ))
        }
        SubscriptionFdType::Read(userdata) => {
            let read_result = socket.readable().await;
            log::trace!("wait_fd {fd_index} readable");

            Ok(handler(
                read_result,
                userdata,
                __wasi_eventtype_t::__WASI_EVENTTYPE_FD_READ,
            ))
        }
        SubscriptionFdType::Both { read, write } => {
            tokio::select! {
                read_result=socket.readable()=>{
                    log::trace!("wait_fd {fd_index} readable");

                    Ok(handler(
                        read_result,
                        read,
                        __wasi_eventtype_t::__WASI_EVENTTYPE_FD_READ,
                    ))
                }
                write_result=socket.writable()=>{
                    log::trace!("wait_fd {fd_index} writeable");

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
            match ctx.vfs.get_socket(fd as usize) {
                Ok(s) => {
                    wait.push(wait_fd(fd as usize, s, type_));
                }
                Err(e) => {
                    r_events[i] = handle_event_err(type_, e);
                    i += 1;
                }
            }
        }

        if i == 0 {
            let mut connected_fds = vec![];

            let (v, connected_fd) = wait.select_next_some().await?;
            connected_fds.push(connected_fd);
            r_events[i] = v;
            i += 1;

            'wait_poll: loop {
                if i >= nsubscriptions {
                    break 'wait_poll;
                }
                futures::select! {
                    v = wait.next() => {
                        if let Some(v) = v {
                            let (v, connected_fd) = v?;
                            connected_fds.push(connected_fd);
                            r_events[i] = v;
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

            drop(wait);

            for fd in connected_fds.into_iter().flatten() {
                if let Ok(socket) = ctx.vfs.get_mut_socket(fd) {
                    socket.state.so_conn_state = ConnectState::Connected;
                    socket.writable.set_writable();
                }
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
        match ctx.vfs.get_socket(fd as usize) {
            Ok(s) => {
                wait.push(wait_fd(fd as usize, s, type_));
            }
            Err(e) => {
                r_events[i] = handle_event_err(type_, e);
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

        let mut connected_fds = vec![];

        let (first, connected_fd) = sleep.unwrap()?;
        connected_fds.push(connected_fd);
        r_events[i] = first;
        i += 1;

        'wait: loop {
            if i >= nsubscriptions {
                break 'wait;
            }
            futures::select! {
                v = wait.next() => {
                    if let Some(v) = v {
                        let (v,connected_fd) = v?;
                        connected_fds.push(connected_fd);
                        r_events[i] = v;
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

        drop(wait);

        for fd in connected_fds.into_iter().flatten() {
            if let Ok(socket) = ctx.vfs.get_mut_socket(fd) {
                socket.state.so_conn_state = ConnectState::Connected;
            }
        }
    }

    mem.write_data(revents_num_ptr, i as u32)?;
    Ok(())
}

pub async fn poll_oneoff<M: Memory>(
    ctx: &mut WasiCtx,
    mem: &mut M,
    in_ptr: WasmPtr<__wasi_subscription_t>,
    out_ptr: WasmPtr<__wasi_event_t>,
    nsubscriptions: __wasi_size_t,
    revents_num_ptr: WasmPtr<__wasi_size_t>,
) -> Result<(), Errno> {
    log::trace!("poll_oneoff");
    poll_oneoff_impl(ctx, mem, in_ptr, out_ptr, nsubscriptions, revents_num_ptr).await
}

async fn poll_oneoff_impl<M: Memory>(
    ctx: &mut WasiCtx,
    mem: &mut M,
    in_ptr: WasmPtr<__wasi_subscription_t>,
    out_ptr: WasmPtr<__wasi_event_t>,
    nsubscriptions: __wasi_size_t,
    revents_num_ptr: WasmPtr<__wasi_size_t>,
) -> Result<(), Errno> {
    if nsubscriptions == 0 {
        return Ok(());
    }

    let nsubscriptions = nsubscriptions as usize;

    let subs = mem.get_slice(in_ptr, nsubscriptions)?;
    let prepoll = PrePoll::from_wasi_subscription(subs)?;

    log::trace!("poll_oneoff subs prepoll={:#?}", prepoll);

    match prepoll {
        PrePoll::OnlyFd(fd_vec) => {
            poll_only_fd(ctx, mem, out_ptr, nsubscriptions, revents_num_ptr, fd_vec).await?;
        }
        PrePoll::ClockAndFd(clock, fd_vec) => {
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
                r_event.error = e.0;
                mem.write_data(revents_num_ptr, 1)?;
                return Ok(());
            }
            if let Some(ddl) = clock.timeout {
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
