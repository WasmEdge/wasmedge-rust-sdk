//! Defines WasmEdge Memory and MemType structs.
//!
//! A WasmEdge `Memory` defines a linear memory as described by `MemType`.
//! `MemType` specifies the limits on the size of a memory by a range. The start of
//! the limit range specifies min size (initial size) of that memory, while the end
//! restricts the size to which the memory can grow later.

use crate::{ffi, types::WasmEdgeLimit, utils::check, WasmEdgeResult};
use wasmedge_types::error::{MemError, WasmEdgeError};

/// Defines a WebAssembly memory instance, which is a linear memory described by its [type](crate::MemType). Each memory instance consists of a vector of bytes and an optional maximum size, and its size is a multiple of the WebAssembly page size (*64KiB* of each page).
#[derive(Debug)]
pub struct Memory {
    pub(crate) inner: InnerMemory,
}
impl Memory {
    /// Create a new [Memory] to be associated with the given capacity limit.
    ///
    /// # Arguments
    ///
    /// * `ty` - The type of the new [Memory] instance.
    ///
    /// # Errors
    ///
    /// * If fail to create the memory instance, then [WasmEdgeError::Mem(MemError::Create)](wasmedge_types::error::MemError) is returned.
    ///
    pub fn create(ty: &wasmedge_types::MemoryType) -> WasmEdgeResult<Self> {
        let ty: MemType = ty.into();
        let ctx = unsafe { ffi::WasmEdge_MemoryInstanceCreate(ty.inner.0 as *const _) };

        match ctx.is_null() {
            true => Err(Box::new(WasmEdgeError::Mem(MemError::Create))),
            false => Ok(Memory {
                inner: InnerMemory(ctx),
            }),
        }
    }

    /// Returns the type of the [Memory].
    ///
    /// # Errors
    ///
    /// If fail to get the type from the [Memory], then an error is returned.
    ///
    pub fn ty(&self) -> WasmEdgeResult<wasmedge_types::MemoryType> {
        let ty_ctx = unsafe { ffi::WasmEdge_MemoryInstanceGetMemoryType(self.inner.0) };
        match ty_ctx.is_null() {
            true => Err(Box::new(WasmEdgeError::Mem(MemError::Type))),
            false => {
                let ty = std::mem::ManuallyDrop::new(MemType {
                    inner: InnerMemType(ty_ctx as *mut _),
                });
                Ok((&*ty).into())
            }
        }
    }

    /// Copies the data from the [Memory] to the output buffer.
    ///
    /// # Arguments
    ///
    /// * `offset` - The data start offset in the [Memory].
    ///
    /// * `len` - The requested data length.
    ///
    /// # Errors
    ///
    /// If the `offset + len` is larger than the data size in the [Memory], then an error is returned.
    ///
    pub fn get_data(&self, offset: u32, len: u32) -> WasmEdgeResult<Vec<u8>> {
        let mut data = Vec::with_capacity(len as usize);
        unsafe {
            check(ffi::WasmEdge_MemoryInstanceGetData(
                self.inner.0,
                data.as_mut_ptr(),
                offset,
                len,
            ))?;
            data.set_len(len as usize);
        }

        Ok(data.into_iter().collect())
    }

    /// Copies the data from the given input buffer into the [Memory].
    ///
    /// # Arguments
    ///
    /// * `data` - The data buffer to copy.
    ///
    /// * `offset` - The data start offset in the [Memory].
    ///
    /// # Errors
    ///
    /// If the sum of the `offset` and the data length is larger than the size of the [Memory],
    /// then an error is returned.
    ///
    pub fn set_data(&mut self, data: impl AsRef<[u8]>, offset: u32) -> WasmEdgeResult<()> {
        unsafe {
            check(ffi::WasmEdge_MemoryInstanceSetData(
                self.inner.0,
                data.as_ref().as_ptr(),
                offset,
                data.as_ref().len() as u32,
            ))
        }
    }

    /// Returns the const data pointer to the [Memory].
    ///
    /// # Arguments
    ///
    /// * `offset` - The data start offset in the [Memory].
    ///
    /// * `len` - The requested data length. If the size of `offset` + `len` is larger than the data size in the [Memory]
    ///
    ///
    /// # Errors
    ///
    /// If fail to get the data pointer, then an error is returned.
    ///
    /// # Safety
    ///
    /// The lifetime of the returned pointer must not exceed that of the object itself.
    ///
    pub unsafe fn data_pointer(&self, offset: u32, len: u32) -> WasmEdgeResult<*const u8> {
        let ptr = unsafe { ffi::WasmEdge_MemoryInstanceGetPointerConst(self.inner.0, offset, len) };
        match ptr.is_null() {
            true => Err(Box::new(WasmEdgeError::Mem(MemError::ConstPtr))),
            false => Ok(ptr),
        }
    }

