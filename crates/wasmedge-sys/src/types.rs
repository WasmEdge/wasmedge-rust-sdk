//! Defines the WebAssembly primitive types.

use crate::{ffi, instance::function::AsFunc, FuncRef, Function};
use core::ffi::c_void;
use std::ffi::CString;
use wasmedge_types::ValType;

#[derive(Debug, Clone)]
pub(crate) struct WasmEdgeLimit {
    min: u32,
    max: Option<u32>,
    shared: bool,
}
impl WasmEdgeLimit {
    pub(crate) fn new(min: u32, max: Option<u32>, shared: bool) -> Self {
        Self { min, max, shared }
    }

    pub(crate) fn min(&self) -> u32 {
        self.min
    }

    pub(crate) fn max(&self) -> Option<u32> {
        self.max
    }

    pub(crate) fn shared(&self) -> bool {
        self.shared
    }
}
impl From<WasmEdgeLimit> for ffi::WasmEdge_Limit {
    fn from(limit: WasmEdgeLimit) -> Self {
        let max = match limit.max() {
            Some(max) => max,
            None => u32::MAX,
        };

        Self {
            Min: limit.min(),
            Max: max,
            HasMax: limit.max().is_some(),
            Shared: limit.shared,
        }
    }
}
impl From<ffi::WasmEdge_Limit> for WasmEdgeLimit {
    fn from(limit: ffi::WasmEdge_Limit) -> Self {
        let max = match limit.HasMax {
            true => Some(limit.Max),
            false => None,
        };
        WasmEdgeLimit::new(limit.Min, max, limit.Shared)
    }
}

/// Struct of WasmEdge String.
#[derive(Debug)]
pub struct WasmEdgeString {
    inner: InnerWasmEdgeString,
}
impl Drop for WasmEdgeString {
    fn drop(&mut self) {
        unsafe { ffi::WasmEdge_StringDelete(self.inner.0) }
    }
}
impl WasmEdgeString {
    pub(crate) fn as_raw(&self) -> ffi::WasmEdge_String {
        self.inner.0
    }

    /// # Safety
    ///
    /// After calling this function, the caller is responsible for managing the lifetime of `ffi::WasmEdge_String`` and should call `ffi::WasmEdge_StringDelete`` at the appropriate time.
    pub unsafe fn into_raw(self) -> ffi::WasmEdge_String {
        let s = self.inner.0;
        std::mem::forget(self);
        s
    }

    /// # Safety
    ///
    /// This function takes ownership of `s`, so do not call `ffi::WasmEdge_StringDelete` on `s` after this.
    pub unsafe fn from_raw(s: ffi::WasmEdge_String) -> Self {
        Self {
            inner: InnerWasmEdgeString(s),
        }
    }
}
impl<T: AsRef<str>> From<T> for WasmEdgeString {
    fn from(s: T) -> Self {
        let ctx = if s.as_ref().contains('\0') {
            let buffer = s.as_ref().as_bytes();
            unsafe {
                ffi::WasmEdge_StringCreateByBuffer(buffer.as_ptr() as *const _, buffer.len() as u32)
            }
        } else {
            let cs = CString::new(s.as_ref()).expect(
                "Failed to create a CString: the supplied bytes contain an internal 0 byte",
            );
            unsafe { ffi::WasmEdge_StringCreateByCString(cs.as_ptr()) }
        };

        Self {
            inner: InnerWasmEdgeString(ctx),
        }
    }
}
impl PartialEq for WasmEdgeString {
    fn eq(&self, other: &Self) -> bool {
        unsafe { ffi::WasmEdge_StringIsEqual(self.inner.0, other.inner.0) }
    }
}
impl Eq for WasmEdgeString {}

impl AsRef<ffi::WasmEdge_String> for WasmEdgeString {
    fn as_ref(&self) -> &ffi::WasmEdge_String {
        &self.inner.0
    }
}
impl AsMut<ffi::WasmEdge_String> for WasmEdgeString {
    fn as_mut(&mut self) -> &mut ffi::WasmEdge_String {
        &mut self.inner.0
    }
}

