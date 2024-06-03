//! Defines WasmEdge Table and TableType structs.
//!
//! A WasmEdge `Table` defines a WebAssembly table instance described by its `TableType`.
//! `TableType` specifies the limits on the size of a table. The start of
//! the limit range specifies the lower bound (inclusive) of the size, while
//! the end resticts the upper bound (inclusive).

use crate::{
    ffi::{self},
    types::{WasmEdgeLimit, WasmValue},
    utils::check,
    WasmEdgeResult,
};

use wasmedge_types::{
    error::{TableError, WasmEdgeError},
    RefType, ValType,
};

/// A WasmEdge [Table] defines a WebAssembly table instance described by its [type](crate::TableType). A table is an array-like structure and stores function references.
///
/// This [example](https://github.com/WasmEdge/WasmEdge/tree/master/bindings/rust/wasmedge-sys/examples/table_and_funcref.rs) shows how to use [Table] to store and retrieve function references.
#[derive(Debug)]
pub struct Table {
    pub(crate) inner: InnerTable,
}
impl Table {
    /// Creates a new [Table] to be associated with the given element type and the size.
    ///
    /// # Arguments
    ///
    /// - `ty` specifies the type of the new [Table].
    ///
    /// # Error
    ///
    /// * If fail to create the table instance, then WasmEdgeError::Table(TableError::Create)(crate::error::TableError) is returned.
    pub fn create(ty: wasmedge_types::TableType) -> WasmEdgeResult<Self> {
        let ty: TableType = ty.into();
        let ctx = unsafe { ffi::WasmEdge_TableInstanceCreate(ty.inner.0) };

        if ctx.is_null() {
            Err(Box::new(WasmEdgeError::Table(TableError::Create)))
        } else {
            Ok(Table {
                inner: InnerTable(ctx),
            })
        }
    }

    /// Returns the [TableType] of the [Table].
    ///
    /// # Error
    ///
    /// If fail to get type, then an error is returned.
    pub fn ty(&self) -> WasmEdgeResult<wasmedge_types::TableType> {
        let ty_ctx = unsafe { ffi::WasmEdge_TableInstanceGetTableType(self.inner.0) };
        if ty_ctx.is_null() {
            Err(Box::new(WasmEdgeError::Table(TableError::Type)))
        } else {
            let ty = std::mem::ManuallyDrop::new(TableType {
                inner: InnerTableType(ty_ctx as *mut _),
            });
            Ok((&*ty).into())
        }
    }

    /// Returns the element value at a specific position in the [Table].
    ///
    /// # Arguments
    ///
    /// - `idx` specifies the position in the [Table], at which the [WasmValue](crate::WasmValue) is returned.
    ///
    /// # Error
    ///
    /// If fail to get the data, then an error is returned.
    pub fn get_data(&self, idx: u32) -> WasmEdgeResult<WasmValue> {
        let raw_val = unsafe {
            let mut data = ffi::WasmEdge_ValueGenI32(0);
            check(ffi::WasmEdge_TableInstanceGetData(
                self.inner.0,
                &mut data as *mut _,
                idx,
            ))?;
            data
        };
        Ok(raw_val.into())
    }

    /// Sets a new element value at a specific position in the [Table].
    ///
    /// # Arguments
    ///
    /// - `data` specifies the new data.
    ///
    /// - `idx` specifies the position of the new data to be stored in the [Table].
    ///
    /// # Error
    ///
    /// If fail to set data, then an error is returned.
    pub fn set_data(&mut self, data: WasmValue, idx: u32) -> WasmEdgeResult<()> {
        unsafe {
            check(ffi::WasmEdge_TableInstanceSetData(
                self.inner.0,
                data.as_raw(),
                idx,
            ))
        }
    }

    /// Returns the capacity of the [Table].
    ///
    pub fn capacity(&self) -> usize {
        unsafe { ffi::WasmEdge_TableInstanceGetSize(self.inner.0) as usize }
    }

    /// Increases the capacity of the [Table].
    ///
    /// After growing, the new capacity must be in the range defined by `limit` when the table is created.
    ///
    /// # Argument
    ///
    /// - `size` specifies the size to be added to the [Table].
    ///
    /// # Error
    ///
    /// If fail to increase the size of the [Table], then an error is returned.
    pub fn grow(&mut self, size: u32) -> WasmEdgeResult<()> {
        unsafe { check(ffi::WasmEdge_TableInstanceGrow(self.inner.0, size)) }
    }

    /// # Safety
    ///
    /// The lifetime of the returned pointer must not exceed that of the object itself.
    pub unsafe fn as_ptr(&self) -> *const ffi::WasmEdge_TableInstanceContext {
        self.inner.0 as *const _
    }