    /// Returns the data pointer to the [Memory].
    ///
    /// # Arguments
    ///
    /// * `offset` - The data start offset in the [Memory].
    ///
    /// * `len` - The requested data length. If the size of `offset` + `len` is larger than the data size in the [Memory]
    ///
    /// # Errors
    ///
    /// If fail to get the data pointer, then an error is returned.
    ///
    /// # Safety
    ///
    /// The lifetime of the returned pointer must not exceed that of the object itself.
    ///
    pub unsafe fn data_pointer_mut(&mut self, offset: u32, len: u32) -> WasmEdgeResult<*mut u8> {
        let ptr = unsafe { ffi::WasmEdge_MemoryInstanceGetPointer(self.inner.0, offset, len) };
        match ptr.is_null() {
            true => Err(Box::new(WasmEdgeError::Mem(MemError::MutPtr))),
            false => Ok(ptr),
        }
    }

    /// Returns the size, in WebAssembly pages (64 KiB of each page), of this wasm memory.
    pub fn size(&self) -> u32 {
        unsafe { ffi::WasmEdge_MemoryInstanceGetPageSize(self.inner.0) }
    }

    /// Grows this WebAssembly memory by `count` pages.
    ///
    /// # Arguments
    ///
    /// * `count` - The page counts to be extended to the [Memory].
    ///
    /// # Errors
    ///
    /// If fail to grow the page count, then an error is returned.
    ///
    pub fn grow(&mut self, count: u32) -> WasmEdgeResult<()> {
        unsafe { check(ffi::WasmEdge_MemoryInstanceGrowPage(self.inner.0, count)) }
    }

    /// # Safety
    ///
    /// Provides a raw pointer to the inner memory context.
    /// The lifetime of the returned pointer must not exceed that of the object itself.
    pub unsafe fn as_ptr(&self) -> *const ffi::WasmEdge_MemoryInstanceContext {
        self.inner.0
    }
}
impl Drop for Memory {
    fn drop(&mut self) {
        unsafe { ffi::WasmEdge_MemoryInstanceDelete(self.inner.0) };
    }
}

impl Memory {
    pub fn get_ref<T: Sized>(&self, offset: usize) -> Option<&T> {
        unsafe {
            let r = std::mem::size_of::<T>();
            let ptr = self.data_pointer(offset as u32, r as u32).ok()?;
            ptr.cast::<T>().as_ref()
        }
    }

    pub fn slice<T: Sized>(&self, offset: usize, len: usize) -> Option<&[T]> {
        unsafe {
            let r = std::mem::size_of::<T>() * len;
            let ptr = self.data_pointer(offset as u32, r as u32).ok()? as *const T;
            Some(std::slice::from_raw_parts(ptr, len))
        }
    }

    pub fn get_ref_mut<T: Sized>(&mut self, offset: usize) -> Option<&mut T> {
        unsafe {
            let r = std::mem::size_of::<T>();
            let ptr = self.data_pointer_mut(offset as u32, r as u32).ok()?;
            ptr.cast::<T>().as_mut()
        }
    }

    pub fn mut_slice<T: Sized>(&self, offset: usize, len: usize) -> Option<&mut [T]> {
        unsafe {
            let r = std::mem::size_of::<T>() * len;
            let ptr = self.data_pointer(offset as u32, r as u32).ok()? as *mut T;
            Some(std::slice::from_raw_parts_mut(ptr, len))
        }
    }

    pub fn write<T: Sized>(&mut self, offset: usize, data: T) -> Option<()> {
        let p = self.get_ref_mut(offset)?;
        *p = data;
        Some(())
    }
}

#[derive(Debug)]
pub(crate) struct InnerMemory(pub(crate) *mut ffi::WasmEdge_MemoryInstanceContext);
unsafe impl Send for InnerMemory {}
unsafe impl Sync for InnerMemory {}

