use crate::snapshots::{
    common::{
        memory::{Memory, WasmPtr},
        net::{self, AddressFamily, SocketType, WasiSocketState},
        types::*,
    },
    env::{self, VFD},
    Errno, WasiCtx,
};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

cfg_if::cfg_if! {
    if #[cfg(any(
        target_os = "linux", target_os = "android",
        target_os = "dragonfly", target_os = "freebsd",
        target_os = "openbsd", target_os = "netbsd",
        target_os = "haiku", target_os = "nto"))] {
        use libc::MSG_NOSIGNAL;
    } else {
        const MSG_NOSIGNAL: std::ffi::c_int = 0x0;
    }
}

fn parse_wasi_ip<M: Memory>(mem: &M, addr_ptr: WasmPtr<__wasi_address_t>) -> Result<IpAddr, Errno> {
    let wasi_addr = *(mem.get_data(addr_ptr)?);
    if wasi_addr.buf_len != 4 && wasi_addr.buf_len != 16 {
        return Err(Errno::__WASI_ERRNO_INVAL);
    }
    let addr_buf_ptr = WasmPtr::<u8>::from(wasi_addr.buf as usize);

    let addr = if wasi_addr.buf_len == 4 {
        let addr_buf = mem.get_slice(addr_buf_ptr, 4)?;
        IpAddr::V4(Ipv4Addr::new(
            addr_buf[0],
            addr_buf[1],
            addr_buf[2],
            addr_buf[3],
        ))
    } else {
        let addr_buf_ref = mem.get_slice(addr_buf_ptr, 16)?;
        let mut addr_buf = [0u8; 16];
        addr_buf.copy_from_slice(addr_buf_ref);
        IpAddr::V6(Ipv6Addr::from(addr_buf))
    };
    Ok(addr)
}

pub fn sock_open<M: Memory>(
    ctx: &mut WasiCtx,
    mem: &mut M,
    af: __wasi_address_family_t::Type,
    ty: __wasi_sock_type_t::Type,
    ro_fd_ptr: WasmPtr<__wasi_fd_t>,
) -> Result<(), Errno> {
    let mut state = WasiSocketState::default();
    match af {
        __wasi_address_family_t::__WASI_ADDRESS_FAMILY_INET4 => {
            state.sock_type.0 = AddressFamily::Inet4
        }
        __wasi_address_family_t::__WASI_ADDRESS_FAMILY_INET6 => {
            state.sock_type.0 = AddressFamily::Inet6
        }
        _ => return Err(Errno::__WASI_ERRNO_INVAL),
    }
    match ty {
        __wasi_sock_type_t::__WASI_SOCK_TYPE_SOCK_DGRAM => {
            state.sock_type.1 = SocketType::Datagram;
        }
        __wasi_sock_type_t::__WASI_SOCK_TYPE_SOCK_STREAM => {
            state.sock_type.1 = SocketType::Stream;
        }
        _ => return Err(Errno::__WASI_ERRNO_INVAL),
    }

    let s = net::async_tokio::AsyncWasiSocket::open(state)?;
    let fd = ctx.insert_vfd(env::VFD::AsyncSocket(s).into())?;
    mem.write_data(ro_fd_ptr, fd)?;
    Ok(())
}

pub fn sock_bind<M: Memory>(
    ctx: &mut WasiCtx,
    mem: &mut M,
    fd: __wasi_fd_t,
    addr_ptr: WasmPtr<__wasi_address_t>,
    port: u32,
) -> Result<(), Errno> {
    let ip = parse_wasi_ip(mem, addr_ptr)?;
    let addr = SocketAddr::new(ip, port as u16);

    let sock_fd = ctx.get_mut_vfd(fd)?;
    if let VFD::AsyncSocket(s) = sock_fd {
        s.bind(addr)?;
        Ok(())
    } else {
        Err(Errno::__WASI_ERRNO_NOTSOCK)
    }
}

