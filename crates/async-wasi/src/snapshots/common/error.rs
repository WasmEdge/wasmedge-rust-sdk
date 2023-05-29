use std::{fmt::Debug, io::ErrorKind};

pub use super::types::__wasi_errno_t as wasi_errno;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub struct Errno(pub wasi_errno::Type);

impl Default for Errno {
    fn default() -> Self {
        Errno::__WASI_ERRNO_SUCCESS
    }
}

impl Errno {
    pub const __WASI_ERRNO_SUCCESS: Errno = Errno(0);
    pub const __WASI_ERRNO_2BIG: Errno = Errno(1);
    pub const __WASI_ERRNO_ACCES: Errno = Errno(2);
    pub const __WASI_ERRNO_ADDRINUSE: Errno = Errno(3);
    pub const __WASI_ERRNO_ADDRNOTAVAIL: Errno = Errno(4);
    pub const __WASI_ERRNO_AFNOSUPPORT: Errno = Errno(5);
    pub const __WASI_ERRNO_AGAIN: Errno = Errno(6);
    pub const __WASI_ERRNO_ALREADY: Errno = Errno(7);
    pub const __WASI_ERRNO_BADF: Errno = Errno(8);
    pub const __WASI_ERRNO_BADMSG: Errno = Errno(9);
    pub const __WASI_ERRNO_BUSY: Errno = Errno(10);
    pub const __WASI_ERRNO_CANCELED: Errno = Errno(11);
    pub const __WASI_ERRNO_CHILD: Errno = Errno(12);
    pub const __WASI_ERRNO_CONNABORTED: Errno = Errno(13);
    pub const __WASI_ERRNO_CONNREFUSED: Errno = Errno(14);
    pub const __WASI_ERRNO_CONNRESET: Errno = Errno(15);
    pub const __WASI_ERRNO_DEADLK: Errno = Errno(16);
    pub const __WASI_ERRNO_DESTADDRREQ: Errno = Errno(17);
    pub const __WASI_ERRNO_DOM: Errno = Errno(18);
    pub const __WASI_ERRNO_DQUOT: Errno = Errno(19);
    pub const __WASI_ERRNO_EXIST: Errno = Errno(20);
    pub const __WASI_ERRNO_FAULT: Errno = Errno(21);
    pub const __WASI_ERRNO_FBIG: Errno = Errno(22);
    pub const __WASI_ERRNO_HOSTUNREACH: Errno = Errno(23);
    pub const __WASI_ERRNO_IDRM: Errno = Errno(24);
    pub const __WASI_ERRNO_ILSEQ: Errno = Errno(25);
    pub const __WASI_ERRNO_INPROGRESS: Errno = Errno(26);
    pub const __WASI_ERRNO_INTR: Errno = Errno(27);
    pub const __WASI_ERRNO_INVAL: Errno = Errno(28);
    pub const __WASI_ERRNO_IO: Errno = Errno(29);
    pub const __WASI_ERRNO_ISCONN: Errno = Errno(30);
    pub const __WASI_ERRNO_ISDIR: Errno = Errno(31);
    pub const __WASI_ERRNO_LOOP: Errno = Errno(32);
    pub const __WASI_ERRNO_MFILE: Errno = Errno(33);
    pub const __WASI_ERRNO_MLINK: Errno = Errno(34);
    pub const __WASI_ERRNO_MSGSIZE: Errno = Errno(35);
    pub const __WASI_ERRNO_MULTIHOP: Errno = Errno(36);
    pub const __WASI_ERRNO_NAMETOOLONG: Errno = Errno(37);
    pub const __WASI_ERRNO_NETDOWN: Errno = Errno(38);
    pub const __WASI_ERRNO_NETRESET: Errno = Errno(39);
    pub const __WASI_ERRNO_NETUNREACH: Errno = Errno(40);
    pub const __WASI_ERRNO_NFILE: Errno = Errno(41);
    pub const __WASI_ERRNO_NOBUFS: Errno = Errno(42);
    pub const __WASI_ERRNO_NODEV: Errno = Errno(43);
    pub const __WASI_ERRNO_NOENT: Errno = Errno(44);
    pub const __WASI_ERRNO_NOEXEC: Errno = Errno(45);
    pub const __WASI_ERRNO_NOLCK: Errno = Errno(46);
    pub const __WASI_ERRNO_NOLINK: Errno = Errno(47);
    pub const __WASI_ERRNO_NOMEM: Errno = Errno(48);
    pub const __WASI_ERRNO_NOMSG: Errno = Errno(49);
    pub const __WASI_ERRNO_NOPROTOOPT: Errno = Errno(50);
    pub const __WASI_ERRNO_NOSPC: Errno = Errno(51);
    pub const __WASI_ERRNO_NOSYS: Errno = Errno(52);
    pub const __WASI_ERRNO_NOTCONN: Errno = Errno(53);
    pub const __WASI_ERRNO_NOTDIR: Errno = Errno(54);
    pub const __WASI_ERRNO_NOTEMPTY: Errno = Errno(55);
    pub const __WASI_ERRNO_NOTRECOVERABLE: Errno = Errno(56);
    pub const __WASI_ERRNO_NOTSOCK: Errno = Errno(57);
    pub const __WASI_ERRNO_NOTSUP: Errno = Errno(58);
    pub const __WASI_ERRNO_NOTTY: Errno = Errno(59);
    pub const __WASI_ERRNO_NXIO: Errno = Errno(60);
    pub const __WASI_ERRNO_OVERFLOW: Errno = Errno(61);
    pub const __WASI_ERRNO_OWNERDEAD: Errno = Errno(62);
    pub const __WASI_ERRNO_PERM: Errno = Errno(63);
    pub const __WASI_ERRNO_PIPE: Errno = Errno(64);
    pub const __WASI_ERRNO_PROTO: Errno = Errno(65);
    pub const __WASI_ERRNO_PROTONOSUPPORT: Errno = Errno(66);
    pub const __WASI_ERRNO_PROTOTYPE: Errno = Errno(67);
    pub const __WASI_ERRNO_RANGE: Errno = Errno(68);
    pub const __WASI_ERRNO_ROFS: Errno = Errno(69);
    pub const __WASI_ERRNO_SPIPE: Errno = Errno(70);
    pub const __WASI_ERRNO_SRCH: Errno = Errno(71);
    pub const __WASI_ERRNO_STALE: Errno = Errno(72);
    pub const __WASI_ERRNO_TIMEDOUT: Errno = Errno(73);
    pub const __WASI_ERRNO_TXTBSY: Errno = Errno(74);
    pub const __WASI_ERRNO_XDEV: Errno = Errno(75);
    pub const __WASI_ERRNO_NOTCAPABLE: Errno = Errno(76);
    pub const __WASI_ERRNO_AIADDRFAMILY: Errno = Errno(77);
    pub const __WASI_ERRNO_AIAGAIN: Errno = Errno(78);
    pub const __WASI_ERRNO_AIBADFLAG: Errno = Errno(79);
    pub const __WASI_ERRNO_AIFAIL: Errno = Errno(80);
    pub const __WASI_ERRNO_AIFAMILY: Errno = Errno(81);
    pub const __WASI_ERRNO_AIMEMORY: Errno = Errno(82);
    pub const __WASI_ERRNO_AINODATA: Errno = Errno(83);
    pub const __WASI_ERRNO_AINONAME: Errno = Errno(84);
    pub const __WASI_ERRNO_AISERVICE: Errno = Errno(85);
    pub const __WASI_ERRNO_AISOCKTYPE: Errno = Errno(86);
    pub const __WASI_ERRNO_AISYSTEM: Errno = Errno(87);
}