/// Defines the type of a wasm memory instance
#[derive(Debug)]
pub(crate) struct MemType {
    pub(crate) inner: InnerMemType,
}
impl MemType {
    /// Create a new [MemType] to be associated with the given limit range for the capacity.
    ///
    /// # Arguments
    ///
    /// * 'min' - The initial size of the linear memory.
    ///
    /// * 'max' - The upper bound of the linear memory size allowed to grow. If 'max' is set 'None', then the maximum size will be set `u32::MAX`.
    ///
    /// * `shared` - Whether the memory is shared or not. Reference [Threading proposal for WebAssembly](https://github.com/WebAssembly/threads/blob/main/proposals/threads/Overview.md#shared-linear-memory) for details about shared memory. If `shared` is set `true`, then `max` MUST not be `None`.
    ///
    /// # Errors
    ///
    /// If fail to create a [MemType], then an error is returned.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let ty = MemType::create(0, Some(u32::MAX), false);
    /// ```
    ///
    pub(crate) fn create(min: u32, max: Option<u32>, shared: bool) -> WasmEdgeResult<Self> {
        if shared && max.is_none() {
            return Err(Box::new(WasmEdgeError::Mem(MemError::CreateSharedType)));
        }
        let ctx =
            unsafe { ffi::WasmEdge_MemoryTypeCreate(WasmEdgeLimit::new(min, max, shared).into()) };
        match ctx.is_null() {
            true => Err(Box::new(WasmEdgeError::MemTypeCreate)),
            false => Ok(Self {
                inner: InnerMemType(ctx),
            }),
        }
    }

    /// Returns the initial size of a [Memory].
    pub(crate) fn min(&self) -> u32 {
        let limit = unsafe { ffi::WasmEdge_MemoryTypeGetLimit(self.inner.0) };
        let limit: WasmEdgeLimit = limit.into();
        limit.min()
    }

    /// Returns the maximum size of a [Memory] allowed to grow.
    pub(crate) fn max(&self) -> Option<u32> {
        let limit = unsafe { ffi::WasmEdge_MemoryTypeGetLimit(self.inner.0) };
        let limit: WasmEdgeLimit = limit.into();
        limit.max()
    }

    /// Returns whether the memory is shared or not.
    pub(crate) fn shared(&self) -> bool {
        let limit = unsafe { ffi::WasmEdge_MemoryTypeGetLimit(self.inner.0) };
        let limit: WasmEdgeLimit = limit.into();
        limit.shared()
    }
}
impl Drop for MemType {
    fn drop(&mut self) {
        unsafe { ffi::WasmEdge_MemoryTypeDelete(self.inner.0) }
    }
}
impl From<&wasmedge_types::MemoryType> for MemType {
    fn from(ty: &wasmedge_types::MemoryType) -> Self {
        MemType::create(ty.minimum(), ty.maximum(), ty.shared()).expect(
            "[wasmedge-sys] Failed to convert wasmedge_types::MemoryType into wasmedge_sys::MemType.",
        )
    }
}
impl From<wasmedge_types::MemoryType> for MemType {
    fn from(ty: wasmedge_types::MemoryType) -> Self {
        (&ty).into()
    }
}
impl From<&MemType> for wasmedge_types::MemoryType {
    fn from(ty: &MemType) -> Self {
        wasmedge_types::MemoryType::new(ty.min(), ty.max(), ty.shared()).expect(
            "[wasmedge-sys] Failed to convert wasmedge_sys::MemType into wasmedge_types::MemoryType."
        )
    }
}

#[derive(Debug)]
pub(crate) struct InnerMemType(pub(crate) *mut ffi::WasmEdge_MemoryTypeContext);
unsafe impl Send for InnerMemType {}
unsafe impl Sync for InnerMemType {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use wasmedge_types::error::{CoreError, CoreExecutionError, WasmEdgeError};

    #[test]
    fn test_memory_type() {
        // case 1
        let result = MemType::create(0, Some(u32::MAX), false);
        assert!(result.is_ok());
        let ty = result.unwrap();
        assert!(!ty.inner.0.is_null());
        assert_eq!(ty.min(), 0);
        assert_eq!(ty.max(), Some(u32::MAX));

        // case 2
        let result = MemType::create(10, Some(101), false);
        assert!(result.is_ok());
        let ty = result.unwrap();
        assert!(!ty.inner.0.is_null());
        assert_eq!(ty.min(), 10);
        assert_eq!(ty.max(), Some(101));
    }