pub fn sock_listen<M: Memory>(
    ctx: &mut WasiCtx,
    _mem: &mut M,
    fd: __wasi_fd_t,
    backlog: u32,
) -> Result<(), Errno> {
    let sock_fd = ctx.get_mut_vfd(fd)?;
    if let VFD::AsyncSocket(s) = sock_fd {
        s.listen(backlog)?;
        Ok(())
    } else {
        Err(Errno::__WASI_ERRNO_NOTSOCK)
    }
}

pub async fn sock_accept<M: Memory>(
    ctx: &mut WasiCtx,
    mem: &mut M,
    fd: __wasi_fd_t,
    ro_fd_ptr: WasmPtr<__wasi_fd_t>,
) -> Result<(), Errno> {
    #[cfg(feature = "serialize")]
    {
        use crate::snapshots::serialize::IoState;
        if let VFD::AsyncSocket(s) = ctx.get_mut_vfd(fd)? {
            if !s.state.nonblocking && s.state.local_addr.is_some() {
                ctx.io_state = IoState::Accept {
                    bind: s.state.local_addr.unwrap(),
                }
            }
        } else {
            return Err(Errno::__WASI_ERRNO_NOTSOCK);
        }
    }

    let sock_fd = ctx.get_mut_vfd(fd)?;
    if let VFD::AsyncSocket(s) = sock_fd {
        let cs = s.accept().await;
        #[cfg(feature = "serialize")]
        {
            use crate::snapshots::serialize::IoState;
            ctx.io_state = IoState::Empty;
        }
        let cs = cs?;
        let new_fd = ctx.insert_vfd(env::VFD::AsyncSocket(cs).into())?;
        mem.write_data(ro_fd_ptr, new_fd)?;
        Ok(())
    } else {
        Err(Errno::__WASI_ERRNO_NOTSOCK)
    }
}

pub async fn sock_connect<M: Memory>(
    ctx: &mut WasiCtx,
    mem: &mut M,
    fd: __wasi_fd_t,
    addr_ptr: WasmPtr<__wasi_address_t>,
    port: u32,
) -> Result<(), Errno> {
    let ip = parse_wasi_ip(mem, addr_ptr)?;
    let addr = SocketAddr::new(ip, port as u16);

    let sock_fd = ctx.get_mut_vfd(fd)?;
    if let VFD::AsyncSocket(s) = sock_fd {
        s.connect(addr).await?;
        Ok(())
    } else {
        Err(Errno::__WASI_ERRNO_NOTSOCK)
    }
}

pub async fn sock_recv<M: Memory>(
    ctx: &mut WasiCtx,
    mem: &mut M,
    fd: __wasi_fd_t,
    buf_ptr: WasmPtr<__wasi_iovec_t>,
    buf_len: __wasi_size_t,
    flags: __wasi_riflags_t::Type,
    ro_data_len_ptr: WasmPtr<__wasi_size_t>,
    ro_flags_ptr: WasmPtr<__wasi_roflags_t::Type>,
) -> Result<(), Errno> {
    let sock_fd = ctx.get_mut_vfd(fd)?;
    if let VFD::AsyncSocket(s) = sock_fd {
        let mut iovec = mem.mut_iovec(buf_ptr, buf_len)?;
        let mut native_flags = 0;

        if flags & __wasi_riflags_t::__WASI_RIFLAGS_RECV_PEEK > 0 {
            native_flags |= libc::MSG_PEEK;
        }
        if flags & __wasi_riflags_t::__WASI_RIFLAGS_RECV_WAITALL > 0 {
            native_flags |= libc::MSG_WAITALL;
        }

        let (n, trunc) = s.recv(&mut iovec, native_flags).await?;
        if trunc {
            mem.write_data(
                ro_flags_ptr,
                __wasi_roflags_t::__WASI_ROFLAGS_RECV_DATA_TRUNCATED,
            )?;
        }

        mem.write_data(ro_data_len_ptr, (n as u32).to_le())?;
        Ok(())
    } else {
        Err(Errno::__WASI_ERRNO_NOTSOCK)
    }
}

