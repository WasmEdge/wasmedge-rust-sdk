use super::{
    common::{
        clock,
        error::Errno,
        memory::{Memory, WasmPtr},
        types::*,
    },
    env::{
        vfs::{self, FdFlags, WASIRights},
        AsyncVM,
    },
    WasiCtx,
};

#[cfg(all(unix, feature = "async_tokio"))]
pub mod async_poll;
#[cfg(all(unix, feature = "async_tokio"))]
pub mod async_socket;

pub fn args_get<M: Memory>(
    ctx: &WasiCtx,
    mem: &mut M,
    argv: WasmPtr<__wasi_size_t>,
    argv_buf: WasmPtr<u8>,
) -> Result<(), Errno> {
    log::trace!("args_get");

    let mut header_offset = 0;
    for (argv_index, arg) in ctx.args.iter().enumerate() {
        let arg_buf = mem.mut_data(argv + argv_index)?;
        *arg_buf = ((argv_buf.0 + header_offset) as u32).to_le();

        let arg_bytes = arg.as_bytes();
        let arg_buf = mem.mut_slice(argv_buf + header_offset, arg.len())?;
        arg_buf.copy_from_slice(arg_bytes);
        let ptr = mem.mut_data::<u8>(argv_buf + header_offset + arg.len())?;
        *ptr = 0u8;

        header_offset += arg.len() + 1;
    }
    Ok(())
}

pub fn args_sizes_get<M: Memory>(
    ctx: &WasiCtx,
    mem: &mut M,
    argc: WasmPtr<__wasi_size_t>,
    argv_buf_size: WasmPtr<__wasi_size_t>,
) -> Result<(), Errno> {
    log::trace!("args_sizes_get");

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
    ctx: &WasiCtx,
    mem: &mut M,
    environ: WasmPtr<__wasi_size_t>,
    environ_buf: WasmPtr<u8>,
) -> Result<(), Errno> {
    log::trace!("environ_get");

    let mut header_offset = 0;

    for (environ_index, env) in ctx.envs.iter().enumerate() {
        let environ_ptr = mem.mut_data(environ + environ_index)?;
        *environ_ptr = ((environ_buf.0 + header_offset) as u32).to_le();

        let env_bytes = env.as_bytes();
        let env_buf = mem.mut_slice(environ_buf + header_offset, env.len())?;
        env_buf.copy_from_slice(env_bytes);
        let ptr = mem.mut_data::<u8>(environ_buf + header_offset + env.len())?;
        *ptr = 0u8;

        header_offset += env.len() + 1;
    }
    Ok(())
}

pub fn environ_sizes_get<M: Memory>(
    ctx: &WasiCtx,
    mem: &mut M,
    environ_count: WasmPtr<__wasi_size_t>,
    environ_buf_size: WasmPtr<__wasi_size_t>,
) -> Result<(), Errno> {
    log::trace!("environ_sizes_get");

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
    log::trace!("clock_res_get");

    let resolution = clock::wasi_clock_res_get(clock_id)?;
    let resolution_ptr = mem.mut_data(resolution_ptr)?;
    *resolution_ptr = resolution.to_le();
    Ok(())
}

pub fn clock_time_get<M: Memory>(
    ctx: &WasiCtx,
    mem: &mut M,
    clock_id: __wasi_clockid_t::Type,
    precision: __wasi_timestamp_t,
    time_ptr: WasmPtr<__wasi_timestamp_t>,
) -> Result<(), Errno> {
    log::trace!("clock_time_get");

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
    log::trace!("random_get");

    let u8_buffer = mem.mut_slice(buf, buf_len as usize)?;
    getrandom::getrandom(u8_buffer).map_err(|_| Errno(__wasi_errno_t::__WASI_ERRNO_IO))
}

pub fn fd_prestat_get<M: Memory>(
    ctx: &mut WasiCtx,
    mem: &mut M,
    fd: __wasi_fd_t,
    prestat_ptr: WasmPtr<__wasi_prestat_t>,
) -> Result<(), Errno> {
    log::trace!("fd_prestat_get({fd})");

    let prestat = mem.mut_data(prestat_ptr)?;

    let pr_name_len = ctx.vfs.fd_preopen_get(fd as usize)?.as_bytes().len() as u32;

    prestat.tag = __wasi_preopentype_t::__WASI_PREOPENTYPE_DIR;
    prestat.u = __wasi_prestat_u_t {
        dir: __wasi_prestat_dir_t { pr_name_len },
    };
    Ok(())
}

