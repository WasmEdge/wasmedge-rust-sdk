use super::common::clock;

use super::common::error::Errno;
use super::common::memory::{Memory, WasmPtr};
use super::common::types::*;
use super::env::{self, AsyncVM};
use super::WasiCtx;

#[cfg(all(unix, feature = "async_tokio"))]
pub mod async_poll;
#[cfg(all(unix, feature = "async_tokio"))]
pub mod async_socket;

pub fn args_get<M: Memory>(
    ctx: &mut WasiCtx,
    mem: &mut M,
    argv: WasmPtr<__wasi_size_t>,
    argv_buf: WasmPtr<u8>,
) -> Result<(), Errno> {
    let mut header_offset = 0;
    let mut argv_index = 0;
    for arg in &ctx.args {
        let arg_buf = mem.mut_data(argv + argv_index)?;
        *arg_buf = ((argv_buf.0 + header_offset) as u32).to_le();

        let arg_bytes = arg.as_bytes();
        let arg_buf = mem.mut_slice(argv_buf + header_offset, arg.len())?;
        arg_buf.copy_from_slice(arg_bytes);

        argv_index += 1;
        header_offset += arg.len() + 1;
    }
    Ok(())
}

pub fn args_sizes_get<M: Memory>(
    ctx: &mut WasiCtx,
    mem: &mut M,
    argc: WasmPtr<__wasi_size_t>,
    argv_buf_size: WasmPtr<__wasi_size_t>,
) -> Result<(), Errno> {
    let wasi_argc = ctx.args.len();
    debug_assert!(wasi_argc < __wasi_size_t::MAX as usize);
    let argc = mem.mut_data(argc)?;
    *argc = (wasi_argc as u32).to_le();

    let mut wasi_argv_buf_size = 0;
    for argv in &ctx.args {
        // add \0
        wasi_argv_buf_size += argv.len() + 1;
    }
    let argv_buf_size = mem.mut_data(argv_buf_size)?;
    *argv_buf_size = (wasi_argv_buf_size as u32).to_le();

    Ok(())
}

pub fn environ_get<M: Memory>(
    ctx: &mut WasiCtx,
    mem: &mut M,
    environ: WasmPtr<__wasi_size_t>,
    environ_buf: WasmPtr<u8>,
) -> Result<(), Errno> {
    let mut header_offset = 0;
    let mut environ_index = 0;

    for env in &ctx.envs {
        let environ_ptr = mem.mut_data(environ + environ_index)?;
        *environ_ptr = ((environ_buf.0 + header_offset) as u32).to_le();

        let env_bytes = env.as_bytes();
        let env_buf = mem.mut_slice(environ_buf + header_offset, env.len())?;
        env_buf.copy_from_slice(env_bytes);

        environ_index += 1;
        header_offset += env.len() + 1;
    }
    Ok(())
}

pub fn environ_sizes_get<M: Memory>(
    ctx: &mut WasiCtx,
    mem: &mut M,
    environ_count: WasmPtr<__wasi_size_t>,
    environ_buf_size: WasmPtr<__wasi_size_t>,
) -> Result<(), Errno> {
    let wasi_envs_len = ctx.envs.len();
    debug_assert!(
        wasi_envs_len < __wasi_size_t::MAX as usize,
        "wasi_envs_len({wasi_envs_len})"
    );
    let environ_count = mem.mut_data(environ_count)?;
    *environ_count = (wasi_envs_len as u32).to_le();

    let mut wasi_envs_buf_size = 0;
    for env in &ctx.envs {
        // add \0
        wasi_envs_buf_size += env.len() + 1;
    }
    let environ_buf_size = mem.mut_data(environ_buf_size)?;
    *environ_buf_size = (wasi_envs_buf_size as u32).to_le();

    Ok(())
}

pub fn clock_res_get<M: Memory>(
    _ctx: &mut WasiCtx,
    mem: &mut M,
    clock_id: __wasi_clockid_t::Type,
    resolution_ptr: WasmPtr<__wasi_timestamp_t>,
) -> Result<(), Errno> {
    let resolution = clock::wasi_clock_res_get(clock_id)?;
    let resolution_ptr = mem.mut_data(resolution_ptr)?;
    *resolution_ptr = resolution.to_le();
    Ok(())
}

