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