pub fn fd_prestat_dir_name<M: Memory>(
    ctx: &mut WasiCtx,
    mem: &mut M,
    fd: __wasi_fd_t,
    path_buf_ptr: WasmPtr<u8>,
    path_max_len: __wasi_size_t,
) -> Result<(), Errno> {
    log::trace!("fd_prestat_dir_name({fd})");

    let path = ctx.vfs.fd_preopen_get(fd as usize)?;
    let path_bytes = path.as_bytes();
    let path_len = path_bytes.len();
    if path_len > path_max_len as usize {
        return Err(Errno::__WASI_ERRNO_NAMETOOLONG);
    }
    let path_buf = mem.mut_slice(path_buf_ptr, path_max_len as usize)?;
    path_buf.clone_from_slice(&path_bytes[0..path_max_len as usize]);
    Ok(())
}

pub fn fd_renumber<M: Memory>(
    ctx: &mut WasiCtx,
    _mem: &mut M,
    from: __wasi_fd_t,
    to: __wasi_fd_t,
) -> Result<(), Errno> {
    log::trace!("fd_renumber {from} {to}");

    ctx.vfs.fd_renumber(from as usize, to as usize)
}

pub fn fd_advise<M: Memory>(
    ctx: &mut WasiCtx,
    _mem: &mut M,
    fd: __wasi_fd_t,
    offset: __wasi_filesize_t,
    len: __wasi_filesize_t,
    advice: __wasi_advice_t::Type,
) -> Result<(), Errno> {
    log::trace!("fd_advise {fd}");

    ctx.vfs.fd_advise(fd as usize, offset, len, advice)
}

pub fn fd_allocate<M: Memory>(
    ctx: &mut WasiCtx,
    _mem: &mut M,
    fd: __wasi_fd_t,
    offset: __wasi_filesize_t,
    len: __wasi_filesize_t,
) -> Result<(), Errno> {
    log::trace!("fd_allocate {fd}");

    ctx.vfs.get_mut_file(fd as usize)?.fd_allocate(offset, len)
}

pub fn fd_close<M: Memory>(ctx: &mut WasiCtx, _mem: &mut M, fd: __wasi_fd_t) -> Result<(), Errno> {
    log::trace!("fd_close {fd}");

    ctx.vfs.fd_close(fd as usize)
}

pub fn fd_seek<M: Memory>(
    ctx: &mut WasiCtx,
    mem: &mut M,
    fd: __wasi_fd_t,
    offset: __wasi_filedelta_t,
    whence: __wasi_whence_t::Type,
    newoffset_ptr: WasmPtr<__wasi_filesize_t>,
) -> Result<(), Errno> {
    log::trace!("fd_seek {fd}");

    let fs = ctx.vfs.get_mut_file(fd as usize)?;
    let newoffset = mem.mut_data(newoffset_ptr)?;
    *newoffset = fs.fd_seek(offset, whence)?.to_le();
    Ok(())
}

pub fn fd_sync<M: Memory>(ctx: &mut WasiCtx, _mem: &mut M, fd: __wasi_fd_t) -> Result<(), Errno> {
    log::trace!("fd_sync {fd}");

    let fs = ctx.vfs.get_mut_file(fd as usize)?;
    fs.fd_sync()
}

pub fn fd_datasync<M: Memory>(
    ctx: &mut WasiCtx,
    _mem: &mut M,
    fd: __wasi_fd_t,
) -> Result<(), Errno> {
    log::trace!("fd_datasync {fd}");

    let fs = ctx.vfs.get_mut_file(fd as usize)?;
    fs.fd_datasync()
}

pub fn fd_tell<M: Memory>(
    ctx: &mut WasiCtx,
    mem: &mut M,
    fd: __wasi_fd_t,
    offset: WasmPtr<__wasi_filesize_t>,
) -> Result<(), Errno> {
    log::trace!("fd_tell {fd}");

    let fs = ctx.vfs.get_mut_file(fd as usize)?;
    mem.write_data(offset, fs.fd_tell()?)
}

pub fn fd_fdstat_get<M: Memory>(
    ctx: &mut WasiCtx,
    mem: &mut M,
    fd: __wasi_fd_t,
    buf_ptr: WasmPtr<__wasi_fdstat_t>,
) -> Result<(), Errno> {
    log::trace!("fd_fdstat_get {fd}");

    let fd_stat;
    #[cfg(all(unix, feature = "async_tokio"))]
    {
        if let Ok(s) = ctx.vfs.get_socket(fd as usize) {
            fd_stat = s.fd_fdstat_get()?;
        } else {
            fd_stat = ctx.vfs.get_inode(fd as usize)?.fd_fdstat_get()?;
        }
    }
    #[cfg(not(all(unix, feature = "async_tokio")))]
    {
        fd_stat = ctx.vfs.get_inode(fd as usize)?.fd_fdstat_get()?;
    }

    mem.write_data(buf_ptr, __wasi_fdstat_t::from(fd_stat))
}

