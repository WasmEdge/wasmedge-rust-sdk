//! Defines WasmEdge Store struct.

use crate::{
    WasmEdgeResult, ffi,
    instance::{
        InnerRef,
        module::{InnerInstance, Instance},
    },
    types::WasmEdgeString,
};

use wasmedge_types::error::{StoreError, WasmEdgeError};

/// The [Store] is a collection of registered modules and assists wasm modules in finding the import modules they need.
#[derive(Debug)]
pub struct Store {
    pub(crate) inner: InnerStore,
}
impl Store {
    /// Creates a new [Store].
    ///
    /// # Error
    ///
    /// If fail to create, then an error is returned.
    pub fn create() -> WasmEdgeResult<Self> {
        let ctx = unsafe { ffi::WasmEdge_StoreCreate() };
        if ctx.is_null() {
            Err(Box::new(WasmEdgeError::Store(StoreError::Create)))
        } else {
            Ok(Store {
                inner: InnerStore(ctx),
            })
        }
    }

    /// Returns the length of the registered [modules](crate::Module).
    pub fn module_len(&self) -> u32 {
        unsafe { ffi::WasmEdge_StoreListModuleLength(self.inner.0 as *const _) }
    }

    /// Returns the names of all registered [modules](crate::Module).
    pub fn module_names(&self) -> Option<Vec<String>> {
        let len_mod_names = self.module_len();
        if len_mod_names > 0 {
            let mut mod_names = Vec::with_capacity(len_mod_names as usize);
            // SAFETY: `mod_names` is reserved with capacity `len_mod_names` — the exact
            // count just queried via `module_len` (`WasmEdge_StoreListModuleLength`) — and
            // `WasmEdge_StoreListModule` fills that many POD `WasmEdge_String`s, so all
            // `len_mod_names` slots are initialized before `set_len`.
            unsafe {
                ffi::WasmEdge_StoreListModule(self.inner.0, mod_names.as_mut_ptr(), len_mod_names);
                mod_names.set_len(len_mod_names as usize);
            };

            let names = mod_names
                .into_iter()
                .map(|x| x.into())
                .collect::<Vec<String>>();
            Some(names)
        } else {
            None
        }
    }

    /// Returns the module instance by the module name.
    ///
    /// # Argument
    ///
    /// * `name` - The name of the module instance to get.
    ///
    /// # Error
    ///
    /// If fail to find the target [module instance](crate::Instance), then an error is returned.
    pub fn module(&self, name: impl AsRef<str>) -> WasmEdgeResult<InnerRef<Instance, &Self>> {
        let mod_name: WasmEdgeString = name.as_ref().into();
        let ctx = unsafe { ffi::WasmEdge_StoreFindModule(self.inner.0, mod_name.as_raw()) };
        if ctx.is_null() {
            Err(Box::new(WasmEdgeError::Store(StoreError::NotFoundModule(
                name.as_ref().to_string(),
            ))))
        } else {
            let inst = Instance {
                inner: InnerInstance(ctx as _),
            };
            unsafe {
                Ok(InnerRef::create_from_ref(
                    std::mem::ManuallyDrop::new(inst),
                    self,
                ))
            }
        }
    }

    /// Checks if the [Store] contains a module of which the name matches the given name.
    ///
    /// # Argument
    ///
    /// * `name` - The name of the module to search.
    ///
    pub fn contains(&self, name: impl AsRef<str>) -> bool {
        if self.module_len() == 0 {
            return false;
        }

        match self.module_names() {
            Some(names) => names.iter().any(|x| x == name.as_ref()),
            None => false,
        }
    }
}
impl Drop for Store {
    fn drop(&mut self) {
        unsafe { ffi::WasmEdge_StoreDelete(self.inner.0) }
    }
}

#[derive(Debug)]
pub(crate) struct InnerStore(pub(crate) *mut ffi::WasmEdge_StoreContext);
// SAFETY: (assumed, pre-existing) owns an opaque `*mut WasmEdge_StoreContext`.
// `Send` is sound: a move transfers sole ownership of a thread-agnostic handle.
// `Sync` is the assumed half — wasmedge_execution.h documents
// `WasmEdge_StoreFindModule`, `WasmEdge_StoreListModuleLength`, and
// `WasmEdge_StoreListModule` (this crate's only `&self` C calls) as
// thread-safe, but mutation only happens through `&mut Store` via the
// `WasmEdge_ExecutorRegister*` family, which carries no such annotation —
// so `Sync` remains an unverified, inherited invariant for that path.
unsafe impl Send for InnerStore {}
unsafe impl Sync for InnerStore {}