pub fn clock_time_get<M: Memory>(
    ctx: &mut WasiCtx,
    mem: &mut M,
    clock_id: __wasi_clockid_t::Type,
    precision: __wasi_timestamp_t,
    time_ptr: WasmPtr<__wasi_timestamp_t>,
) -> Result<(), Errno> {
    let time = clock::wasi_clock_time_get(ctx, clock_id, precision)?;
    let time_ptr = mem.mut_data(time_ptr)?;
    *time_ptr = time.to_le();
    Ok(())
}

pub fn random_get<M: Memory>(
    _ctx: &mut WasiCtx,
    mem: &mut M,
    buf: WasmPtr<u8>,
    buf_len: __wasi_size_t,
) -> Result<(), Errno> {
    let u8_buffer = mem.mut_slice(buf, buf_len as usize)?;
    getrandom::getrandom(u8_buffer).map_err(|_| Errno(__wasi_errno_t::__WASI_ERRNO_IO))
}

pub fn fd_prestat_get<M: Memory>(
    ctx: &mut WasiCtx,
    mem: &mut M,
    fd: __wasi_fd_t,
    prestat_ptr: WasmPtr<__wasi_prestat_t>,
) -> Result<(), Errno> {
    use env::vfs::INode;
    use env::VFD;

    let prestat = mem.mut_data(prestat_ptr)?;
    let fd = ctx.get_mut_vfd(fd)?;
    match fd {
        VFD::Inode(INode::PreOpenDir(dir)) => {
            let path_str = dir.real_path.to_str().ok_or(Errno::__WASI_ERRNO_NOTSUP)?;
            let pr_name_len = path_str.as_bytes().len() as u32;

            prestat.tag = __wasi_preopentype_t::__WASI_PREOPENTYPE_DIR;
            prestat.u = __wasi_prestat_u_t {
                dir: __wasi_prestat_dir_t { pr_name_len },
            };
            Ok(())
        }
        _ => Err(Errno::__WASI_ERRNO_BADF),
    }
}

pub fn fd_prestat_dir_name<M: Memory>(
    ctx: &mut WasiCtx,
    mem: &mut M,
    fd: __wasi_fd_t,
    path_buf_ptr: WasmPtr<u8>,
    path_max_len: __wasi_size_t,
) -> Result<(), Errno> {
    use env::vfs::INode;
    use env::VFD;

    let fd = ctx.get_mut_vfd(fd)?;
    match fd {
        VFD::Inode(INode::PreOpenDir(dir)) => {
            let path_str = dir.real_path.to_str().ok_or(Errno::__WASI_ERRNO_NOTSUP)?;
            let path_bytes = path_str.as_bytes();
            let path_len = path_bytes.len();

            if path_len < path_max_len as usize {
                return Err(Errno::__WASI_ERRNO_NAMETOOLONG);
            }

            let path_buf = mem.mut_slice(path_buf_ptr, path_max_len as usize)?;

            path_buf.clone_from_slice(&path_bytes[0..path_max_len as usize]);

            Ok(())
        }
        _ => Err(Errno::__WASI_ERRNO_BADF),
    }
}

pub fn fd_renumber<M: Memory>(
    ctx: &mut WasiCtx,
    _mem: &mut M,
    from: __wasi_fd_t,
    to: __wasi_fd_t,
) -> Result<(), Errno> {
    ctx.renumber_vfd(from, to)
}

pub fn fd_advise<M: Memory>(
    ctx: &mut WasiCtx,
    _mem: &mut M,
    fd: __wasi_fd_t,
    _offset: __wasi_filesize_t,
    _len: __wasi_filesize_t,
    _advice: __wasi_advice_t::Type,
) -> Result<(), Errno> {
    use env::VFD;

    if let VFD::Inode(_) = ctx.get_mut_vfd(fd)? {
        Ok(())
    } else {
        Err(Errno::__WASI_ERRNO_BADF)
    }
}

pub fn fd_allocate<M: Memory>(
    ctx: &mut WasiCtx,
    _mem: &mut M,
    fd: __wasi_fd_t,
    offset: __wasi_filesize_t,
    len: __wasi_filesize_t,
) -> Result<(), Errno> {
    use super::env::vfs::INode;
    use super::env::VFD;

    if let VFD::Inode(INode::File(f)) = ctx.get_mut_vfd(fd)? {
        f.fd_allocate(offset, len)
    } else {
        Err(Errno::__WASI_ERRNO_BADF)
    }
}

pub fn fd_close<M: Memory>(ctx: &mut WasiCtx, _mem: &mut M, fd: __wasi_fd_t) -> Result<(), Errno> {
    ctx.remove_vfd(fd)?;
    Ok(())
}