pub fn fd_fdstat_set_flags<M: Memory>(
    ctx: &mut WasiCtx,
    _mem: &mut M,
    fd: __wasi_fd_t,
    flags: __wasi_fdflags_t::Type,
) -> Result<(), Errno> {
    log::trace!("fd_fdstat_set_flags {fd}");

    let fdflags = FdFlags::from_bits_truncate(flags);

    if let Ok(fs) = ctx.vfs.get_mut_file(fd as usize) {
        fs.fd_fdstat_set_flags(fdflags)?;
        return Ok(());
    }
    #[cfg(all(unix, feature = "async_tokio"))]
    if let Ok(s) = ctx.vfs.get_mut_socket(fd as usize) {
        s.set_nonblocking(fdflags.contains(FdFlags::NONBLOCK))?;
        return Ok(());
    }

    Err(Errno::__WASI_ERRNO_BADF)
}

pub fn fd_fdstat_set_rights<M: Memory>(
    ctx: &mut WasiCtx,
    _mem: &mut M,
    fd: __wasi_fd_t,
    fs_rights_base: __wasi_rights_t::Type,
    fs_rights_inheriting: __wasi_rights_t::Type,
) -> Result<(), Errno> {
    log::trace!("fd_fdstat_set_rights {fd}");

    let fs_rights_base = WASIRights::from_bits_truncate(fs_rights_base);
    let fs_rights_inheriting = WASIRights::from_bits_truncate(fs_rights_inheriting);
    ctx.vfs
        .get_mut_inode(fd as usize)?
        .fd_fdstat_set_rights(fs_rights_base, fs_rights_inheriting)
}

pub fn fd_filestat_get<M: Memory>(
    ctx: &mut WasiCtx,
    mem: &mut M,
    fd: __wasi_fd_t,
    buf: WasmPtr<__wasi_filestat_t>,
) -> Result<(), Errno> {
    log::trace!("fd_filestat_get {fd}");

    let filestat = ctx.vfs.get_inode(fd as usize)?.fd_filestat_get()?;
    mem.write_data(buf, __wasi_filestat_t::from(filestat))
}

pub fn fd_filestat_set_size<M: Memory>(
    ctx: &mut WasiCtx,
    _mem: &mut M,
    fd: __wasi_fd_t,
    st_size: __wasi_filesize_t,
) -> Result<(), Errno> {
    log::trace!("fd_filestat_set_size {fd}");

    let fs = ctx.vfs.get_mut_file(fd as usize)?;
    fs.fd_filestat_set_size(st_size)
}

pub fn fd_filestat_set_times<M: Memory>(
    ctx: &mut WasiCtx,
    _mem: &mut M,
    fd: __wasi_fd_t,
    st_atim: __wasi_timestamp_t,
    st_mtim: __wasi_timestamp_t,
    fst_flags: __wasi_fstflags_t::Type,
) -> Result<(), Errno> {
    log::trace!("fd_filestat_set_times {fd}");

    let inode = ctx.vfs.get_mut_inode(fd as usize)?;
    inode.fd_filestat_set_times(st_atim, st_mtim, fst_flags)
}

pub fn fd_read<M: Memory>(
    ctx: &mut WasiCtx,
    mem: &mut M,
    fd: __wasi_fd_t,
    iovs: WasmPtr<__wasi_iovec_t>,
    iovs_len: __wasi_size_t,
    nread: WasmPtr<__wasi_size_t>,
) -> Result<(), Errno> {
    log::trace!("fd_read {fd}");

    let fs = ctx.vfs.get_mut_file(fd as usize)?;
    let mut bufs = mem.mut_iovec(iovs, iovs_len)?;
    let n = fs.fd_read(&mut bufs)? as __wasi_size_t;
    mem.write_data(nread, n.to_le())
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
    log::trace!("fd_pread {fd}");

    let fs = ctx.vfs.get_mut_file(fd as usize)?;
    let mut bufs = mem.mut_iovec(iovs, iovs_len)?;
    let n = fs.fd_pread(&mut bufs, offset)? as __wasi_size_t;
    mem.write_data(nread, n.to_le())
}

pub fn fd_write<M: Memory>(
    ctx: &mut WasiCtx,
    mem: &mut M,
    fd: __wasi_fd_t,
    iovs: WasmPtr<__wasi_ciovec_t>,
    iovs_len: __wasi_size_t,
    nwritten: WasmPtr<__wasi_size_t>,
) -> Result<(), Errno> {
    log::trace!("fd_write {fd}");

    let fs = ctx.vfs.get_mut_file(fd as usize)?;
    let bufs = mem.get_iovec(iovs, iovs_len)?;
    let n = fs.fd_write(&bufs)? as __wasi_size_t;
    mem.write_data(nwritten, n.to_le())
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
    log::trace!("fd_pwrite {fd}");

    let fs = ctx.vfs.get_mut_file(fd as usize)?;
    let bufs = mem.get_iovec(iovs, iovs_len)?;
    let n = fs.fd_pwrite(&bufs, offset)? as __wasi_size_t;
    mem.write_data(nwritten, n.to_le())
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
    log::trace!("fd_readdir {fd}");

    let dir = ctx.vfs.get_dir(fd as usize)?;
    let buf = mem.mut_slice(buf, buf_len as usize)?;
    let bufused = dir.fd_readdir(cookie as usize, buf)? as __wasi_size_t;
    let bufused_ptr = mem.mut_data(bufused_ptr)?;
    *bufused_ptr = bufused.to_le();
    Ok(())
}

