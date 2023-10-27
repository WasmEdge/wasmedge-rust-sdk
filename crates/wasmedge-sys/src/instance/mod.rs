//! Defines WasmEdge instance structs, including Function, Global, Memory, Table, and etc.

pub mod function;
pub mod global;
pub mod memory;
pub mod module;
pub mod table;

use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};

#[doc(hidden)]
pub use function::Function;
#[doc(hidden)]
pub use global::Global;
#[doc(hidden)]
pub use memory::Memory;
#[doc(hidden)]
pub use module::Instance;
#[doc(hidden)]
pub use table::Table;
pub use wasmedge_types::{FuncType, GlobalType, MemoryType, TableType};

pub struct InnerRef<D, Ref: ?Sized> {
    value: std::mem::ManuallyDrop<D>,
    _ref: std::marker::PhantomData<Ref>,
}

impl<D, Ref: ?Sized> InnerRef<D, &Ref> {
    /// # Safety
    ///
    /// The return value type of this function should ensure the correctness of lifetimes.
    pub unsafe fn create_from_ref(value: std::mem::ManuallyDrop<D>, _r: &Ref) -> Self {
        let r = Default::default();
        Self { value, _ref: r }
    }

    /// # Safety
    ///
    /// The return value type of this function should ensure the correctness of lifetimes.
    pub unsafe fn create_ref(value: std::mem::ManuallyDrop<D>) -> Self {
        let r = Default::default();
        Self { value, _ref: r }
    }
}

impl<D, Ref: ?Sized> InnerRef<D, &mut Ref> {
    /// # Safety
    ///
    /// The return value type of this function should ensure the correctness of lifetimes.
    pub unsafe fn create_from_mut(value: std::mem::ManuallyDrop<D>, _r: &mut Ref) -> Self {
        let r = Default::default();
        Self { value, _ref: r }
    }

    /// # Safety
    ///
    /// The return value type of this function should ensure the correctness of lifetimes.
    pub unsafe fn create_mut(value: std::mem::ManuallyDrop<D>) -> Self {
        let r = Default::default();
        Self { value, _ref: r }
    }
}

impl<D: Debug, Ref> Debug for InnerRef<D, Ref> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.fmt(f)
    }
}

impl<D, Ref> Deref for InnerRef<D, &Ref> {
    type Target = D;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<D: Clone, Ref> Clone for InnerRef<D, &mut Ref> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            _ref: self._ref,
        }
    }
}

impl<D, Ref> Deref for InnerRef<D, &mut Ref> {
    type Target = D;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<D, Ref> DerefMut for InnerRef<D, &mut Ref> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}