pub fn fd_seek<M: Memory>(
    ctx: &mut WasiCtx,
    mem: &mut M,
    fd: __wasi_fd_t,
    offset: __wasi_filedelta_t,
    whence: __wasi_whence_t::Type,
    newoffset_ptr: WasmPtr<__wasi_filesize_t>,
) -> Result<(), Errno> {
    use env::vfs::INode;
    use env::VFD;
    let fd = ctx.get_mut_vfd(fd)?;
    match fd {
        VFD::Inode(INode::File(fs)) => {
            let newoffset = mem.mut_data(newoffset_ptr)?;
            *newoffset = fs.fd_seek(offset, whence)?.to_le();
            Ok(())
        }
        _ => Err(Errno::__WASI_ERRNO_SPIPE),
    }
}

pub fn fd_sync<M: Memory>(ctx: &mut WasiCtx, _mem: &mut M, fd: __wasi_fd_t) -> Result<(), Errno> {
    use env::vfs::INode;
    use env::VFD;
    let fd = ctx.get_mut_vfd(fd)?;
    match fd {
        VFD::Inode(INode::File(fs)) => fs.fd_sync(),
        VFD::Inode(INode::Stdin(fs)) => fs.fd_sync(),
        VFD::Inode(INode::Stdout(fs)) => fs.fd_sync(),
        VFD::Inode(INode::Stderr(fs)) => fs.fd_sync(),
        _ => Err(Errno::__WASI_ERRNO_BADF),
    }
}

pub fn fd_datasync<M: Memory>(
    ctx: &mut WasiCtx,
    _mem: &mut M,
    fd: __wasi_fd_t,
) -> Result<(), Errno> {
    use env::vfs::INode;
    use env::VFD;
    let fd = ctx.get_mut_vfd(fd)?;
    match fd {
        VFD::Inode(INode::File(fs)) => fs.fd_datasync(),
        VFD::Inode(INode::Stdin(fs)) => fs.fd_datasync(),
        VFD::Inode(INode::Stdout(fs)) => fs.fd_datasync(),
        VFD::Inode(INode::Stderr(fs)) => fs.fd_datasync(),
        _ => Err(Errno::__WASI_ERRNO_BADF),
    }
}

pub fn fd_tell<M: Memory>(
    ctx: &mut WasiCtx,
    mem: &mut M,
    fd: __wasi_fd_t,
    offset: WasmPtr<__wasi_filesize_t>,
) -> Result<(), Errno> {
    use env::vfs::INode;
    use env::VFD;
    let fd = ctx.get_mut_vfd(fd)?;
    match fd {
        VFD::Inode(INode::File(fs)) => mem.write_data(offset, fs.fd_tell()?),
        VFD::Inode(INode::Stdin(fs)) => mem.write_data(offset, fs.fd_tell()?),
        VFD::Inode(INode::Stdout(fs)) => mem.write_data(offset, fs.fd_tell()?),
        VFD::Inode(INode::Stderr(fs)) => mem.write_data(offset, fs.fd_tell()?),
        _ => Err(Errno::__WASI_ERRNO_SPIPE),
    }
}

pub fn fd_fdstat_get<M: Memory>(
    ctx: &mut WasiCtx,
    mem: &mut M,
    fd: __wasi_fd_t,
    buf_ptr: WasmPtr<__wasi_fdstat_t>,
) -> Result<(), Errno> {
    use env::vfs::INode;
    use env::VFD;

    let fd = ctx.get_mut_vfd(fd)?;
    match fd {
        VFD::Inode(INode::File(fs)) => {
            mem.write_data(buf_ptr, __wasi_fdstat_t::from(fs.fd_fdstat_get()?))
        }
        VFD::Inode(INode::Stdin(fs)) => {
            mem.write_data(buf_ptr, __wasi_fdstat_t::from(fs.fd_fdstat_get()?))
        }
        VFD::Inode(INode::Stdout(fs)) => {
            mem.write_data(buf_ptr, __wasi_fdstat_t::from(fs.fd_fdstat_get()?))
        }
        VFD::Inode(INode::Stderr(fs)) => {
            mem.write_data(buf_ptr, __wasi_fdstat_t::from(fs.fd_fdstat_get()?))
        }
        VFD::Inode(INode::Dir(dir_fs)) => {
            mem.write_data(buf_ptr, __wasi_fdstat_t::from(dir_fs.fd_fdstat_get()?))
        }
        #[cfg(all(unix, feature = "async_tokio"))]
        VFD::AsyncSocket(s) => mem.write_data(buf_ptr, __wasi_fdstat_t::from(s.fd_fdstat_get()?)),
        _ => Err(Errno::__WASI_ERRNO_BADF),
    }
}

