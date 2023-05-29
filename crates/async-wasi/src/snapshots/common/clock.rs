use crate::snapshots::{
    common::error::Errno,
    common::types::{__wasi_clockid_t, __wasi_errno_t, __wasi_timestamp_t},
    WasiCtx,
};

pub fn wasi_clock_res_get(clock_id: __wasi_clockid_t::Type) -> Result<u64, Errno> {
    match clock_id {
        __wasi_clockid_t::__WASI_CLOCKID_MONOTONIC => Ok(1),
        __wasi_clockid_t::__WASI_CLOCKID_REALTIME => Ok(1),
        _ => Err(Errno(__wasi_errno_t::__WASI_ERRNO_BADF)),
    }
}

pub fn wasi_clock_time_get(
    ctx: &WasiCtx,
    clock_id: __wasi_clockid_t::Type,
    _precision: __wasi_timestamp_t,
) -> Result<u64, Errno> {
    use std::time::SystemTime;
    match clock_id {
        __wasi_clockid_t::__WASI_CLOCKID_REALTIME | __wasi_clockid_t::__WASI_CLOCKID_MONOTONIC => {
            let d = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap();
            Ok(d.as_nanos() as u64)
        }
        _ => Err(Errno(__wasi_errno_t::__WASI_ERRNO_NODEV)),
    }
}