impl From<wasi_errno::Type> for Errno {
    fn from(e: wasi_errno::Type) -> Self {
        Errno(e)
    }
}

impl From<ErrorKind> for Errno {
    fn from(e: ErrorKind) -> Self {
        match e {
            ErrorKind::NotFound => Errno::__WASI_ERRNO_NOENT,
            ErrorKind::PermissionDenied => Errno::__WASI_ERRNO_PERM,
            ErrorKind::ConnectionRefused => Errno::__WASI_ERRNO_CONNREFUSED,
            ErrorKind::ConnectionReset => Errno::__WASI_ERRNO_CONNRESET,
            ErrorKind::ConnectionAborted => Errno::__WASI_ERRNO_CONNABORTED,
            ErrorKind::NotConnected => Errno::__WASI_ERRNO_NOTCONN,
            ErrorKind::AddrInUse => Errno::__WASI_ERRNO_ADDRINUSE,
            ErrorKind::AddrNotAvailable => Errno::__WASI_ERRNO_ADDRNOTAVAIL,
            ErrorKind::BrokenPipe => Errno::__WASI_ERRNO_PIPE,
            ErrorKind::AlreadyExists => Errno::__WASI_ERRNO_EXIST,
            ErrorKind::WouldBlock => Errno::__WASI_ERRNO_AGAIN,
            ErrorKind::InvalidInput => Errno::__WASI_ERRNO_INVAL,
            ErrorKind::InvalidData => Errno::__WASI_ERRNO_IO,
            ErrorKind::TimedOut => Errno::__WASI_ERRNO_TIMEDOUT,
            ErrorKind::WriteZero => Errno::__WASI_ERRNO_IO,
            ErrorKind::Interrupted => Errno::__WASI_ERRNO_INTR,
            ErrorKind::UnexpectedEof => Errno::__WASI_ERRNO_IO,
            ErrorKind::Unsupported => Errno::__WASI_ERRNO_NOTSUP,
            ErrorKind::OutOfMemory => Errno::__WASI_ERRNO_NOMEM,
            _ => Errno::__WASI_ERRNO_IO,
        }
    }
}

