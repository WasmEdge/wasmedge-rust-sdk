use super::error::Errno;
use super::types::{__wasi_ciovec_t, __wasi_iovec_t, __wasi_size_t};
use std::io::{IoSlice, IoSliceMut};
use std::ops::{Add, Deref, DerefMut, Sub};

pub trait Memory {
    fn get_data<'a, T: Sized>(&'a self, offset: WasmPtr<T>) -> Result<&'a T, Errno>;

    fn get_slice<'a, T: Sized>(&'a self, offset: WasmPtr<T>, len: usize) -> Result<&'a [T], Errno>;

    fn get_iovec<'a>(
        &'a self,
        iovec_ptr: WasmPtr<__wasi_ciovec_t>,
        iovec_len: __wasi_size_t,
    ) -> Result<Vec<IoSlice<'a>>, Errno>;

    fn mut_data<'a, T: Sized>(&'a mut self, offset: WasmPtr<T>) -> Result<&'a mut T, Errno>;

    fn mut_slice<'a, T: Sized>(
        &'a mut self,
        offset: WasmPtr<T>,
        len: usize,
    ) -> Result<&'a mut [T], Errno>;

    fn mut_iovec<'a>(
        &'a mut self,
        iovec_ptr: WasmPtr<__wasi_iovec_t>,
        iovec_len: __wasi_size_t,
    ) -> Result<Vec<IoSliceMut<'a>>, Errno>;

    fn write_data<'a, T: Sized>(&'a mut self, offset: WasmPtr<T>, data: T) -> Result<(), Errno>;
}

#[derive(Clone, Copy)]
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
impl<T: Sized> Into<usize> for WasmPtr<T> {
    fn into(self) -> usize {
        self.0
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
