//! Defines WasmEdge Store struct.

use crate::{
    ffi,
    instance::{
        module::{InnerInstance, Instance},
        InnerRef,
    },
    types::WasmEdgeString,
    WasmEdgeResult,
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
        match ctx.is_null() {
            true => Err(Box::new(WasmEdgeError::Store(StoreError::Create))),
            false => Ok(Store {
                inner: InnerStore(ctx),
            }),
        }
    }

    /// Returns the length of the registered [modules](crate::Module).
    pub fn module_len(&self) -> u32 {
        unsafe { ffi::WasmEdge_StoreListModuleLength(self.inner.0 as *const _) }
    }

    /// Returns the names of all registered [modules](crate::Module).
    pub fn module_names(&self) -> Option<Vec<String>> {
        let len_mod_names = self.module_len();
        match len_mod_names > 0 {
            true => {
                let mut mod_names = Vec::with_capacity(len_mod_names as usize);
                unsafe {
                    ffi::WasmEdge_StoreListModule(
                        self.inner.0,
                        mod_names.as_mut_ptr(),
                        len_mod_names,
                    );
                    mod_names.set_len(len_mod_names as usize);
                };

                let names = mod_names
                    .into_iter()
                    .map(|x| x.into())
                    .collect::<Vec<String>>();
                Some(names)
            }
            false => None,
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
        match ctx.is_null() {
            true => Err(Box::new(WasmEdgeError::Store(StoreError::NotFoundModule(
                name.as_ref().to_string(),
            )))),
            false => {
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
unsafe impl Send for InnerStore {}
unsafe impl Sync for InnerStore {}