pub async fn sock_recv_from<M: Memory>(
    ctx: &mut WasiCtx,
    mem: &mut M,
    fd: __wasi_fd_t,
    buf_ptr: WasmPtr<__wasi_iovec_t>,
    buf_len: __wasi_size_t,
    wasi_addr_ptr: WasmPtr<__wasi_address_t>,
    flags: __wasi_riflags_t::Type,
    port_ptr: WasmPtr<u32>,
    ro_data_len_ptr: WasmPtr<__wasi_size_t>,
    ro_flags_ptr: WasmPtr<__wasi_roflags_t::Type>,
) -> Result<(), Errno> {
    let sock_fd = ctx.get_mut_vfd(fd)?;
    if let VFD::AsyncSocket(s) = sock_fd {
        let wasi_addr = *(mem.mut_data(wasi_addr_ptr)?);
        if wasi_addr.buf_len < 128 {
            return Err(Errno::__WASI_ERRNO_INVAL);
        }

        let mut iovec = mem.mut_iovec(buf_ptr, buf_len)?;
        let mut native_flags = 0;

        if flags & __wasi_riflags_t::__WASI_RIFLAGS_RECV_PEEK > 0 {
            native_flags |= libc::MSG_PEEK;
        }
        if flags & __wasi_riflags_t::__WASI_RIFLAGS_RECV_WAITALL > 0 {
            native_flags |= libc::MSG_WAITALL;
        }

        let (n, trunc, addr) = s.recv_from(&mut iovec, native_flags).await?;

        match addr {
            Some(SocketAddr::V4(addrv4)) => {
                let family_ptr = WasmPtr::<u16>::from(wasi_addr.buf as usize);
                let wasi_addr_buf_ptr = WasmPtr::<u8>::from(2 + wasi_addr.buf as usize);
                let wasi_addr_buf = mem.mut_slice(wasi_addr_buf_ptr, 4)?;
                wasi_addr_buf.copy_from_slice(&addrv4.ip().octets());

                mem.write_data(
                    family_ptr,
                    __wasi_address_family_t::__WASI_ADDRESS_FAMILY_INET4 as u16,
                )?;

                mem.write_data(port_ptr, (addrv4.port() as u32).to_le())?;
            }
            Some(SocketAddr::V6(addrv6)) => {
                let family_ptr = WasmPtr::<u16>::from(wasi_addr.buf as usize);
                let wasi_addr_buf_ptr = WasmPtr::<u8>::from(2 + wasi_addr.buf as usize);
                let wasi_addr_buf = mem.mut_slice(wasi_addr_buf_ptr, 16)?;
                wasi_addr_buf.copy_from_slice(&addrv6.ip().octets());
                mem.write_data(
                    family_ptr,
                    __wasi_address_family_t::__WASI_ADDRESS_FAMILY_INET6 as u16,
                )?;
                mem.write_data(port_ptr, (addrv6.port() as u32).to_le())?;
            }
            None => {}
        };

        if trunc {
            mem.write_data(
                ro_flags_ptr,
                __wasi_roflags_t::__WASI_ROFLAGS_RECV_DATA_TRUNCATED,
            )?;
        }

        mem.write_data(ro_data_len_ptr, (n as u32).to_le())?;
        Ok(())
    } else {
        Err(Errno::__WASI_ERRNO_NOTSOCK)
    }
}

pub async fn sock_send<M: Memory>(
    ctx: &mut WasiCtx,
    mem: &mut M,
    fd: __wasi_fd_t,
    buf_ptr: WasmPtr<__wasi_ciovec_t>,
    buf_len: __wasi_size_t,
    _flags: __wasi_siflags_t,
    send_len_ptr: WasmPtr<__wasi_size_t>,
) -> Result<(), Errno> {
    let sock_fd = ctx.get_mut_vfd(fd)?;
    if let VFD::AsyncSocket(s) = sock_fd {
        let iovec = mem.get_iovec(buf_ptr, buf_len)?;
        let n = s.send(&iovec, MSG_NOSIGNAL).await?;
        mem.write_data(send_len_ptr, (n as u32).to_le())?;
        Ok(())
    } else {
        Err(Errno::__WASI_ERRNO_NOTSOCK)
    }
}