    #[test]
    #[allow(clippy::assertions_on_result_states)]
    fn test_memory_grow() {
        // create a Memory with a limit range [10, 20]
        let result = wasmedge_types::MemoryType::new(10, Some(20), false);
        assert!(result.is_ok());
        let ty = result.unwrap();
        let result = Memory::create(&ty);
        assert!(result.is_ok());
        let mut mem = result.unwrap();
        assert!(!mem.inner.0.is_null());

        // get type
        let result = mem.ty();
        assert!(result.is_ok());
        let ty = result.unwrap();
        // check limit
        assert_eq!(ty.minimum(), 10);
        assert_eq!(ty.maximum(), Some(20));

        // check page count
        let count = mem.size();
        assert_eq!(count, 10);

        // grow 5 pages
        let result = mem.grow(10);
        assert!(result.is_ok());
        assert_eq!(mem.size(), 20);

        // grow additional  pages, which causes a failure
        let result = mem.grow(1);
        assert!(result.is_err());
    }

    #[test]
    #[allow(clippy::assertions_on_result_states)]
    fn test_memory_data() {
        // create a Memory: the min size 1 and the max size 2
        let result = wasmedge_types::MemoryType::new(1, Some(2), false);
        assert!(result.is_ok());
        let ty = result.unwrap();
        let result = Memory::create(&ty);
        assert!(result.is_ok());
        let mut mem = result.unwrap();
        assert!(!mem.inner.0.is_null());

        // check page count
        let count = mem.size();
        assert_eq!(count, 1);

        // get data before set data
        let result = mem.get_data(0, 10);
        assert!(result.is_ok());
        let data = result.unwrap();
        assert_eq!(data, vec![0; 10]);

        // set data
        let result = mem.set_data(vec![1; 10], 10);
        assert!(result.is_ok());
        // get data after set data
        let result = mem.get_data(10, 10);
        assert!(result.is_ok());
        let data = result.unwrap();
        assert_eq!(data, vec![1; 10]);

        // set data and the data length is larger than the data size in the memory
        let result = mem.set_data(vec![1; 10], u32::pow(2, 16) - 9);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            Box::new(WasmEdgeError::Core(CoreError::Execution(
                CoreExecutionError::MemoryOutOfBounds
            )))
        );

        // grow the memory size
        let result = mem.grow(1);
        assert!(result.is_ok());
        assert_eq!(mem.size(), 2);
        let result = mem.set_data(vec![1; 10], u32::pow(2, 16) - 9);
        assert!(result.is_ok());
    }

    #[test]
    fn test_memory_send() {
        {
            let result = wasmedge_types::MemoryType::new(10, Some(101), false);
            assert!(result.is_ok());
            let ty = result.unwrap();

            let handle = thread::spawn(move || {
                assert_eq!(ty.minimum(), 10);
                assert_eq!(ty.maximum(), Some(101));
            });

            handle.join().unwrap()
        }

        {
            // create a Memory with a limit range [10, 20]
            let result = wasmedge_types::MemoryType::new(10, Some(20), false);
            assert!(result.is_ok());
            let ty = result.unwrap();
            let result = Memory::create(&ty);
            assert!(result.is_ok());
            let mem = result.unwrap();
            assert!(!mem.inner.0.is_null());

            let handle = thread::spawn(move || {
                // get type
                let result = mem.ty();
                assert!(result.is_ok());
                let ty = result.unwrap();
                // check limit
                assert_eq!(ty.minimum(), 10);
                assert_eq!(ty.maximum(), Some(20));

                // check page count
                let count = mem.size();
                assert_eq!(count, 10);
            });

            handle.join().unwrap()
        }
    }

    #[test]
    fn test_memory_sync() {
        // create a Memory with a limit range [10, 20]
        let result = wasmedge_types::MemoryType::new(10, Some(20), false);
        assert!(result.is_ok());
        let ty = result.unwrap();
        let result = Memory::create(&ty);
        assert!(result.is_ok());
        let mem = result.unwrap();
        assert!(!mem.inner.0.is_null());
        let mem = &mem;

        std::thread::scope(|s| {
            let _ = s
                .spawn(|| {
                    // get type
                    let result = mem.ty();
                    assert!(result.is_ok());
                    let ty = result.unwrap();
                    // check limit
                    assert_eq!(ty.minimum(), 10);
                    assert_eq!(ty.maximum(), Some(20));

                    // check page count
                    let count = mem.size();
                    assert_eq!(count, 10);
                })
                .join();
        })
    }
}