pub fn fd_fdstat_set_flags<M: Memory>(
    ctx: &mut WasiCtx,
    _mem: &mut M,
    fd: __wasi_fd_t,
    flags: __wasi_fdflags_t::Type,
) -> Result<(), Errno> {
    use env::vfs::FdFlags;
    use env::vfs::INode;
    use env::VFD;

    let fdflags = FdFlags::from_bits_truncate(flags);

    let fd = ctx.get_mut_vfd(fd)?;
    match fd {
        VFD::Inode(INode::File(fs)) => fs.fd_fdstat_set_flags(fdflags),

        #[cfg(all(unix, feature = "async_tokio"))]
        VFD::AsyncSocket(s) => {
            s.set_nonblocking(fdflags.contains(FdFlags::NONBLOCK))?;
            Ok(())
        }

        _ => Err(Errno::__WASI_ERRNO_BADF),
    }
}

pub fn fd_fdstat_set_rights<M: Memory>(
    ctx: &mut WasiCtx,
    _mem: &mut M,
    fd: __wasi_fd_t,
    fs_rights_base: __wasi_rights_t::Type,
    fs_rights_inheriting: __wasi_rights_t::Type,
) -> Result<(), Errno> {
    use env::vfs::INode;
    use env::vfs::WASIRights;
    use env::VFD;

    let fd = ctx.get_mut_vfd(fd)?;
    let fs_rights_base = WASIRights::from_bits_truncate(fs_rights_base);
    let fs_rights_inheriting = WASIRights::from_bits_truncate(fs_rights_inheriting);

    match fd {
        VFD::Inode(INode::File(fs)) => {
            fs.fd_fdstat_set_rights(fs_rights_base, fs_rights_inheriting)
        }
        VFD::Inode(INode::Dir(dir)) => {
            dir.fd_fdstat_set_rights(fs_rights_base, fs_rights_inheriting)
        }
        _ => Err(Errno::__WASI_ERRNO_BADF),
    }
}

pub fn fd_filestat_get<M: Memory>(
    ctx: &mut WasiCtx,
    mem: &mut M,
    fd: __wasi_fd_t,
    buf: WasmPtr<__wasi_filestat_t>,
) -> Result<(), Errno> {
    use env::vfs::INode;
    use env::VFD;

    let fd = ctx.get_mut_vfd(fd)?;

    match fd {
        VFD::Inode(INode::File(fs)) => {
            let filestat = fs.fd_filestat_get()?;
            mem.write_data(buf, __wasi_filestat_t::from(filestat))
        }
        VFD::Inode(INode::Stdin(fs)) => {
            let filestat = fs.fd_filestat_get()?;
            mem.write_data(buf, __wasi_filestat_t::from(filestat))
        }
        VFD::Inode(INode::Stdout(fs)) => {
            let filestat = fs.fd_filestat_get()?;
            mem.write_data(buf, __wasi_filestat_t::from(filestat))
        }
        VFD::Inode(INode::Stderr(fs)) => {
            let filestat = fs.fd_filestat_get()?;
            mem.write_data(buf, __wasi_filestat_t::from(filestat))
        }
        VFD::Inode(INode::Dir(dir)) => {
            let filestat = dir.fd_filestat_get()?;
            mem.write_data(buf, __wasi_filestat_t::from(filestat))
        }
        _ => Err(Errno::__WASI_ERRNO_BADF),
    }
}

pub fn fd_filestat_set_size<M: Memory>(
    ctx: &mut WasiCtx,
    _mem: &mut M,
    fd: __wasi_fd_t,
    st_size: __wasi_filesize_t,
) -> Result<(), Errno> {
    use env::vfs::INode;
    use env::VFD;

    let fd = ctx.get_mut_vfd(fd)?;
    match fd {
        VFD::Inode(INode::File(fs)) => fs.fd_filestat_set_size(st_size),
        _ => Err(Errno::__WASI_ERRNO_BADF),
    }
}

