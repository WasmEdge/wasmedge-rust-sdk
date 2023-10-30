//! Defines WasmEdge CallingFrame.

use crate::{
    ffi,
    instance::{memory::InnerMemory, InnerRef},
    Memory,
};

/// Represents a calling frame on top of stack.
#[derive(Debug)]
pub struct CallingFrame {
    pub(crate) inner: InnerCallingFrame,
}

impl CallingFrame {
    /// Creates a CallingFrame instance.
    pub(crate) fn create(ctx: *const ffi::WasmEdge_CallingFrameContext) -> Self {
        Self {
            inner: InnerCallingFrame(ctx),
        }
    }

    /// Returns an immutable smart pointer borrowing the [memory instance](crate::Memory) by the given index from the module instance of the current
    /// calling frame. If the memory instance is not found, returns `None`.
    ///
    /// By default, a WASM module has only one memory instance after instantiation. Therefore, users can pass in `0` as
    /// the index to get the memory instance in host function body. When the [MultiMemories](crate::Config::multi_memories)
    /// config option is enabled, there would be more than one memory instances in the wasm module. Users can retrieve
    /// the target memory instance by specifying the index of the memory instance in the wasm module instance.
    ///
    /// # Arguments
    ///
    /// * idx - The index of the memory instance.
    pub fn memory_ref(&self, idx: u32) -> Option<InnerRef<Memory, &Self>> {
        unsafe {
            let ctx = ffi::WasmEdge_CallingFrameGetMemoryInstance(self.inner.0, idx);

            if ctx.is_null() {
                None
            } else {
                let mem = Memory {
                    inner: InnerMemory(ctx),
                };
                Some(InnerRef::create_from_ref(
                    std::mem::ManuallyDrop::new(mem),
                    self,
                ))
            }
        }
    }

    /// Returns an mutable smart pointer borrowing the [memory instance](crate::Memory) by the given index from the module instance of the current
    /// calling frame. If the memory instance is not found, returns `None`.
    ///
    /// By default, a WASM module has only one memory instance after instantiation. Therefore, users can pass in `0` as
    /// the index to get the memory instance in host function body. When the [MultiMemories](crate::Config::multi_memories)
    /// config option is enabled, there would be more than one memory instances in the wasm module. Users can retrieve
    /// the target memory instance by specifying the index of the memory instance in the wasm module instance.
    ///
    /// # Arguments
    ///
    /// * idx - The index of the memory instance.
    pub fn memory_mut(&mut self, idx: u32) -> Option<InnerRef<Memory, &mut Self>> {
        unsafe {
            let ctx = ffi::WasmEdge_CallingFrameGetMemoryInstance(self.inner.0, idx);

            if ctx.is_null() {
                None
            } else {
                let mem = Memory {
                    inner: InnerMemory(ctx),
                };
                Some(InnerRef::create_from_mut(
                    std::mem::ManuallyDrop::new(mem),
                    self,
                ))
            }
        }
    }
}

#[derive(Debug)]
pub(crate) struct InnerCallingFrame(pub(crate) *const ffi::WasmEdge_CallingFrameContext);
unsafe impl Send for InnerCallingFrame {}
unsafe impl Sync for InnerCallingFrame {}