pub async fn sock_send_to<M: Memory>(
    ctx: &mut WasiCtx,
    mem: &mut M,
    fd: __wasi_fd_t,
    buf_ptr: WasmPtr<__wasi_ciovec_t>,
    buf_len: __wasi_size_t,
    wasi_addr_ptr: WasmPtr<__wasi_address_t>,
    port: u32,
    _flags: __wasi_siflags_t,
    send_len_ptr: WasmPtr<__wasi_size_t>,
) -> Result<(), Errno> {
    let sock_fd = ctx.get_mut_vfd(fd)?;
    if let VFD::AsyncSocket(s) = sock_fd {
        let ip = parse_wasi_ip(mem, wasi_addr_ptr)?;
        let addr = SocketAddr::new(ip, port as u16);
        let iovec = mem.get_iovec(buf_ptr, buf_len)?;

        let n = s.send_to(&iovec, addr, MSG_NOSIGNAL).await?;
        mem.write_data(send_len_ptr, (n as u32).to_le())?;
        Ok(())
    } else {
        Err(Errno::__WASI_ERRNO_NOTSOCK)
    }
}

pub fn sock_shutdown<M: Memory>(
    ctx: &mut WasiCtx,
    _mem: &mut M,
    fd: __wasi_fd_t,
    how: __wasi_sdflags_t::Type,
) -> Result<(), Errno> {
    use std::net::Shutdown;
    let sock_fd = ctx.get_mut_vfd(fd)?;
    if let VFD::AsyncSocket(s) = sock_fd {
        const BOTH: __wasi_sdflags_t::Type =
            __wasi_sdflags_t::__WASI_SDFLAGS_WR | __wasi_sdflags_t::__WASI_SDFLAGS_RD;

        let how = match how {
            __wasi_sdflags_t::__WASI_SDFLAGS_RD => Shutdown::Read,
            __wasi_sdflags_t::__WASI_SDFLAGS_WR => Shutdown::Write,
            BOTH => Shutdown::Both,
            _ => return Err(Errno::__WASI_ERRNO_INVAL),
        };

        s.shutdown(how)?;
        Ok(())
    } else {
        Err(Errno::__WASI_ERRNO_NOTSOCK)
    }
}

pub fn sock_getpeeraddr<M: Memory>(
    ctx: &mut WasiCtx,
    mem: &mut M,
    fd: __wasi_fd_t,
    wasi_addr_ptr: WasmPtr<__wasi_address_t>,
    addr_type: WasmPtr<u32>,
    port_ptr: WasmPtr<u32>,
) -> Result<(), Errno> {
    let sock_fd = ctx.get_mut_vfd(fd)?;
    if let VFD::AsyncSocket(s) = sock_fd {
        let wasi_addr = *(mem.mut_data(wasi_addr_ptr)?);

        let addr = s.get_peer()?;

        let addr_len: u32 = match addr {
            SocketAddr::V4(addrv4) => {
                let wasi_addr_buf_ptr = WasmPtr::<u8>::from(wasi_addr.buf as usize);
                let wasi_addr_buf = mem.mut_slice(wasi_addr_buf_ptr, 4)?;
                wasi_addr_buf.copy_from_slice(&addrv4.ip().octets());
                mem.write_data(port_ptr, (addrv4.port() as u32).to_le())?;
                4
            }
            SocketAddr::V6(addrv6) => {
                let wasi_addr_buf_ptr = WasmPtr::<u8>::from(wasi_addr.buf as usize);
                let wasi_addr_buf = mem.mut_slice(wasi_addr_buf_ptr, 16)?;
                wasi_addr_buf.copy_from_slice(&addrv6.ip().octets());
                mem.write_data(port_ptr, (addrv6.port() as u32).to_le())?;
                16
            }
        };

        let wasi_addr = mem.mut_data(wasi_addr_ptr)?;
        wasi_addr.buf_len = addr_len.to_le();
        mem.write_data(addr_type, addr_len.to_le())?;

        Ok(())
    } else {
        Err(Errno::__WASI_ERRNO_NOTSOCK)
    }
}