pub fn fd_filestat_set_times<M: Memory>(
    ctx: &mut WasiCtx,
    _mem: &mut M,
    fd: __wasi_fd_t,
    st_atim: __wasi_timestamp_t,
    st_mtim: __wasi_timestamp_t,
    fst_flags: __wasi_fstflags_t::Type,
) -> Result<(), Errno> {
    use env::vfs::INode;
    use env::VFD;

    let fd = ctx.get_mut_vfd(fd)?;

    match fd {
        VFD::Inode(INode::File(fs)) => {
            fs.fd_filestat_set_times(st_atim, st_mtim, fst_flags)?;
            Ok(())
        }
        VFD::Inode(INode::Dir(dir)) => {
            dir.fd_filestat_set_times(st_atim, st_mtim, fst_flags)?;
            Ok(())
        }
        _ => Err(Errno::__WASI_ERRNO_BADF),
    }
}

pub fn fd_read<M: Memory>(
    ctx: &mut WasiCtx,
    mem: &mut M,
    fd: __wasi_fd_t,
    iovs: WasmPtr<__wasi_iovec_t>,
    iovs_len: __wasi_size_t,
    nread: WasmPtr<__wasi_size_t>,
) -> Result<(), Errno> {
    use env::vfs::INode;
    use env::VFD;

    let fd = ctx.get_mut_vfd(fd)?;

    match fd {
        VFD::Inode(INode::File(fs)) => {
            let mut bufs = mem.mut_iovec(iovs, iovs_len)?;
            let n = fs.fd_read(&mut bufs)? as __wasi_size_t;
            mem.write_data(nread, n.to_le())
        }
        VFD::Inode(INode::Stdin(fs)) => {
            let mut bufs = mem.mut_iovec(iovs, iovs_len)?;
            let n = fs.fd_read(&mut bufs)? as __wasi_size_t;
            mem.write_data(nread, n.to_le())
        }
        _ => Err(Errno::__WASI_ERRNO_BADF),
    }
}

pub fn fd_pread<M: Memory>(
    ctx: &mut WasiCtx,
    mem: &mut M,
    fd: __wasi_fd_t,
    iovs: WasmPtr<__wasi_iovec_t>,
    iovs_len: __wasi_size_t,
    offset: __wasi_filesize_t,
    nread: WasmPtr<__wasi_size_t>,
) -> Result<(), Errno> {
    use env::vfs::INode;
    use env::VFD;

    let fd = ctx.get_mut_vfd(fd)?;

    match fd {
        VFD::Inode(INode::File(fs)) => {
            let mut bufs = mem.mut_iovec(iovs, iovs_len)?;
            let n = fs.fd_pread(&mut bufs, offset)? as __wasi_size_t;
            mem.write_data(nread, n.to_le())
        }
        VFD::Inode(INode::Stdin(fs)) => {
            let mut bufs = mem.mut_iovec(iovs, iovs_len)?;
            let n = fs.fd_pread(&mut bufs, offset)? as __wasi_size_t;
            mem.write_data(nread, n.to_le())
        }
        _ => Err(Errno::__WASI_ERRNO_BADF),
    }
}

pub fn fd_write<M: Memory>(
    ctx: &mut WasiCtx,
    mem: &mut M,
    fd: __wasi_fd_t,
    iovs: WasmPtr<__wasi_ciovec_t>,
    iovs_len: __wasi_size_t,
    nwritten: WasmPtr<__wasi_size_t>,
) -> Result<(), Errno> {
    use env::vfs::INode;
    use env::VFD;

    let fd = ctx.get_mut_vfd(fd)?;

    match fd {
        VFD::Inode(INode::File(fs)) => {
            let bufs = mem.get_iovec(iovs, iovs_len)?;
            let n = fs.fd_write(&bufs)? as __wasi_size_t;
            mem.write_data(nwritten, n.to_le())
        }
        VFD::Inode(INode::Stdout(fs)) => {
            let bufs = mem.get_iovec(iovs, iovs_len)?;
            let n = fs.fd_write(&bufs)? as __wasi_size_t;
            mem.write_data(nwritten, n.to_le())
        }
        VFD::Inode(INode::Stderr(fs)) => {
            let bufs = mem.get_iovec(iovs, iovs_len)?;
            let n = fs.fd_write(&bufs)? as __wasi_size_t;
            mem.write_data(nwritten, n.to_le())
        }
        _ => Err(Errno::__WASI_ERRNO_BADF),
    }
}