impl From<&WasmEdgeString> for String {
    fn from(s: &WasmEdgeString) -> Self {
        s.as_ref().into()
    }
}
impl From<&WasmEdgeString> for &std::ffi::CStr {
    fn from(s: &WasmEdgeString) -> Self {
        s.as_ref().into()
    }
}
impl From<WasmEdgeString> for String {
    fn from(s: WasmEdgeString) -> Self {
        s.as_ref().into()
    }
}
impl From<WasmEdgeString> for &std::ffi::CStr {
    fn from(s: WasmEdgeString) -> Self {
        s.as_ref().into()
    }
}

impl From<&ffi::WasmEdge_String> for String {
    fn from(s: &ffi::WasmEdge_String) -> Self {
        let cstr = unsafe { std::slice::from_raw_parts(s.Buf as *mut u8, s.Length as usize) };
        String::from_utf8(cstr.to_vec()).unwrap_or_default()
    }
}
impl From<&ffi::WasmEdge_String> for &std::ffi::CStr {
    fn from(s: &ffi::WasmEdge_String) -> Self {
        unsafe { std::ffi::CStr::from_ptr(s.Buf as *const _) }
    }
}
impl From<ffi::WasmEdge_String> for String {
    fn from(s: ffi::WasmEdge_String) -> Self {
        (&s).into()
    }
}
impl From<ffi::WasmEdge_String> for &std::ffi::CStr {
    fn from(s: ffi::WasmEdge_String) -> Self {
        (&s).into()
    }
}

#[derive(Debug)]
pub(crate) struct InnerWasmEdgeString(pub(crate) ffi::WasmEdge_String);
unsafe impl Send for InnerWasmEdgeString {}
unsafe impl Sync for InnerWasmEdgeString {}

/// Defines a WebAssembly value.
#[derive(Debug, Clone, Copy)]
pub struct WasmValue {
    ctx: ffi::WasmEdge_Value,
    ty: ValType,
}
impl WasmValue {
    /// Returns the raw `WasmEdge_Value`.
    pub fn as_raw(&self) -> ffi::WasmEdge_Value {
        self.ctx
    }

    /// Returns the type of a [WasmValue].
    pub fn ty(&self) -> ValType {
        self.ty
    }

    /// Creates a [WasmValue] from a `i32` value.
    ///
    /// # Argument
    ///
    /// * `val` - The source `i32` value.
    pub fn from_i32(val: i32) -> Self {
        Self {
            ctx: unsafe { ffi::WasmEdge_ValueGenI32(val) },
            ty: ValType::I32,
        }
    }

    /// Generates a `i32` value from a [WasmValue].
    pub fn to_i32(&self) -> i32 {
        unsafe { ffi::WasmEdge_ValueGetI32(self.ctx) }
    }

    /// Creates a [WasmValue] from a `i64` value.
    ///
    /// # Argument
    ///
    /// * `val` - The source `i64` value.
    pub fn from_i64(val: i64) -> Self {
        Self {
            ctx: unsafe { ffi::WasmEdge_ValueGenI64(val) },
            ty: ValType::I64,
        }
    }

    /// Generates a `i64` value from a [WasmValue].
    pub fn to_i64(&self) -> i64 {
        unsafe { ffi::WasmEdge_ValueGetI64(self.ctx) }
    }

    /// Creates a [WasmValue] from a `f32` value.
    ///
    /// # Argument
    ///
    /// * `val` - The source `f32` value.
    pub fn from_f32(val: f32) -> Self {
        Self {
            ctx: unsafe { ffi::WasmEdge_ValueGenF32(val) },
            ty: ValType::F32,
        }
    }

    /// Generates a `f32` value from a [WasmValue].
    pub fn to_f32(&self) -> f32 {
        unsafe { ffi::WasmEdge_ValueGetF32(self.ctx) }
    }

    /// Creates a [WasmValue] from a `f64` value.
    ///
    /// # Argument
    ///
    /// * `val` - The source `f64` value.
    pub fn from_f64(val: f64) -> Self {
        Self {
            ctx: unsafe { ffi::WasmEdge_ValueGenF64(val) },
            ty: ValType::F64,
        }
    }

    /// Generates a `f64` value from a [WasmValue].
    pub fn to_f64(&self) -> f64 {
        unsafe { ffi::WasmEdge_ValueGetF64(self.ctx) }
    }

