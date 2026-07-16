use super::{
    error::Errno,
    types::{__wasi_ciovec_t, __wasi_iovec_t, __wasi_size_t},
};
use std::{
    io::{IoSlice, IoSliceMut},
    ops::{Add, Deref, DerefMut, Sub},
};

pub trait Memory {
    fn get_data<T: Sized>(&self, offset: WasmPtr<T>) -> Result<&T, Errno>;

    fn get_slice<T: Sized>(&self, offset: WasmPtr<T>, len: usize) -> Result<&[T], Errno>;

    fn get_iovec<'a>(
        &self,
        iovec_ptr: WasmPtr<__wasi_ciovec_t>,
        iovec_len: __wasi_size_t,
    ) -> Result<Vec<IoSlice<'a>>, Errno>;

    fn mut_data<T: Sized>(&mut self, offset: WasmPtr<T>) -> Result<&mut T, Errno>;

    fn mut_slice<T: Sized>(&mut self, offset: WasmPtr<T>, len: usize) -> Result<&mut [T], Errno>;

    fn mut_iovec(
        &mut self,
        iovec_ptr: WasmPtr<__wasi_iovec_t>,
        iovec_len: __wasi_size_t,
    ) -> Result<Vec<IoSliceMut<'_>>, Errno>;

    fn write_data<T: Sized>(&mut self, offset: WasmPtr<T>, data: T) -> Result<(), Errno>;
}

#[derive(Debug, Clone, Copy)]
pub struct WasmPtr<T: Sized>(pub usize, std::marker::PhantomData<T>);
impl<T: Sized> WasmPtr<T> {
    pub fn is_null(&self) -> bool {
        self.0 == 0
    }
}
impl<T: Sized> From<usize> for WasmPtr<T> {
    fn from(i: usize) -> Self {
        WasmPtr(i, Default::default())
    }
}
impl<T: Sized> From<WasmPtr<T>> for usize {
    fn from(val: WasmPtr<T>) -> Self {
        val.0
    }
}
impl<T: Sized> Add<usize> for WasmPtr<T> {
    type Output = Self;
    fn add(mut self, rhs: usize) -> Self::Output {
        self.0 += rhs * std::mem::size_of::<T>();
        self
    }
}
impl<T: Sized> Sub<usize> for WasmPtr<T> {
    type Output = Self;
    fn sub(mut self, rhs: usize) -> Self::Output {
        self.0 -= rhs * std::mem::size_of::<T>();
        self
    }
}

/// A minimal `Vec<u8>`-backed [`Memory`] implementation used only by unit tests.
///
/// It models a flat guest linear address space: a [`WasmPtr<T>`] is a byte
/// offset into `data`, and slices/values are reinterpreted in place. Callers are
/// responsible for placing values at offsets that satisfy `T`'s alignment (the
/// alignment invariant is asserted in debug builds).
#[cfg(test)]
pub(crate) struct TestMemory {
    pub(crate) data: Vec<u8>,
}

#[cfg(test)]
impl TestMemory {
    pub(crate) fn new(size: usize) -> Self {
        Self {
            data: vec![0u8; size],
        }
    }

    /// Resolve `[offset, offset + len * size_of::<T>())` against `data`,
    /// returning the byte range or `EFAULT` if it is out of bounds.
    fn byte_range<T>(&self, offset: WasmPtr<T>, len: usize) -> Result<(usize, usize), Errno> {
        let start: usize = offset.into();
        let byte_len = len
            .checked_mul(std::mem::size_of::<T>())
            .ok_or(Errno::__WASI_ERRNO_FAULT)?;
        let end = start
            .checked_add(byte_len)
            .ok_or(Errno::__WASI_ERRNO_FAULT)?;
        if end > self.data.len() {
            return Err(Errno::__WASI_ERRNO_FAULT);
        }
        Ok((start, end))
    }
}

#[cfg(test)]
impl Memory for TestMemory {
    fn get_data<T: Sized>(&self, offset: WasmPtr<T>) -> Result<&T, Errno> {
        Ok(&self.get_slice(offset, 1)?[0])
    }

    #[allow(clippy::cast_ptr_alignment)] // test-only; alignment asserted below
    fn get_slice<T: Sized>(&self, offset: WasmPtr<T>, len: usize) -> Result<&[T], Errno> {
        let (start, end) = self.byte_range(offset, len)?;
        let bytes = &self.data[start..end];
        debug_assert_eq!(bytes.as_ptr().align_offset(std::mem::align_of::<T>()), 0);
        // SAFETY: length checked by `byte_range`, alignment asserted above,
        // and the borrow is tied to `&self`.
        Ok(unsafe { std::slice::from_raw_parts(bytes.as_ptr().cast::<T>(), len) })
    }

    fn get_iovec<'a>(
        &self,
        _iovec_ptr: WasmPtr<__wasi_ciovec_t>,
        _iovec_len: __wasi_size_t,
    ) -> Result<Vec<IoSlice<'a>>, Errno> {
        unimplemented!("not needed by tests")
    }

    fn mut_data<T: Sized>(&mut self, offset: WasmPtr<T>) -> Result<&mut T, Errno> {
        Ok(&mut self.mut_slice(offset, 1)?[0])
    }

    #[allow(clippy::cast_ptr_alignment)] // test-only; alignment asserted below
    fn mut_slice<T: Sized>(&mut self, offset: WasmPtr<T>, len: usize) -> Result<&mut [T], Errno> {
        let (start, end) = self.byte_range(offset, len)?;
        let bytes = &mut self.data[start..end];
        debug_assert_eq!(bytes.as_ptr().align_offset(std::mem::align_of::<T>()), 0);
        // SAFETY: length checked by `byte_range`, alignment asserted above,
        // and the borrow is tied to `&mut self`.
        Ok(unsafe { std::slice::from_raw_parts_mut(bytes.as_mut_ptr().cast::<T>(), len) })
    }

    fn mut_iovec(
        &mut self,
        _iovec_ptr: WasmPtr<__wasi_iovec_t>,
        _iovec_len: __wasi_size_t,
    ) -> Result<Vec<IoSliceMut<'_>>, Errno> {
        unimplemented!("not needed by tests")
    }

    fn write_data<T: Sized>(&mut self, offset: WasmPtr<T>, data: T) -> Result<(), Errno> {
        *self.mut_data(offset)? = data;
        Ok(())
    }
}