pub fn sock_getlocaladdr<M: Memory>(
    ctx: &mut WasiCtx,
    mem: &mut M,
    fd: __wasi_fd_t,
    wasi_addr_ptr: WasmPtr<__wasi_address_t>,
    addr_type: WasmPtr<u32>,
    port_ptr: WasmPtr<u32>,
) -> Result<(), Errno> {
    let sock_fd = ctx.get_mut_vfd(fd)?;
    if let VFD::AsyncSocket(s) = sock_fd {
        let wasi_addr = *(mem.mut_data(wasi_addr_ptr)?);

        let addr = s.get_local()?;

        let addr_len: u32 = match addr {
            SocketAddr::V4(addrv4) => {
                let wasi_addr_buf_ptr = WasmPtr::<u8>::from(wasi_addr.buf as usize);
                let wasi_addr_buf = mem.mut_slice(wasi_addr_buf_ptr, 4)?;
                wasi_addr_buf.copy_from_slice(&addrv4.ip().octets());
                mem.write_data(port_ptr, (addrv4.port() as u32).to_le())?;
                4
            }
            SocketAddr::V6(addrv6) => {
                let wasi_addr_buf_ptr = WasmPtr::<u8>::from(wasi_addr.buf as usize);
                let wasi_addr_buf = mem.mut_slice(wasi_addr_buf_ptr, 16)?;
                wasi_addr_buf.copy_from_slice(&addrv6.ip().octets());
                mem.write_data(port_ptr, (addrv6.port() as u32).to_le())?;
                16
            }
        };

        let wasi_addr = mem.mut_data(wasi_addr_ptr)?;
        wasi_addr.buf_len = addr_len.to_le();
        mem.write_data(addr_type, addr_len.to_le())?;

        Ok(())
    } else {
        Err(Errno::__WASI_ERRNO_NOTSOCK)
    }
}