impl From<&std::io::Error> for Errno {
    fn from(e: &std::io::Error) -> Self {
        if let Some(error_code) = e.raw_os_error() {
            match error_code {
                0 => Errno::__WASI_ERRNO_SUCCESS,
                libc::E2BIG => Errno::__WASI_ERRNO_2BIG,
                libc::EACCES => Errno::__WASI_ERRNO_ACCES,
                libc::EADDRINUSE => Errno::__WASI_ERRNO_ADDRINUSE,
                libc::EADDRNOTAVAIL => Errno::__WASI_ERRNO_ADDRNOTAVAIL,
                libc::EAFNOSUPPORT => Errno::__WASI_ERRNO_AFNOSUPPORT,
                libc::EAGAIN => Errno::__WASI_ERRNO_AGAIN,
                libc::EALREADY => Errno::__WASI_ERRNO_ALREADY,
                libc::EBADF => Errno::__WASI_ERRNO_BADF,
                libc::EBADMSG => Errno::__WASI_ERRNO_BADMSG,
                libc::EBUSY => Errno::__WASI_ERRNO_BUSY,
                libc::ECANCELED => Errno::__WASI_ERRNO_CANCELED,
                libc::ECHILD => Errno::__WASI_ERRNO_CHILD,
                libc::ECONNABORTED => Errno::__WASI_ERRNO_CONNABORTED,
                libc::ECONNREFUSED => Errno::__WASI_ERRNO_CONNREFUSED,
                libc::ECONNRESET => Errno::__WASI_ERRNO_CONNRESET,
                libc::EDEADLK => Errno::__WASI_ERRNO_DEADLK,
                libc::EDESTADDRREQ => Errno::__WASI_ERRNO_DESTADDRREQ,
                libc::EDOM => Errno::__WASI_ERRNO_DOM,
                #[cfg(unix)]
                libc::EDQUOT => Errno::__WASI_ERRNO_DQUOT,
                libc::EEXIST => Errno::__WASI_ERRNO_EXIST,
                libc::EFAULT => Errno::__WASI_ERRNO_FAULT,
                libc::EFBIG => Errno::__WASI_ERRNO_FBIG,
                libc::EHOSTUNREACH => Errno::__WASI_ERRNO_HOSTUNREACH,
                libc::EIDRM => Errno::__WASI_ERRNO_IDRM,
                libc::EILSEQ => Errno::__WASI_ERRNO_ILSEQ,
                libc::EINPROGRESS => Errno::__WASI_ERRNO_INPROGRESS,
                libc::EINTR => Errno::__WASI_ERRNO_INTR,
                libc::EINVAL => Errno::__WASI_ERRNO_INVAL,
                libc::EIO => Errno::__WASI_ERRNO_IO,
                libc::EISCONN => Errno::__WASI_ERRNO_ISCONN,
                libc::EISDIR => Errno::__WASI_ERRNO_ISDIR,
                libc::ELOOP => Errno::__WASI_ERRNO_LOOP,
                libc::EMFILE => Errno::__WASI_ERRNO_MFILE,
                libc::EMLINK => Errno::__WASI_ERRNO_MLINK,
                libc::EMSGSIZE => Errno::__WASI_ERRNO_MSGSIZE,
                #[cfg(unix)]
                libc::EMULTIHOP => Errno::__WASI_ERRNO_MULTIHOP,
                libc::ENAMETOOLONG => Errno::__WASI_ERRNO_NAMETOOLONG,
                libc::ENETDOWN => Errno::__WASI_ERRNO_NETDOWN,
                libc::ENETRESET => Errno::__WASI_ERRNO_NETRESET,
                libc::ENETUNREACH => Errno::__WASI_ERRNO_NETUNREACH,
                libc::ENFILE => Errno::__WASI_ERRNO_NFILE,
                libc::ENOBUFS => Errno::__WASI_ERRNO_NOBUFS,
                libc::ENODEV => Errno::__WASI_ERRNO_NODEV,
                libc::ENOENT => Errno::__WASI_ERRNO_NOENT,
                libc::ENOEXEC => Errno::__WASI_ERRNO_NOEXEC,
                libc::ENOLCK => Errno::__WASI_ERRNO_NOLCK,
                libc::ENOLINK => Errno::__WASI_ERRNO_NOLINK,
                libc::ENOMEM => Errno::__WASI_ERRNO_NOMEM,
                libc::ENOMSG => Errno::__WASI_ERRNO_NOMSG,
                libc::ENOPROTOOPT => Errno::__WASI_ERRNO_NOPROTOOPT,
                libc::ENOSPC => Errno::__WASI_ERRNO_NOSPC,
                libc::ENOSYS => Errno::__WASI_ERRNO_NOSYS,
                libc::ENOTCONN => Errno::__WASI_ERRNO_NOTCONN,
                libc::ENOTDIR => Errno::__WASI_ERRNO_NOTDIR,
                libc::ENOTEMPTY => Errno::__WASI_ERRNO_NOTEMPTY,
                libc::ENOTRECOVERABLE => Errno::__WASI_ERRNO_NOTRECOVERABLE,
                libc::ENOTSOCK => Errno::__WASI_ERRNO_NOTSOCK,
                libc::ENOTSUP => Errno::__WASI_ERRNO_NOTSUP,
                libc::ENOTTY => Errno::__WASI_ERRNO_NOTTY,
                libc::ENXIO => Errno::__WASI_ERRNO_NXIO,
                libc::EOVERFLOW => Errno::__WASI_ERRNO_OVERFLOW,
                libc::EOWNERDEAD => Errno::__WASI_ERRNO_OWNERDEAD,
                libc::EPERM => Errno::__WASI_ERRNO_PERM,
                libc::EPIPE => Errno::__WASI_ERRNO_PIPE,
                libc::EPROTO => Errno::__WASI_ERRNO_PROTO,
                libc::EPROTONOSUPPORT => Errno::__WASI_ERRNO_PROTONOSUPPORT,
                libc::EPROTOTYPE => Errno::__WASI_ERRNO_PROTOTYPE,
                libc::ERANGE => Errno::__WASI_ERRNO_RANGE,
                libc::EROFS => Errno::__WASI_ERRNO_ROFS,
                libc::ESPIPE => Errno::__WASI_ERRNO_SPIPE,
                libc::ESRCH => Errno::__WASI_ERRNO_SRCH,
                #[cfg(unix)]
                libc::ESTALE => Errno::__WASI_ERRNO_STALE,
                libc::ETIMEDOUT => Errno::__WASI_ERRNO_TIMEDOUT,
                libc::ETXTBSY => Errno::__WASI_ERRNO_TXTBSY,
                libc::EXDEV => Errno::__WASI_ERRNO_XDEV,
                _ => Errno::__WASI_ERRNO_IO,
            }
        } else {
            let kind = e.kind();
            Errno::from(kind)
        }
    }
}

impl From<std::io::Error> for Errno {
    fn from(e: std::io::Error) -> Self {
        Errno::from(&e)
    }
}