    /// # Safety
    ///
    /// This function will take over the lifetime management of `ptr`, so do not call `ffi::WasmEdge_TableInstanceDelete` on `ptr` after this.
    pub unsafe fn from_raw(ptr: *mut ffi::WasmEdge_TableInstanceContext) -> Self {
        Self {
            inner: InnerTable(ptr),
        }
    }
}
impl Drop for Table {
    fn drop(&mut self) {
        unsafe { ffi::WasmEdge_TableInstanceDelete(self.inner.0) };
    }
}

#[derive(Debug)]
pub(crate) struct InnerTable(pub(crate) *mut ffi::WasmEdge_TableInstanceContext);
unsafe impl Send for InnerTable {}
unsafe impl Sync for InnerTable {}

/// A WasmEdge [TableType] classifies a [Table] instance over elements of element types within a size range.
#[derive(Debug)]
pub(crate) struct TableType {
    pub(crate) inner: InnerTableType,
}
impl Drop for TableType {
    fn drop(&mut self) {
        unsafe {
            ffi::WasmEdge_TableTypeDelete(self.inner.0);
        }
    }
}
impl TableType {
    /// Creates a new [TableType] to be associated with the given limit range of the size and the reference type.
    ///
    /// # Arguments
    ///
    /// * `elem_type` - The element type.
    ///
    /// * `min` - The initial size of the table to be created.
    ///
    /// * `max` - The maximum size of the table to be created.
    ///
    /// # Error
    ///
    /// If fail to create a [TableType], then an error is returned.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let ty = TableType::create(WasmRefType::FuncRef, 10, Some(20)).expect("fail to create a TableType");
    /// ```
    ///
    pub(crate) fn create(elem_ty: RefType, min: u32, max: Option<u32>) -> WasmEdgeResult<Self> {
        let ty: ValType = elem_ty.into();
        let ctx = unsafe {
            ffi::WasmEdge_TableTypeCreate(ty.into(), WasmEdgeLimit::new(min, max, false).into())
        };
        if ctx.is_null() {
            Err(Box::new(WasmEdgeError::TableTypeCreate))
        } else {
            Ok(Self {
                inner: InnerTableType(ctx),
            })
        }
    }

    /// Returns the element type.
    pub(crate) fn elem_ty(&self) -> RefType {
        let ty = unsafe { ffi::WasmEdge_TableTypeGetRefType(self.inner.0) };
        let ty: ValType = ty.into();
        ty.into()
    }

    /// Returns the initial size of the [Table].
    pub(crate) fn min(&self) -> u32 {
        let limit = unsafe { ffi::WasmEdge_TableTypeGetLimit(self.inner.0) };
        let limit: WasmEdgeLimit = limit.into();
        limit.min()
    }

    /// Returns the maximum size of the [Table].
    pub(crate) fn max(&self) -> Option<u32> {
        let limit = unsafe { ffi::WasmEdge_TableTypeGetLimit(self.inner.0) };
        let limit: WasmEdgeLimit = limit.into();
        limit.max()
    }
}
impl From<wasmedge_types::TableType> for TableType {
    fn from(ty: wasmedge_types::TableType) -> Self {
        TableType::create(ty.elem_ty(), ty.minimum(), ty.maximum()).expect(
            "[wasmedge] Failed to convert wasmedge_types::TableType into wasmedge_sys::TableType.",
        )
    }
}
impl From<&TableType> for wasmedge_types::TableType {
    fn from(ty: &TableType) -> Self {
        wasmedge_types::TableType::new(ty.elem_ty(), ty.min(), ty.max())
    }
}

#[derive(Debug)]
pub(crate) struct InnerTableType(pub(crate) *mut ffi::WasmEdge_TableTypeContext);
unsafe impl Send for InnerTableType {}
unsafe impl Sync for InnerTableType {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{instance::function::AsFunc, CallingFrame, Function, Instance};
    use std::thread;
    use wasmedge_types::{
        error::{CoreError, CoreExecutionError},
        RefType, ValType,
    };

    #[test]
    #[allow(clippy::assertions_on_result_states)]
    fn test_table_type() {
        // create a TableType instance
        let result = TableType::create(RefType::FuncRef, 10, Some(20));
        assert!(result.is_ok());
        let ty = result.unwrap();
        assert!(!ty.inner.0.is_null());

        // check element type
        assert_eq!(ty.elem_ty(), RefType::FuncRef);
        // check limit
        assert_eq!(ty.min(), 10);
        assert_eq!(ty.max(), Some(20));
    }