pub fn sock_getsockopt<M: Memory>(
    ctx: &mut WasiCtx,
    mem: &mut M,
    fd: __wasi_fd_t,
    level: __wasi_sock_opt_level_t::Type,
    name: __wasi_sock_opt_so_t::Type,
    flag: WasmPtr<i32>,
    flag_size_ptr: WasmPtr<__wasi_size_t>,
) -> Result<(), Errno> {
    let sock_fd = ctx.get_mut_vfd(fd)?;
    if let VFD::AsyncSocket(s) = sock_fd {
        let flag_size = *(mem.get_data(flag_size_ptr)?);
        if level != __wasi_sock_opt_level_t::__WASI_SOCK_OPT_LEVEL_SOL_SOCKET {
            return Err(Errno::__WASI_ERRNO_NOSYS);
        }
        let flag_val = match name {
            __wasi_sock_opt_so_t::__WASI_SOCK_OPT_SO_REUSEADDR => {
                if (flag_size as usize) != std::mem::size_of::<i32>() {
                    return Err(Errno::__WASI_ERRNO_INVAL);
                }
                s.get_so_reuseaddr() as i32
            }
            __wasi_sock_opt_so_t::__WASI_SOCK_OPT_SO_TYPE => {
                if (flag_size as usize) != std::mem::size_of::<i32>() {
                    return Err(Errno::__WASI_ERRNO_INVAL);
                }

                let (_, t) = s.get_so_type();
                let s = match t {
                    SocketType::Datagram => __wasi_sock_type_t::__WASI_SOCK_TYPE_SOCK_DGRAM,
                    SocketType::Stream => __wasi_sock_type_t::__WASI_SOCK_TYPE_SOCK_STREAM,
                } as i32;
                s
            }
            __wasi_sock_opt_so_t::__WASI_SOCK_OPT_SO_ERROR => {
                if let Some(e) = s.get_so_error()? {
                    Errno::from(e).0 as i32
                } else {
                    0
                }
            }
            __wasi_sock_opt_so_t::__WASI_SOCK_OPT_SO_DONTROUTE => {
                return Err(Errno::__WASI_ERRNO_NOSYS);
            }
            __wasi_sock_opt_so_t::__WASI_SOCK_OPT_SO_BROADCAST => {
                return Err(Errno::__WASI_ERRNO_NOSYS);
            }
            __wasi_sock_opt_so_t::__WASI_SOCK_OPT_SO_SNDBUF => {
                if (flag_size as usize) != std::mem::size_of::<i32>() {
                    return Err(Errno::__WASI_ERRNO_INVAL);
                }
                s.get_so_send_buf_size() as i32
            }
            __wasi_sock_opt_so_t::__WASI_SOCK_OPT_SO_RCVBUF => {
                if (flag_size as usize) != std::mem::size_of::<i32>() {
                    return Err(Errno::__WASI_ERRNO_INVAL);
                }
                s.get_so_recv_buf_size() as i32
            }
            __wasi_sock_opt_so_t::__WASI_SOCK_OPT_SO_KEEPALIVE => {
                return Err(Errno::__WASI_ERRNO_NOSYS);
            }
            __wasi_sock_opt_so_t::__WASI_SOCK_OPT_SO_OOBINLINE => {
                return Err(Errno::__WASI_ERRNO_NOSYS);
            }
            __wasi_sock_opt_so_t::__WASI_SOCK_OPT_SO_LINGER => {
                return Err(Errno::__WASI_ERRNO_NOSYS);
            }
            __wasi_sock_opt_so_t::__WASI_SOCK_OPT_SO_RCVLOWAT => {
                return Err(Errno::__WASI_ERRNO_NOSYS);
            }
            __wasi_sock_opt_so_t::__WASI_SOCK_OPT_SO_RCVTIMEO => {
                if (flag_size as usize) != std::mem::size_of::<__wasi_timeval>() {
                    return Err(Errno::__WASI_ERRNO_INVAL);
                }

                let timeval = if let Some(timeout) = s.get_so_recv_timeout() {
                    __wasi_timeval {
                        tv_sec: (timeout.as_secs() as i64).to_le(),
                        tv_usec: (timeout.subsec_nanos() as i64).to_le(),
                    }
                } else {
                    __wasi_timeval {
                        tv_sec: 0,
                        tv_usec: 0,
                    }
                };

                let offset = WasmPtr::<__wasi_timeval>::from(flag.0);
                mem.write_data(offset, timeval)?;

                return Ok(());
            }
            __wasi_sock_opt_so_t::__WASI_SOCK_OPT_SO_SNDTIMEO => {
                if (flag_size as usize) != std::mem::size_of::<__wasi_timeval>() {
                    return Err(Errno::__WASI_ERRNO_INVAL);
                }

                let timeval = if let Some(timeout) = s.get_so_send_timeout() {
                    __wasi_timeval {
                        tv_sec: (timeout.as_secs() as i64).to_le(),
                        tv_usec: (timeout.subsec_nanos() as i64).to_le(),
                    }
                } else {
                    __wasi_timeval {
                        tv_sec: 0,
                        tv_usec: 0,
                    }
                };

                let offset = WasmPtr::<__wasi_timeval>::from(flag.0);
                mem.write_data(offset, timeval)?;

                return Ok(());
            }
            __wasi_sock_opt_so_t::__WASI_SOCK_OPT_SO_ACCEPTCONN => {
                if (flag_size as usize) != std::mem::size_of::<i32>() {
                    return Err(Errno::__WASI_ERRNO_INVAL);
                }
                s.get_so_accept_conn()? as i32
            }
            __wasi_sock_opt_so_t::__WASI_SOCK_OPT_SO_BINDTODEVICE => {
                let device = s.device()?.unwrap_or_default();
                let offset = WasmPtr::<u8>::from(flag.0);
                let copy_len = device.len().min((flag_size.wrapping_sub(1)) as usize);
                if copy_len > 0 {
                    let wasm_buf = mem.mut_slice(offset, copy_len)?;
                    wasm_buf.copy_from_slice(&device[0..copy_len]);
                    mem.write_data(flag_size_ptr, (copy_len + 1) as u32)?;
                } else {
                    mem.write_data(flag_size_ptr, 0 as u32)?;
                }
                return Ok(());
            }
            _ => {
                return Err(Errno::__WASI_ERRNO_NOPROTOOPT);
            }
        };

        mem.write_data(flag, flag_val)?;

        Ok(())
    } else {
        Err(Errno::__WASI_ERRNO_NOTSOCK)
    }
}