    /// Creates a [WasmValue] from a `i128` value.
    ///
    /// # Argument
    ///
    /// * `val` - The source `i128` value.
    pub fn from_v128(val: i128) -> Self {
        Self {
            ctx: unsafe { ffi::WasmEdge_ValueGenV128(val) },
            ty: ValType::V128,
        }
    }

    /// Generates a `v128` value from a [WasmValue].
    pub fn to_v128(&self) -> i128 {
        unsafe { ffi::WasmEdge_ValueGetV128(self.ctx) }
    }

    /// Checks if a [WasmValue] is NullRef or not.
    pub fn is_null_ref(&self) -> bool {
        unsafe { ffi::WasmEdge_ValueIsNullRef(self.ctx) }
    }

    /// Creates a [WasmValue] from a [FuncRef](crate::FuncRef).
    ///
    /// Notice that the [WasmValue]s generated from [FuncRef](crate::FuncRef)s are only meaningful when the `bulk_memory_operations` or `reference_types` option is enabled in the [Config](crate::Config).
    ///
    /// # Argument
    ///
    /// * `func_ref` - A [FuncRef] instance.
    pub fn from_func_ref<Func: AsFunc>(func_ref: &Func) -> Self {
        Self {
            ctx: unsafe { ffi::WasmEdge_ValueGenFuncRef(func_ref.get_func_raw()) },
            ty: ValType::FuncRef,
        }
    }

    /// Returns the FuncRef(crate::FuncRef).
    ///
    /// If the [WasmValue] is a `NullRef`, then `None` is returned.
    pub fn func_ref(&self) -> Option<FuncRef<&Self>> {
        unsafe {
            match ffi::WasmEdge_ValueIsNullRef(self.ctx) {
                true => None,
                false => {
                    let ctx = ffi::WasmEdge_ValueGetFuncRef(self.ctx);
                    let f = Function::from_raw(ctx as _);
                    Some(FuncRef::create_from_ref(
                        std::mem::ManuallyDrop::new(f),
                        self,
                    ))
                }
            }
        }
    }

    /// Creates a [WasmValue] from a reference to an external object.
    ///
    /// The [WasmValue]s generated by this function are only meaningful when the `reference_types` option is enabled in
    /// the [Config](crate::Config).
    ///
    /// # Argument
    ///
    /// * `extern_obj` - The reference to an external object.
    pub fn from_extern_ref<T>(extern_obj: &mut T) -> Self {
        let ptr = extern_obj as *mut T as *mut c_void;
        Self {
            ctx: unsafe { ffi::WasmEdge_ValueGenExternRef(ptr) },
            ty: ValType::ExternRef,
        }
    }

    /// Creates a null [WasmValue] with a [ExternRef](wasmedge_types::RefType::ExternRef).
    ///
    /// # Argument
    ///
    /// * `val` - The `[`RefType`] value.
    pub fn null_extern_ref() -> Self {
        Self {
            ctx: unsafe { ffi::WasmEdge_ValueGenExternRef(std::ptr::null_mut()) },
            ty: ValType::ExternRef,
        }
    }

    /// Returns the reference to an external object.
    ///
    /// If the [WasmValue] is a `NullRef`, then `None` is returned.
    pub fn extern_ref<T>(&self) -> Option<&T> {
        unsafe {
            match ffi::WasmEdge_ValueIsNullRef(self.ctx) {
                true => None,
                false => {
                    let ptr = ffi::WasmEdge_ValueGetExternRef(self.ctx);
                    let x = ptr as *mut T;
                    Some(&*x)
                }
            }
        }
    }
}
impl From<ffi::WasmEdge_Value> for WasmValue {
    fn from(raw_val: ffi::WasmEdge_Value) -> Self {
        let ty = raw_val.Type.into();
        Self { ctx: raw_val, ty }
    }
}