    #[test]
    #[allow(clippy::assertions_on_result_states)]
    fn test_table() {
        // create a TableType instance
        let ty = wasmedge_types::TableType::new(RefType::FuncRef, 10, Some(20));

        // create a Table instance
        let result = Table::create(ty);
        assert!(result.is_ok());
        let mut table = result.unwrap();

        // check capacity
        assert_eq!(table.capacity(), 10);

        // get type
        let result = table.ty();
        assert!(result.is_ok());
        let ty = result.unwrap();

        // check limit and element type
        assert_eq!(ty.minimum(), 10);
        assert_eq!(ty.maximum(), Some(20));
        assert_eq!(ty.elem_ty(), RefType::FuncRef);

        // grow the capacity of table
        let result = table.grow(5);
        assert!(result.is_ok());
        // check capacity
        assert_eq!(table.capacity(), 15);
    }

    #[test]
    #[allow(clippy::assertions_on_result_states)]
    fn test_table_data() {
        // create a FuncType
        let func_ty = wasmedge_types::FuncType::new(vec![ValType::I32; 2], vec![ValType::I32]);
        // create a host function
        let mut host_data = ();
        let result =
            unsafe { Function::create_sync_func::<()>(&func_ty, real_add, &mut host_data, 0) };
        assert!(result.is_ok());
        let host_func = result.unwrap();

        // create a TableType instance
        let ty = wasmedge_types::TableType::new(RefType::FuncRef, 10, Some(20));

        // create a Table instance
        let result = Table::create(ty);
        assert!(result.is_ok());
        let mut table = result.unwrap();

        // check capacity
        assert_eq!(table.capacity(), 10);

        // get data in the scope of the capacity
        let result = table.get_data(9);
        assert!(result.is_ok());
        let value = result.unwrap();
        assert!(value.is_null_ref());
        assert_eq!(value.ty(), ValType::FuncRef);

        // call set_data to store a function reference at the given index of the table instance
        let result = table.set_data(WasmValue::from_func_ref((&host_func).into()), 3);
        assert!(result.is_ok());
        // call get_data to recover the function reference from the value at the given index of the table instance
        let result = table.get_data(3);
        assert!(result.is_ok());
        let value = result.unwrap();
        let result = value.func_ref();
        assert!(result.is_some());
        let func_ref = result.unwrap();

        // get the function type by func_ref
        let result = func_ref.ty();
        assert!(result.is_some());
        let func_ty = result.unwrap();
        assert_eq!(func_ty.args_len(), 2);
        assert_eq!(func_ty.args(), &[ValType::I32, ValType::I32]);
        assert_eq!(func_ty.returns_len(), 1);
        assert_eq!(func_ty.returns(), &[ValType::I32]);
    }

    #[test]
    fn test_table_send() {
        // create a TableType instance
        let ty = wasmedge_types::TableType::new(RefType::FuncRef, 10, Some(20));

        // create a Table instance
        let result = Table::create(ty);
        assert!(result.is_ok());
        let table = result.unwrap();

        let handle = thread::spawn(move || {
            assert!(!table.inner.0.is_null());

            // check capacity
            assert_eq!(table.capacity(), 10);

            // get type
            let result = table.ty();
            assert!(result.is_ok());
            let ty = result.unwrap();

            // check limit and element type
            assert_eq!(ty.minimum(), 10);
            assert_eq!(ty.maximum(), Some(20));
            assert_eq!(ty.elem_ty(), RefType::FuncRef);
        });

        handle.join().unwrap();
    }

    #[test]
    fn test_table_sync() {
        // create a TableType instance
        let ty = wasmedge_types::TableType::new(RefType::FuncRef, 10, Some(20));

        // create a Table instance
        let result = Table::create(ty);

        assert!(result.is_ok());
        let table = result.unwrap();

        let table = &table;

        std::thread::scope(move |s| {
            let _ = s
                .spawn(|| {
                    let result = table.ty();
                    assert!(result.is_ok());
                    let ty = result.unwrap();
                    // check limit and element type
                    assert_eq!(ty.minimum(), 10);
                    assert_eq!(ty.maximum(), Some(20));
                    assert_eq!(ty.elem_ty(), RefType::FuncRef);
                })
                .join();
        });
    }

    fn real_add(
        _data: &mut (),
        _inst: &mut Instance,
        _frame: &mut CallingFrame,
        input: Vec<WasmValue>,
    ) -> Result<Vec<WasmValue>, CoreError> {
        println!("Rust: Entering Rust function real_add");

        if input.len() != 2 {
            return Err(CoreError::Execution(CoreExecutionError::FuncSigMismatch));
        }

        let a = if input[0].ty() == ValType::I32 {
            input[0].to_i32()
        } else {
            return Err(CoreError::Execution(CoreExecutionError::FuncSigMismatch));
        };

        let b = if input[1].ty() == ValType::I32 {
            input[0].to_i32()
        } else {
            return Err(CoreError::Execution(CoreExecutionError::FuncSigMismatch));
        };

        let c = a + b;
        println!("Rust: calcuating in real_add c: {c:?}");

        println!("Rust: Leaving Rust function real_add");
        Ok(vec![WasmValue::from_i32(c)])
    }
}