pub fn path_create_directory<M: Memory>(
    ctx: &mut WasiCtx,
    mem: &M,
    dirfd: __wasi_fd_t,
    path_ptr: WasmPtr<u8>,
    path_len: __wasi_size_t,
) -> Result<(), Errno> {
    log::trace!("path_create_directory");

    let path_buf = mem.get_slice(path_ptr, path_len as usize)?;
    let path_str = std::str::from_utf8(path_buf).or(Err(Errno::__WASI_ERRNO_ILSEQ))?;

    log::trace!("path_create_directory({dirfd} {path_str})");
    ctx.vfs.path_create_directory(dirfd as usize, path_str)
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
    let path_buf = mem.get_slice(path_ptr, path_len as usize)?;
    let path_str = std::str::from_utf8(path_buf).or(Err(Errno::__WASI_ERRNO_ILSEQ))?;

    log::trace!("path_filestat_get {dirfd} {path_str}");

    let flags = flags & __wasi_lookupflags_t::__WASI_LOOKUPFLAGS_SYMLINK_FOLLOW > 0;
    let file_stat = ctx.vfs.path_filestat_get(dirfd as usize, path_str, flags)?;
    let stat = mem.mut_data(file_stat_ptr)?;
    *stat = file_stat.into();
    Ok(())
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
    let path_buf = mem.get_slice(path, path_len as usize)?;
    let path = std::str::from_utf8(path_buf).or(Err(Errno::__WASI_ERRNO_ILSEQ))?;

    let oflags = vfs::OFlags::from_bits_truncate(o_flags);
    let fdflags = vfs::FdFlags::from_bits_truncate(fs_flags);
    let fs_rights_base = vfs::WASIRights::from_bits_truncate(fs_rights_base);
    let fs_rights_inheriting = vfs::WASIRights::from_bits_truncate(fs_rights_inheriting);

    let vfd = ctx.vfs.path_open(
        dirfd as usize,
        path,
        oflags,
        fs_rights_base,
        fs_rights_inheriting,
        fdflags,
    )?;

    mem.write_data(fd_ptr, vfd as i32)
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
    mem: &M,
    dirfd: __wasi_fd_t,
    path_ptr: WasmPtr<u8>,
    path_len: __wasi_size_t,
) -> Result<(), Errno> {
    let path_buf = mem.get_slice(path_ptr, path_len as usize)?;
    let path = std::str::from_utf8(path_buf).or(Err(Errno::__WASI_ERRNO_ILSEQ))?;
    log::trace!("path_remove_directory {dirfd} {path}");
    ctx.vfs.path_remove_directory(dirfd as usize, path)
}

pub fn path_rename<M: Memory>(
    ctx: &mut WasiCtx,
    mem: &M,
    old_fd: __wasi_fd_t,
    old_path: WasmPtr<u8>,
    old_path_len: __wasi_size_t,
    new_fd: __wasi_fd_t,
    new_path: WasmPtr<u8>,
    new_path_len: __wasi_size_t,
) -> Result<(), Errno> {
    let old_path = mem.get_slice(old_path, old_path_len as usize)?;
    let old_path = std::str::from_utf8(old_path).or(Err(Errno::__WASI_ERRNO_ILSEQ))?;

    let new_path = mem.get_slice(new_path, new_path_len as usize)?;
    let new_path = std::str::from_utf8(new_path).or(Err(Errno::__WASI_ERRNO_ILSEQ))?;

    log::trace!("path_rename {old_fd} {old_path} {new_fd} {new_path}");

    ctx.vfs
        .path_rename(old_fd as usize, old_path, new_fd as usize, new_path)
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
    mem: &M,
    dirfd: __wasi_fd_t,
    path_ptr: WasmPtr<u8>,
    path_len: __wasi_size_t,
) -> Result<(), Errno> {
    let path_buf = mem.get_slice(path_ptr, path_len as usize)?;
    let path = std::str::from_utf8(path_buf).or(Err(Errno::__WASI_ERRNO_ILSEQ))?;

    log::trace!("path_unlink_file {dirfd} {path}");
    ctx.vfs.path_unlink_file(dirfd as usize, path)
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