impl From<ffi::WasmEdge_ValType> for ValType {
    fn from(value: ffi::WasmEdge_ValType) -> Self {
        unsafe {
            if ffi::WasmEdge_ValTypeIsI32(value) {
                ValType::I32
            } else if ffi::WasmEdge_ValTypeIsI64(value) {
                ValType::I64
            } else if ffi::WasmEdge_ValTypeIsF32(value) {
                ValType::F32
            } else if ffi::WasmEdge_ValTypeIsF64(value) {
                ValType::F64
            } else if ffi::WasmEdge_ValTypeIsV128(value) {
                ValType::V128
            } else if ffi::WasmEdge_ValTypeIsRef(value) {
                if ffi::WasmEdge_ValTypeIsFuncRef(value) {
                    ValType::FuncRef
                } else if ffi::WasmEdge_ValTypeIsExternRef(value) {
                    ValType::ExternRef
                } else {
                    log::warn!(
                        "capi unsupport WasmEdge_RefType `{:x}`",
                        u64::from_be_bytes(value.Data)
                    );
                    ValType::UnsupportedRef
                }
            } else {
                log::warn!(
                    "unknown WasmEdge_ValType `{:x}`",
                    u64::from_be_bytes(value.Data)
                );
                ValType::UnsupportedRef
            }
        }
    }
}
impl From<ValType> for ffi::WasmEdge_ValType {
    fn from(value: ValType) -> Self {
        unsafe {
            match value {
                ValType::I32 => ffi::WasmEdge_ValTypeGenI32(),
                ValType::I64 => ffi::WasmEdge_ValTypeGenI64(),
                ValType::F32 => ffi::WasmEdge_ValTypeGenF32(),
                ValType::F64 => ffi::WasmEdge_ValTypeGenF64(),
                ValType::V128 => ffi::WasmEdge_ValTypeGenV128(),
                ValType::FuncRef => ffi::WasmEdge_ValTypeGenFuncRef(),
                ValType::ExternRef => ffi::WasmEdge_ValTypeGenExternRef(),
                // C API is temporarily unsupported.
                ValType::UnsupportedRef => ffi::WasmEdge_ValTypeGenExternRef(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(unix)]
    use crate::{Table, TableType};
    use std::{
        sync::{Arc, Mutex},
        thread,
    };
    #[cfg(unix)]
    use wasmedge_types::RefType;

    #[test]
    #[cfg(unix)]
    fn test_types_value() {
        // I32
        let val = WasmValue::from_i32(1314);
        assert_eq!(val.ty(), ValType::I32);

        // I64
        let val = WasmValue::from_i64(1314);
        assert_eq!(val.ty(), ValType::I64);

        // F32
        let val = WasmValue::from_f32(13.14);
        assert_eq!(val.ty(), ValType::F32);

        // F64
        let val = WasmValue::from_f64(13.14);
        assert_eq!(val.ty(), ValType::F64);

        // V128
        let val = WasmValue::from_v128(1314);
        assert_eq!(val.ty(), ValType::V128);

        // ExternRef
        let ty = TableType::new(RefType::FuncRef, 10, Some(20));
        let result = Table::create(ty);
        assert!(result.is_ok());
        let mut table = result.unwrap();
        let value = WasmValue::from_extern_ref(&mut table);
        assert_eq!(value.ty(), ValType::ExternRef);
        assert!(value.extern_ref::<Table>().is_some());

        let val1 = WasmValue::from_i32(1314);
        let val2 = WasmValue::from_i32(1314);
        assert_eq!(val1.to_i32(), val2.to_i32());
    }

    #[test]
    #[cfg(unix)]
    fn test_types_value_send() {
        // I32
        let val_i32 = WasmValue::from_i32(1314);
        // I64
        let val_i64 = WasmValue::from_i64(1314);
        // F32
        let val_f32 = WasmValue::from_f32(13.14);
        // F64
        let val_f64 = WasmValue::from_f64(13.14);
        // V128
        let val_v128 = WasmValue::from_v128(1314);

        // ExternRef
        let ty = TableType::new(RefType::FuncRef, 10, Some(20));
        let result = Table::create(ty);
        assert!(result.is_ok());
        let mut table = result.unwrap();
        let val_extern_ref = WasmValue::from_extern_ref(&mut table);

        let handle = thread::spawn(move || {
            let val_i32_c = val_i32;
            assert_eq!(val_i32_c.ty(), ValType::I32);

            let val_i64_c = val_i64;
            assert_eq!(val_i64_c.ty(), ValType::I64);

            let val_f32_c = val_f32;
            assert_eq!(val_f32_c.ty(), ValType::F32);

            let val_f64_c = val_f64;
            assert_eq!(val_f64_c.ty(), ValType::F64);

            let val_v128_c = val_v128;
            assert_eq!(val_v128_c.ty(), ValType::V128);

            let val_extern_ref_c = val_extern_ref;
            assert_eq!(val_extern_ref_c.ty(), ValType::ExternRef);
            assert!(val_extern_ref_c.extern_ref::<Table>().is_some());
        });

        handle.join().unwrap();
    }

    #[test]
    #[cfg(unix)]
    fn test_types_value_sync() {
        // I32
        let val_i32 = Arc::new(Mutex::new(WasmValue::from_i32(1314)));
        let val_i32_cloned = Arc::clone(&val_i32);

        // I64
        let val_i64 = Arc::new(Mutex::new(WasmValue::from_i64(1314)));
        let val_i64_cloned = Arc::clone(&val_i64);

        // F32
        let val_f32 = Arc::new(Mutex::new(WasmValue::from_f32(13.14)));
        let val_f32_cloned = Arc::clone(&val_f32);

        // F64
        let val_f64 = Arc::new(Mutex::new(WasmValue::from_f64(13.14)));
        let val_f64_cloned = Arc::clone(&val_f64);

        // V128
        let val_v128 = Arc::new(Mutex::new(WasmValue::from_v128(1314)));
        let val_v128_cloned = Arc::clone(&val_v128);

        // ExternRef
        let ty = TableType::new(RefType::FuncRef, 10, Some(20));
        let result = Table::create(ty);
        assert!(result.is_ok());
        let mut table = result.unwrap();
        let val_extern_ref = Arc::new(Mutex::new(WasmValue::from_extern_ref(&mut table)));
        let val_extern_ref_cloned = Arc::clone(&val_extern_ref);

        let handle = thread::spawn(move || {
            let result = val_i32_cloned.lock();
            assert!(result.is_ok());
            let val_i32_c = result.unwrap();
            assert_eq!(val_i32_c.ty(), ValType::I32);

            let result = val_i64_cloned.lock();
            assert!(result.is_ok());
            let val_i64_c = result.unwrap();
            assert_eq!(val_i64_c.ty(), ValType::I64);

            let result = val_f32_cloned.lock();
            assert!(result.is_ok());
            let val_f32_c = result.unwrap();
            assert_eq!(val_f32_c.ty(), ValType::F32);

            let result = val_f64_cloned.lock();
            assert!(result.is_ok());
            let val_f64_c = result.unwrap();
            assert_eq!(val_f64_c.ty(), ValType::F64);

            let result = val_v128_cloned.lock();
            assert!(result.is_ok());
            let val_v128_c = result.unwrap();
            assert_eq!(val_v128_c.ty(), ValType::V128);

            let result = val_extern_ref_cloned.lock();
            assert!(result.is_ok());
            let val_extern_ref_c = result.unwrap();
            assert_eq!(val_extern_ref_c.ty(), ValType::ExternRef);
            assert!(val_extern_ref_c.extern_ref::<Table>().is_some());
        });

        handle.join().unwrap();
    }

    #[test]
    fn test_types_string() {
        let s: WasmEdgeString = "hello".into();
        let t: WasmEdgeString = "hello".into();
        assert_eq!(s, t);

        let s: WasmEdgeString = "hello".into();
        let t = String::from(s);
        assert_eq!(t, "hello");

        let s: WasmEdgeString = "hello".into();
        let t: WasmEdgeString = "hello\0".into();
        assert_ne!(s, t);

        let s: WasmEdgeString = "hello\0".into();
        let t = String::from(s);
        assert_eq!(t, "hello\0");

        let s: WasmEdgeString = "he\0llo\0".into();
        let t = String::from(s);
        assert_eq!(t, "he\0llo\0");
    }

    #[test]
    fn test_types_string_send() {
        let s: WasmEdgeString = "hello".into();

        let handle = thread::spawn(move || {
            let t: WasmEdgeString = "hello".into();
            assert_eq!(s, t);
        });

        handle.join().unwrap();
    }

    #[test]
    fn test_types_string_sync() {
        let s: WasmEdgeString = "hello".into();
        let p = Arc::new(Mutex::new(s));

        let s_cloned = Arc::clone(&p);
        let handle = thread::spawn(move || {
            let result = s_cloned.lock();
            assert!(result.is_ok());
            let s = result.unwrap();

            let t: WasmEdgeString = "hello".into();
            assert_eq!(*s, t);
        });

        handle.join().unwrap();
    }
}