pub fn sock_setsockopt<M: Memory>(
    ctx: &mut WasiCtx,
    mem: &mut M,
    fd: __wasi_fd_t,
    level: __wasi_sock_opt_level_t::Type,
    name: __wasi_sock_opt_so_t::Type,
    flag: WasmPtr<i32>,
    flag_size: __wasi_size_t,
) -> Result<(), Errno> {
    let sock_fd = ctx.get_mut_vfd(fd)?;
    if let VFD::AsyncSocket(s) = sock_fd {
        if level != __wasi_sock_opt_level_t::__WASI_SOCK_OPT_LEVEL_SOL_SOCKET {
            return Err(Errno::__WASI_ERRNO_NOSYS);
        }
        match name {
            __wasi_sock_opt_so_t::__WASI_SOCK_OPT_SO_REUSEADDR => {
                if (flag_size as usize) != std::mem::size_of::<i32>() {
                    return Err(Errno::__WASI_ERRNO_INVAL);
                }
                let flag_val = *(mem.get_data(flag)?) > 0;
                s.set_so_reuseaddr(flag_val)?;
            }
            __wasi_sock_opt_so_t::__WASI_SOCK_OPT_SO_TYPE => return Err(Errno::__WASI_ERRNO_FAULT),
            __wasi_sock_opt_so_t::__WASI_SOCK_OPT_SO_ERROR => {
                return Err(Errno::__WASI_ERRNO_FAULT)
            }
            __wasi_sock_opt_so_t::__WASI_SOCK_OPT_SO_DONTROUTE => {
                return Err(Errno::__WASI_ERRNO_NOSYS);
            }
            __wasi_sock_opt_so_t::__WASI_SOCK_OPT_SO_BROADCAST => {
                return Err(Errno::__WASI_ERRNO_NOSYS);
            }
            __wasi_sock_opt_so_t::__WASI_SOCK_OPT_SO_SNDBUF => {
                if (flag_size as usize) != std::mem::size_of::<i32>() {
                    return Err(Errno::__WASI_ERRNO_INVAL);
                }
                let flag_val = *(mem.get_data(flag)?);
                s.set_so_send_buf_size(flag_val as usize)?;
            }
            __wasi_sock_opt_so_t::__WASI_SOCK_OPT_SO_RCVBUF => {
                if (flag_size as usize) != std::mem::size_of::<i32>() {
                    return Err(Errno::__WASI_ERRNO_INVAL);
                }
                let flag_val = *(mem.get_data(flag)?);
                s.set_so_recv_buf_size(flag_val as usize)?;
            }
            __wasi_sock_opt_so_t::__WASI_SOCK_OPT_SO_KEEPALIVE => {
                return Err(Errno::__WASI_ERRNO_NOSYS);
            }
            __wasi_sock_opt_so_t::__WASI_SOCK_OPT_SO_OOBINLINE => {
                return Err(Errno::__WASI_ERRNO_NOSYS);
            }
            __wasi_sock_opt_so_t::__WASI_SOCK_OPT_SO_LINGER => {
                return Err(Errno::__WASI_ERRNO_NOSYS);
            }
            __wasi_sock_opt_so_t::__WASI_SOCK_OPT_SO_RCVLOWAT => {
                return Err(Errno::__WASI_ERRNO_NOSYS);
            }
            __wasi_sock_opt_so_t::__WASI_SOCK_OPT_SO_RCVTIMEO => {
                if (flag_size as usize) != std::mem::size_of::<__wasi_timeval>() {
                    return Err(Errno::__WASI_ERRNO_INVAL);
                }
                let offset = WasmPtr::<__wasi_timeval>::from(flag.0);
                let timeval = *(mem.get_data(offset)?);
                let (tv_sec, tv_usec) =
                    (i64::from_le(timeval.tv_sec), i64::from_le(timeval.tv_usec));

                let timeout = if tv_sec == 0 && tv_usec == 0 {
                    None
                } else {
                    Some(std::time::Duration::new(tv_sec as u64, tv_usec as u32))
                };

                s.set_so_recv_timeout(timeout)?;
            }
            __wasi_sock_opt_so_t::__WASI_SOCK_OPT_SO_SNDTIMEO => {
                if (flag_size as usize) != std::mem::size_of::<__wasi_timeval>() {
                    return Err(Errno::__WASI_ERRNO_INVAL);
                }
                let offset = WasmPtr::<__wasi_timeval>::from(flag.0);
                let timeval = *(mem.get_data(offset)?);
                let (tv_sec, tv_usec) =
                    (i64::from_le(timeval.tv_sec), i64::from_le(timeval.tv_usec));

                let timeout = if tv_sec == 0 && tv_usec == 0 {
                    None
                } else {
                    Some(std::time::Duration::new(tv_sec as u64, tv_usec as u32))
                };

                s.set_so_send_timeout(timeout)?;
            }
            __wasi_sock_opt_so_t::__WASI_SOCK_OPT_SO_ACCEPTCONN => {
                return Err(Errno::__WASI_ERRNO_FAULT);
            }
            __wasi_sock_opt_so_t::__WASI_SOCK_OPT_SO_BINDTODEVICE => {
                if flag_size == 0 {
                    s.bind_device(None)?;
                } else {
                    let buf_ptr = WasmPtr::<u8>::from(flag.0);
                    let wasm_buf = mem.get_slice(buf_ptr, flag_size as usize)?;
                    s.bind_device(Some(wasm_buf))?;
                }
                return Ok(());
            }
            _ => {
                return Err(Errno::__WASI_ERRNO_NOPROTOOPT);
            }
        };

        Ok(())
    } else {
        Err(Errno::__WASI_ERRNO_NOTSOCK)
    }
}