pub fn fd_pwrite<M: Memory>(
    ctx: &mut WasiCtx,
    mem: &mut M,
    fd: __wasi_fd_t,
    iovs: WasmPtr<__wasi_ciovec_t>,
    iovs_len: __wasi_size_t,
    offset: __wasi_filesize_t,
    nwritten: WasmPtr<__wasi_size_t>,
) -> Result<(), Errno> {
    use env::vfs::INode;
    use env::VFD;

    let fd = ctx.get_mut_vfd(fd)?;

    match fd {
        VFD::Inode(INode::File(fs)) => {
            let bufs = mem.get_iovec(iovs, iovs_len)?;
            let n = fs.fd_pwrite(&bufs, offset)? as __wasi_size_t;
            mem.write_data(nwritten, n.to_le())
        }
        VFD::Inode(INode::Stdout(fs)) => {
            let bufs = mem.get_iovec(iovs, iovs_len)?;
            let n = fs.fd_pwrite(&bufs, offset)? as __wasi_size_t;
            mem.write_data(nwritten, n.to_le())
        }
        VFD::Inode(INode::Stderr(fs)) => {
            let bufs = mem.get_iovec(iovs, iovs_len)?;
            let n = fs.fd_pwrite(&bufs, offset)? as __wasi_size_t;
            mem.write_data(nwritten, n.to_le())
        }
        _ => Err(Errno::__WASI_ERRNO_BADF),
    }
}

pub fn fd_readdir<M: Memory>(
    ctx: &mut WasiCtx,
    mem: &mut M,
    fd: __wasi_fd_t,
    buf: WasmPtr<u8>,
    buf_len: __wasi_size_t,
    cookie: __wasi_dircookie_t,
    bufused_ptr: WasmPtr<__wasi_size_t>,
) -> Result<(), Errno> {
    use env::vfs::INode;
    use env::VFD;
    let fd = ctx.get_mut_vfd(fd)?;
    let buf = mem.mut_slice(buf, buf_len as usize)?;
    match fd {
        VFD::Inode(INode::PreOpenDir(dir)) => {
            let bufused = dir.fd_readdir(cookie as usize, buf)? as __wasi_size_t;
            let bufused_ptr = mem.mut_data(bufused_ptr)?;
            *bufused_ptr = bufused.to_le();
            Ok(())
        }
        VFD::Inode(INode::Dir(dir)) => {
            let bufused = dir.fd_readdir(cookie as usize, buf)? as __wasi_size_t;
            let bufused_ptr = mem.mut_data(bufused_ptr)?;
            *bufused_ptr = bufused.to_le();
            Ok(())
        }
        _ => Err(Errno::__WASI_ERRNO_NOTDIR),
    }
}

pub fn path_create_directory<M: Memory>(
    ctx: &mut WasiCtx,
    mem: &mut M,
    dirfd: __wasi_fd_t,
    path_ptr: WasmPtr<u8>,
    path_len: __wasi_size_t,
) -> Result<(), Errno> {
    use env::vfs::INode;
    use env::VFD;
    let fd = ctx.get_mut_vfd(dirfd)?;
    match fd {
        VFD::Inode(INode::PreOpenDir(dir)) => {
            let path_buf = mem.get_slice(path_ptr, path_len as usize)?;
            let path_str = std::str::from_utf8(path_buf).or(Err(Errno::__WASI_ERRNO_ILSEQ))?;
            dir.path_create_directory(path_str)?;
            Ok(())
        }
        VFD::Inode(INode::Dir(_)) => Err(Errno::__WASI_ERRNO_ACCES),
        _ => Err(Errno::__WASI_ERRNO_NOTDIR),
    }
}

pub fn path_filestat_get<M: Memory>(
    ctx: &mut WasiCtx,
    mem: &mut M,
    dirfd: __wasi_fd_t,
    flags: __wasi_lookupflags_t::Type,
    path_ptr: WasmPtr<u8>,
    path_len: __wasi_size_t,
    file_stat_ptr: WasmPtr<__wasi_filestat_t>,
) -> Result<(), Errno> {
    use env::vfs::INode;
    use env::VFD;
    let fd = ctx.get_mut_vfd(dirfd)?;
    match fd {
        VFD::Inode(INode::PreOpenDir(dir)) => {
            let path_buf = mem.get_slice(path_ptr, path_len as usize)?;
            let path_str = std::str::from_utf8(path_buf).or(Err(Errno::__WASI_ERRNO_ILSEQ))?;
            let flags = flags & __wasi_lookupflags_t::__WASI_LOOKUPFLAGS_SYMLINK_FOLLOW > 0;
            let file_stat = dir.path_filestat_get(path_str, flags)?;
            let stat = mem.mut_data(file_stat_ptr)?;
            *stat = file_stat.into();
            stat.dev = 3;
            Ok(())
        }
        VFD::Inode(INode::Dir(_)) => Err(Errno::__WASI_ERRNO_ACCES),
        _ => Err(Errno::__WASI_ERRNO_NOTDIR),
    }
}