pub async fn sock_lookup_ip<M: Memory>(
    _ctx: &mut WasiCtx,
    mem: &mut M,
    host_name_ptr: WasmPtr<u8>,
    host_name_len: __wasi_size_t,
    lookup_type: __wasi_address_family_t::Type,
    addr_buf: WasmPtr<u8>,
    addr_buf_max_len: __wasi_size_t,
    raddr_num_ptr: WasmPtr<__wasi_size_t>,
) -> Result<(), Errno> {
    match lookup_type {
        __wasi_address_family_t::__WASI_ADDRESS_FAMILY_INET4 => {
            let host_name_buf = mem.get_slice(host_name_ptr, host_name_len as usize)?;
            let host_name =
                std::str::from_utf8(host_name_buf).or(Err(Errno::__WASI_ERRNO_ILSEQ))?;
            let addrs = tokio::net::lookup_host(format!("{host_name}:0")).await?;
            let write_buf = mem.mut_slice(addr_buf, addr_buf_max_len as usize)?;
            let mut i = 0;
            for addr in addrs {
                if let SocketAddr::V4(ip) = addr {
                    let buf = ip.ip().octets();
                    if let Some(w_buf) = write_buf.get_mut(i * 4..(i + 1) * 4) {
                        w_buf.copy_from_slice(&buf);
                        i += 1;
                    } else {
                        break;
                    }
                }
            }
            mem.write_data(raddr_num_ptr, i as u32)?;
            Ok(())
        }
        __wasi_address_family_t::__WASI_ADDRESS_FAMILY_INET6 => {
            let host_name_buf = mem.get_slice(host_name_ptr, host_name_len as usize)?;
            let host_name =
                std::str::from_utf8(host_name_buf).or(Err(Errno::__WASI_ERRNO_ILSEQ))?;
            let addrs = tokio::net::lookup_host(format!("{host_name}:0")).await?;
            let write_buf = mem.mut_slice(addr_buf, addr_buf_max_len as usize)?;
            let mut i = 0;
            for addr in addrs {
                if let SocketAddr::V6(ip) = addr {
                    let buf = ip.ip().octets();
                    if let Some(w_buf) = write_buf.get_mut(i * 16..(i + 1) * 16) {
                        w_buf.copy_from_slice(&buf);
                        i += 1;
                    } else {
                        break;
                    }
                }
            }
            mem.write_data(raddr_num_ptr, i as u32)?;
            Ok(())
        }
        _ => Err(Errno::__WASI_ERRNO_INVAL),
    }
}