pub fn path_filestat_set_times<M: Memory>(
    _ctx: &mut WasiCtx,
    _mem: &mut M,
    _dirfd: __wasi_fd_t,
    _flags: __wasi_lookupflags_t::Type,
    _path: WasmPtr<u8>,
    _path_len: __wasi_size_t,
    _st_atim: __wasi_timestamp_t,
    _st_mtim: __wasi_timestamp_t,
    _fst_flags: __wasi_fstflags_t::Type,
) -> Result<(), Errno> {
    Err(Errno::__WASI_ERRNO_NOSYS)
}

pub fn path_link<M: Memory>(
    _ctx: &mut WasiCtx,
    _mem: &mut M,
    _old_fd: __wasi_fd_t,
    _old_flags: __wasi_lookupflags_t::Type,
    _old_path: WasmPtr<u8>,
    _old_path_len: __wasi_size_t,
    _new_fd: __wasi_fd_t,
    _new_path: WasmPtr<u8>,
    _new_path_len: __wasi_size_t,
) -> Result<(), Errno> {
    Err(Errno::__WASI_ERRNO_NOSYS)
}

pub fn path_open<M: Memory>(
    ctx: &mut WasiCtx,
    mem: &mut M,
    dirfd: __wasi_fd_t,
    _dirflags: __wasi_lookupflags_t::Type,
    path: WasmPtr<u8>,
    path_len: __wasi_size_t,
    o_flags: __wasi_oflags_t::Type,
    fs_rights_base: __wasi_rights_t::Type,
    fs_rights_inheriting: __wasi_rights_t::Type,
    fs_flags: __wasi_fdflags_t::Type,
    fd_ptr: WasmPtr<__wasi_fd_t>,
) -> Result<(), Errno> {
    use env::vfs;
    use env::vfs::INode;
    use env::VFD;
    let vfd = ctx.get_mut_vfd(dirfd)?;
    match vfd {
        VFD::Inode(INode::PreOpenDir(dir)) => {
            let path_buf = mem.get_slice(path, path_len as usize)?;
            let path = std::str::from_utf8(path_buf).or(Err(Errno::__WASI_ERRNO_ILSEQ))?;

            let oflags = vfs::OFlags::from_bits_truncate(o_flags);
            let fdflags = vfs::FdFlags::from_bits_truncate(fs_flags);
            let fs_rights_base = vfs::WASIRights::from_bits_truncate(fs_rights_base);
            let fs_rights_inheriting = vfs::WASIRights::from_bits_truncate(fs_rights_inheriting);
            if oflags.contains(vfs::OFlags::DIRECTORY) {
                let dir = dir.path_open_dir(
                    &path,
                    oflags,
                    fs_rights_base,
                    fs_rights_inheriting,
                    fdflags,
                )?;

                let vfd = VFD::Inode(INode::Dir(dir));
                let fd_ptr = mem.mut_data(fd_ptr)?;
                *fd_ptr = ctx.insert_vfd(vfd)?;
            } else {
                let fs = dir.path_open_file(&path, oflags, fs_rights_base, fdflags)?;

                let vfd = VFD::Inode(INode::File(fs));
                let fd_ptr = mem.mut_data(fd_ptr)?;
                *fd_ptr = ctx.insert_vfd(vfd)?;
            }
            Ok(())
        }
        _ => Err(Errno::__WASI_ERRNO_NOENT),
    }
}

pub fn path_readlink<M: Memory>(
    _ctx: &mut WasiCtx,
    _mem: &mut M,
    _dir_fd: __wasi_fd_t,
    _path: WasmPtr<u8>,
    _path_len: __wasi_size_t,
    _buf: WasmPtr<u8>,
    _buf_len: __wasi_size_t,
    _buf_used: WasmPtr<__wasi_size_t>,
) -> Result<(), Errno> {
    Err(Errno::__WASI_ERRNO_NOSYS)
}

pub fn path_remove_directory<M: Memory>(
    ctx: &mut WasiCtx,
    mem: &mut M,
    dirfd: __wasi_fd_t,
    path_ptr: WasmPtr<u8>,
    path_len: __wasi_size_t,
) -> Result<(), Errno> {
    use env::vfs::INode;
    use env::VFD;
    let fd = ctx.get_mut_vfd(dirfd)?;
    match fd {
        VFD::Inode(INode::PreOpenDir(dir)) => {
            let path_buf = mem.get_slice(path_ptr, path_len as usize)?;
            let path = std::str::from_utf8(path_buf).or(Err(Errno::__WASI_ERRNO_ILSEQ))?;
            dir.path_remove_directory(path)?;
            Ok(())
        }
        VFD::Inode(INode::Dir(_)) => Err(Errno::__WASI_ERRNO_ACCES),
        _ => Err(Errno::__WASI_ERRNO_NOTDIR),
    }
}

pub fn path_rename<M: Memory>(
    ctx: &mut WasiCtx,
    mem: &mut M,
    old_fd: __wasi_fd_t,
    old_path: WasmPtr<u8>,
    old_path_len: __wasi_size_t,
    new_fd: __wasi_fd_t,
    new_path: WasmPtr<u8>,
    new_path_len: __wasi_size_t,
) -> Result<(), Errno> {
    use env::vfs::INode;
    use env::VFD;
    let old_fd = ctx.get_vfd(old_fd)?;
    let new_fd = ctx.get_vfd(new_fd)?;

    match (old_fd, new_fd) {
        (VFD::Inode(INode::PreOpenDir(old_dir)), VFD::Inode(INode::PreOpenDir(new_dir))) => {
            let old_path = mem.get_slice(old_path, old_path_len as usize)?;
            let old_path = std::str::from_utf8(old_path).or(Err(Errno::__WASI_ERRNO_ILSEQ))?;

            let new_path = mem.get_slice(new_path, new_path_len as usize)?;
            let new_path = std::str::from_utf8(new_path).or(Err(Errno::__WASI_ERRNO_ILSEQ))?;

            let old_file_path = old_dir.get_absolutize_path(&old_path)?;
            let new_file_path = new_dir.get_absolutize_path(&new_path)?;

            std::fs::rename(old_file_path, new_file_path)?;
            Ok(())
        }
        (VFD::Inode(INode::Dir(_)), _) | (_, VFD::Inode(INode::Dir(_))) => {
            Err(Errno::__WASI_ERRNO_ACCES)
        }
        _ => Err(Errno::__WASI_ERRNO_NOTDIR),
    }
}

pub fn path_symlink<M: Memory>(
    _ctx: &mut WasiCtx,
    _mem: &mut M,
    _old_path: WasmPtr<u8>,
    _old_path_len: __wasi_size_t,
    _fd: __wasi_fd_t,
    _new_path: WasmPtr<u8>,
    _new_path_len: __wasi_size_t,
) -> Result<(), Errno> {
    Err(Errno::__WASI_ERRNO_NOSYS)
}

pub fn path_unlink_file<M: Memory>(
    ctx: &mut WasiCtx,
    mem: &mut M,
    dirfd: __wasi_fd_t,
    path_ptr: WasmPtr<u8>,
    path_len: __wasi_size_t,
) -> Result<(), Errno> {
    use env::vfs::INode;
    use env::VFD;
    let fd = ctx.get_mut_vfd(dirfd)?;
    match fd {
        VFD::Inode(INode::PreOpenDir(dir)) => {
            let path_buf = mem.get_slice(path_ptr, path_len as usize)?;
            let path = std::str::from_utf8(path_buf).or(Err(Errno::__WASI_ERRNO_ILSEQ))?;
            dir.path_unlink_file(path)?;
            Ok(())
        }
        VFD::Inode(INode::Dir(_)) => Err(Errno::__WASI_ERRNO_ACCES),
        _ => Err(Errno::__WASI_ERRNO_NOTDIR),
    }
}

pub fn proc_exit<M: Memory>(ctx: &mut WasiCtx, _mem: &mut M, code: __wasi_exitcode_t) {
    ctx.exit_code = u32::from_le(code)
}

pub fn proc_raise<M: Memory>(
    _ctx: &mut WasiCtx,
    _mem: &mut M,
    _sig: __wasi_signal_t::Type,
) -> Result<(), Errno> {
    Err(Errno::__WASI_ERRNO_NOSYS)
}

pub fn sched_yield<VM: AsyncVM>(_ctx: &mut WasiCtx, vm: &mut VM) -> Result<(), Errno> {
    vm.yield_now()
}
