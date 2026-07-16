#![doc(
    html_logo_url = "https://github.com/cncf/artwork/blob/master/projects/wasm-edge-runtime/icon/color/wasm-edge-runtime-icon-color.png?raw=true",
    html_favicon_url = "https://raw.githubusercontent.com/cncf/artwork/49169bdbc88a7ce3c4a722c641cc2d548bd5c340/projects/wasm-edge-runtime/icon/color/wasm-edge-runtime-icon-color.svg"
)]

//! The [wasmedge-types](https://crates.io/crates/wasmedge-types) crate defines a group of common data structures used by both [wasmedge-sdk](https://crates.io/crates/wasmedge-sdk) and [wasmedge-sys](https://crates.io/crates/wasmedge-sys) crates.
//!
//! See also
//!
//! * [WasmEdge Runtime](https://wasmedge.org/)

pub mod error;

use error::TryFromIntError;

/// Defines WasmEdge reference types.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum RefType {
    /// Refers to the infinite union of all references to host functions, regardless of their function types.
    FuncRef,

    /// Refers to the infinite union of all references to objects and that can be passed into WebAssembly under this type.
    ExternRef,
}

impl From<ValType> for RefType {
    fn from(value: ValType) -> Self {
        match value {
            ValType::FuncRef => RefType::FuncRef,
            ValType::ExternRef => RefType::ExternRef,
            _ => panic!("[wasmedge-types] Invalid WasmEdge_RefType: {value:#X?}"),
        }
    }
}

impl From<RefType> for ValType {
    fn from(value: RefType) -> Self {
        match value {
            RefType::FuncRef => ValType::FuncRef,
            RefType::ExternRef => ValType::ExternRef,
        }
    }
}

/// Defines WasmEdge value types.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ValType {
    /// 32-bit integer.
    ///
    /// Integers are not inherently signed or unsigned, their interpretation is determined by individual operations.
    I32,
    /// 64-bit integer.
    ///
    /// Integers are not inherently signed or unsigned, their interpretation is determined by individual operations.
    I64,
    /// 32-bit floating-point data as defined by the [IEEE 754-2019](https://ieeexplore.ieee.org/document/8766229).
    F32,
    /// 64-bit floating-point data as defined by the [IEEE 754-2019](https://ieeexplore.ieee.org/document/8766229).
    F64,
    /// 128-bit vector of packed integer or floating-point data.
    ///
    /// The packed data can be interpreted as signed or unsigned integers, single or double precision floating-point
    /// values, or a single 128 bit type. The interpretation is determined by individual operations.
    V128,
    /// A reference to a host function.
    FuncRef,
    /// A reference to object.
    ExternRef,
    /// A reference that unsupported by c-api.
    UnsupportedRef,
}

/// Defines the mutability property of WasmEdge Global variables.
///
/// `Mutability` determines the mutability property of a WasmEdge Global variable is either mutable or immutable.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Mutability {
    /// Identifies an immutable global variable.
    Const,
    /// Identifies a mutable global variable.
    Var,
}
/// Generates the `From<u32>`/`From<i32>` conversions (panicking) and their infallible inverses
/// (`From<$enum_ty> for u32`/`i32`), plus non-panicking `TryFrom<u32>`/`TryFrom<i32>`
/// counterparts, for a simple enum whose variants map 1:1 to small integer discriminants.
///
/// `$panic_msg` is the exact `panic!` format string used by the pre-existing hand-written
/// `From<u32>`/`From<i32>` impls, preserved byte-for-byte (including the choice of decimal vs.
/// `{:#X}` hex formatting) so this dedup changes no observable behavior.
///
/// N.B. the non-panicking counterparts are exposed as inherent `try_from_u32`/`try_from_i32`
/// associated functions rather than `impl TryFrom<u32>`/`impl TryFrom<i32>`. A hand-written
/// `TryFrom<u32> for $enum_ty` is rejected by rustc (E0119) once `From<u32> for $enum_ty`
/// exists, because it conflicts with the standard library's blanket
/// `impl<T, U> TryFrom<U> for T where U: Into<T>` (which is already satisfied transitively via
/// `From` -> `Into`). Keeping the panicking `From` impls (required, zero behavior change) rules
/// out the literal `TryFrom` trait for the fallible counterpart.
macro_rules! impl_int_enum_conversions {
    (
        $enum_ty:ident { $($variant:ident = $val:literal),+ $(,)? },
        $panic_msg:literal
    ) => {
        /// Converts an integer discriminant into the target enum.
        ///
        /// # Panics
        ///
        /// Panics if `value` does not correspond to a known variant. Use
        #[doc = concat!("[`", stringify!($enum_ty), "::try_from_u32`]")]
        /// for a non-panicking alternative.
        impl From<u32> for $enum_ty {
            fn from(value: u32) -> Self {
                match value {
                    $($val => $enum_ty::$variant,)+
                    _ => panic!($panic_msg, value),
                }
            }
        }
        impl From<$enum_ty> for u32 {
            fn from(value: $enum_ty) -> Self {
                match value {
                    $($enum_ty::$variant => $val,)+
                }
            }
        }
        /// Converts an integer discriminant into the target enum.
        ///
        /// # Panics
        ///
        /// Panics if `value` does not correspond to a known variant. Use
        #[doc = concat!("[`", stringify!($enum_ty), "::try_from_i32`]")]
        /// for a non-panicking alternative.
        impl From<i32> for $enum_ty {
            fn from(value: i32) -> Self {
                match value {
                    $($val => $enum_ty::$variant,)+
                    _ => panic!($panic_msg, value),
                }
            }
        }
        impl From<$enum_ty> for i32 {
            fn from(value: $enum_ty) -> Self {
                match value {
                    $($enum_ty::$variant => $val,)+
                }
            }
        }

        impl $enum_ty {
            /// Attempts to convert a `u32` discriminant into
            #[doc = concat!("[`", stringify!($enum_ty), "`].")]
            ///
            /// Returns [`Err`] instead of panicking when `value` does not correspond to a
            /// known variant (unlike [`From<u32>`](From)).
            pub fn try_from_u32(value: u32) -> Result<Self, TryFromIntError> {
                match value {
                    $($val => Ok($enum_ty::$variant),)+
                    _ => Err(TryFromIntError::new(value as i64, stringify!($enum_ty))),
                }
            }

            /// Attempts to convert an `i32` discriminant into
            #[doc = concat!("[`", stringify!($enum_ty), "`].")]
            ///
            /// Returns [`Err`] instead of panicking when `value` does not correspond to a
            /// known variant (unlike [`From<i32>`](From)).
            pub fn try_from_i32(value: i32) -> Result<Self, TryFromIntError> {
                match value {
                    $($val => Ok($enum_ty::$variant),)+
                    _ => Err(TryFromIntError::new(value as i64, stringify!($enum_ty))),
                }
            }
        }
    };
}

impl_int_enum_conversions!(
    Mutability {
        Const = 0,
        Var = 1,
    },
    "[wasmedge-types] Invalid WasmEdge_Mutability: {:#X}"
);

/// Defines WasmEdge AOT compiler optimization level.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompilerOptimizationLevel {
    /// Disable as many optimizations as possible.
    O0,

    /// Optimize quickly without destroying debuggability.
    O1,

    /// Optimize for fast execution as much as possible without triggering significant incremental compile time or code size growth.
    O2,

    ///  Optimize for fast execution as much as possible.
    O3,

    ///  Optimize for small code size as much as possible without triggering
    ///  significant incremental compile time or execution time slowdowns.
    Os,

    /// Optimize for small code size as much as possible.
    Oz,
}
impl_int_enum_conversions!(
    CompilerOptimizationLevel {
        O0 = 0,
        O1 = 1,
        O2 = 2,
        O3 = 3,
        Os = 4,
        Oz = 5,
    },
    "Unknown CompilerOptimizationLevel value: {}"
);

/// Defines WasmEdge AOT compiler output binary format.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompilerOutputFormat {
    /// Native dynamic library format.
    Native,

    /// WebAssembly with AOT compiled codes in custom sections.
    Wasm,
}
impl_int_enum_conversions!(
    CompilerOutputFormat {
        Native = 0,
        Wasm = 1,
    },
    "Unknown CompilerOutputFormat value: {}"
);

#[cfg(test)]
mod int_enum_conversions_tests {
    use super::*;

    #[test]
    fn mutability_round_trip() {
        assert_eq!(Mutability::from(0u32), Mutability::Const);
        assert_eq!(Mutability::from(1u32), Mutability::Var);
        assert_eq!(Mutability::from(0i32), Mutability::Const);
        assert_eq!(Mutability::from(1i32), Mutability::Var);
        assert_eq!(u32::from(Mutability::Const), 0);
        assert_eq!(u32::from(Mutability::Var), 1);
        assert_eq!(i32::from(Mutability::Const), 0);
        assert_eq!(i32::from(Mutability::Var), 1);
    }

    #[test]
    #[should_panic(expected = "[wasmedge-types] Invalid WasmEdge_Mutability: 0x2A")]
    fn mutability_from_u32_panics_on_invalid_value() {
        // Preserves the exact pre-existing panic message (hex, uppercase) of the hand-written
        // `From<u32> for Mutability` impl this macro replaced.
        let _ = Mutability::from(42u32);
    }

    #[test]
    #[should_panic(expected = "[wasmedge-types] Invalid WasmEdge_Mutability: 0x2A")]
    fn mutability_from_i32_panics_on_invalid_value() {
        let _ = Mutability::from(42i32);
    }

    #[test]
    fn mutability_try_from_valid() {
        assert_eq!(Mutability::try_from_u32(0), Ok(Mutability::Const));
        assert_eq!(Mutability::try_from_i32(1), Ok(Mutability::Var));
    }

    #[test]
    fn mutability_try_from_invalid_does_not_panic() {
        let err = Mutability::try_from_u32(42).unwrap_err();
        assert_eq!(err.to_string(), "Unknown Mutability value: 42");

        let err = Mutability::try_from_i32(-1).unwrap_err();
        assert_eq!(err.to_string(), "Unknown Mutability value: -1");
    }

    #[test]
    fn compiler_optimization_level_round_trip() {
        let cases = [
            (0u32, CompilerOptimizationLevel::O0),
            (1, CompilerOptimizationLevel::O1),
            (2, CompilerOptimizationLevel::O2),
            (3, CompilerOptimizationLevel::O3),
            (4, CompilerOptimizationLevel::Os),
            (5, CompilerOptimizationLevel::Oz),
        ];
        for (val, variant) in cases {
            assert_eq!(CompilerOptimizationLevel::from(val), variant);
            assert_eq!(CompilerOptimizationLevel::from(val as i32), variant);
            assert_eq!(u32::from(variant), val);
            assert_eq!(i32::from(variant), val as i32);
            assert_eq!(CompilerOptimizationLevel::try_from_u32(val), Ok(variant));
            assert_eq!(
                CompilerOptimizationLevel::try_from_i32(val as i32),
                Ok(variant)
            );
        }
    }

    #[test]
    #[should_panic(expected = "Unknown CompilerOptimizationLevel value: 6")]
    fn compiler_optimization_level_from_u32_panics_on_invalid_value() {
        let _ = CompilerOptimizationLevel::from(6u32);
    }

    #[test]
    fn compiler_optimization_level_try_from_invalid_does_not_panic() {
        assert!(CompilerOptimizationLevel::try_from_u32(6).is_err());
        assert!(CompilerOptimizationLevel::try_from_i32(-1).is_err());
    }

    #[test]
    fn compiler_output_format_round_trip() {
        let cases = [
            (0u32, CompilerOutputFormat::Native),
            (1, CompilerOutputFormat::Wasm),
        ];
        for (val, variant) in cases {
            assert_eq!(CompilerOutputFormat::from(val), variant);
            assert_eq!(CompilerOutputFormat::from(val as i32), variant);
            assert_eq!(u32::from(variant), val);
            assert_eq!(i32::from(variant), val as i32);
            assert_eq!(CompilerOutputFormat::try_from_u32(val), Ok(variant));
            assert_eq!(CompilerOutputFormat::try_from_i32(val as i32), Ok(variant));
        }
    }

    #[test]
    #[should_panic(expected = "Unknown CompilerOutputFormat value: 2")]
    fn compiler_output_format_from_u32_panics_on_invalid_value() {
        let _ = CompilerOutputFormat::from(2u32);
    }

    #[test]
    fn compiler_output_format_try_from_invalid_does_not_panic() {
        assert!(CompilerOutputFormat::try_from_u32(2).is_err());
        assert!(CompilerOutputFormat::try_from_i32(-1).is_err());
    }
}

/// Defines WasmEdge host module registration enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HostRegistration {
    Wasi,
    WasmEdgeProcess,
    WasiNn,
    WasiCryptoCommon,
    WasiCryptoAsymmetricCommon,
    WasiCryptoKx,
    WasiCryptoSignatures,
    WasiCryptoSymmetric,
}
impl From<u32> for HostRegistration {
    fn from(val: u32) -> Self {
        match val {
            0 => HostRegistration::Wasi,
            1 => HostRegistration::WasmEdgeProcess,
            2 => HostRegistration::WasiNn,
            3 => HostRegistration::WasiCryptoCommon,
            4 => HostRegistration::WasiCryptoAsymmetricCommon,
            5 => HostRegistration::WasiCryptoKx,
            6 => HostRegistration::WasiCryptoSignatures,
            7 => HostRegistration::WasiCryptoSymmetric,
            _ => panic!("Unknown WasmEdge_HostRegistration value: {val}"),
        }
    }
}
impl From<HostRegistration> for u32 {
    fn from(val: HostRegistration) -> u32 {
        match val {
            HostRegistration::Wasi => 0,
            HostRegistration::WasmEdgeProcess => 1,
            HostRegistration::WasiNn => 2,
            HostRegistration::WasiCryptoCommon => 3,
            HostRegistration::WasiCryptoAsymmetricCommon => 4,
            HostRegistration::WasiCryptoKx => 5,
            HostRegistration::WasiCryptoSignatures => 6,
            HostRegistration::WasiCryptoSymmetric => 7,
        }
    }
}

/// Defines the type of external WasmEdge instances.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExternalInstanceType {
    /// A WasmEdge instance that is a WasmEdge Func.
    Func(FuncType),
    /// A WasmEdge instance that is a WasmEdge Table.
    Table(TableType),
    /// A WasmEdge instance that is a WasmEdge Memory.
    Memory(MemoryType),
    /// A WasmEdge instance that is a WasmEdge Global.
    Global(GlobalType),
}
impl From<u32> for ExternalInstanceType {
    fn from(value: u32) -> Self {
        match value {
            0 => ExternalInstanceType::Func(FuncType::default()),
            1 => ExternalInstanceType::Table(TableType::default()),
            2 => ExternalInstanceType::Memory(MemoryType::default()),
            3 => ExternalInstanceType::Global(GlobalType::default()),
            _ => panic!("[wasmedge-types] Invalid WasmEdge_ExternalType: {value:#X}",),
        }
    }
}
impl From<i32> for ExternalInstanceType {
    fn from(value: i32) -> Self {
        match value {
            0 => ExternalInstanceType::Func(FuncType::default()),
            1 => ExternalInstanceType::Table(TableType::default()),
            2 => ExternalInstanceType::Memory(MemoryType::default()),
            3 => ExternalInstanceType::Global(GlobalType::default()),
            _ => panic!("[wasmedge-types] Invalid WasmEdge_ExternalType: {value:#X}",),
        }
    }
}
impl std::fmt::Display for ExternalInstanceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            ExternalInstanceType::Func(_) => "function",
            ExternalInstanceType::Table(_) => "table",
            ExternalInstanceType::Memory(_) => "memory",
            ExternalInstanceType::Global(_) => "global",
        };
        write!(f, "{message}")
    }
}

/// Struct of WasmEdge FuncType.
///
/// A [FuncType] is used to declare the types of the parameters and return values of a WasmEdge Func to be created.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FuncType {
    args: Vec<ValType>,
    returns: Vec<ValType>,
}
impl FuncType {
    /// Creates a new [FuncType] with the given types of arguments and returns.
    ///
    /// # Arguments
    ///
    /// * `args` - A vector of [ValType]s that represent the types of the arguments.
    ///
    /// * `returns` - A vector of [ValType]s that represent the types of the returns.
    pub fn new(args: Vec<ValType>, returns: Vec<ValType>) -> Self {
        Self { args, returns }
    }

    /// Returns the types of the arguments of a host function.
    pub fn args(&self) -> &[ValType] {
        &self.args
    }

    /// Returns the number of the arguments of a host function.
    pub fn args_len(&self) -> usize {
        self.args.len()
    }

    /// Returns the types of the returns of a host function.
    pub fn returns(&self) -> &[ValType] {
        &self.returns
    }

    /// Returns the number of the returns of a host function.
    pub fn returns_len(&self) -> usize {
        self.returns.len()
    }
}

/// Struct of WasmEdge TableType.
///
/// A [TableType] is used to declare the element type and the size range of a WasmEdge Table to be created.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableType {
    elem_ty: RefType,
    min: u32,
    max: Option<u32>,
}
impl TableType {
    /// Creates a new [TableType] with the given element type and the size range.
    ///
    /// # Arguments
    ///
    /// * `elem_ty` - The element type of the table to be created.
    ///
    /// * `min` - The minimum size of the table to be created.
    ///
    /// * `max` - The maximum size of the table to be created.
    pub fn new(elem_ty: RefType, min: u32, max: Option<u32>) -> Self {
        Self { elem_ty, min, max }
    }

    /// Returns the element type defined in the [TableType].
    pub fn elem_ty(&self) -> RefType {
        self.elem_ty
    }

    /// Returns the minimum size defined in the [TableType].
    pub fn minimum(&self) -> u32 {
        self.min
    }

    /// Returns the maximum size defined in the [TableType].
    pub fn maximum(&self) -> Option<u32> {
        self.max
    }
}
impl Default for TableType {
    fn default() -> Self {
        Self {
            elem_ty: RefType::FuncRef,
            min: 0,
            max: None,
        }
    }
}

/// Struct of WasmEdge MemoryType.
///
/// A [MemoryType] is used to declare the size range of a WasmEdge Memory to be created.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MemoryType {
    min: u32,
    max: Option<u32>,
    shared: bool,
}
impl MemoryType {
    /// Creates a new [MemoryType] with the given size range.
    ///
    /// # Arguments
    ///
    /// * `min` - The minimum size of the memory to be created.
    ///
    /// * `max` - The maximum size of the memory to be created. If `shared` is set to true, `max` must be set.
    ///
    /// * `shared` - Enables shared memory if true.
    pub fn new(min: u32, max: Option<u32>, shared: bool) -> WasmEdgeResult<Self> {
        if shared && max.is_none() {
            return Err(Box::new(error::WasmEdgeError::Mem(
                error::MemError::CreateSharedType,
            )));
        }
        Ok(Self { min, max, shared })
    }

    /// Returns the minimum size defined in the [MemoryType].
    pub fn minimum(&self) -> u32 {
        self.min
    }

    /// Returns the maximum size defined in the [MemoryType].
    pub fn maximum(&self) -> Option<u32> {
        self.max
    }

    /// Returns whether the memory is shared.
    pub fn shared(&self) -> bool {
        self.shared
    }
}

/// Struct of WasmEdge GlobalType.
///
/// A [GlobalType] is used to declare the type of a WasmEdge Global to be created.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlobalType {
    ty: ValType,
    mutability: Mutability,
}
impl GlobalType {
    /// Creates a new [GlobalType] with the given value type and mutability.
    ///
    /// # Arguments
    ///
    /// * `ty` - The value type of the global to be created.
    ///
    /// * `mutability` - The value mutability property of the global to be created.
    pub fn new(ty: ValType, mutability: Mutability) -> Self {
        Self { ty, mutability }
    }

    /// Returns the value type defined in the [GlobalType].
    pub fn value_ty(&self) -> ValType {
        self.ty
    }

    /// Returns the value mutability property defined in the [GlobalType].
    pub fn mutability(&self) -> Mutability {
        self.mutability
    }
}
impl Default for GlobalType {
    fn default() -> Self {
        Self {
            ty: ValType::I32,
            mutability: Mutability::Var,
        }
    }
}

/// Parses in-memory bytes as either the [WebAssembly Text format](http://webassembly.github.io/spec/core/text/index.html), or a binary WebAssembly module.
pub use wat::parse_bytes as wat2wasm;

/// The WasmEdge result type.
pub type WasmEdgeResult<T> = Result<T, Box<error::WasmEdgeError>>;

/// This is a workaround solution to the [`never`](https://doc.rust-lang.org/std/primitive.never.html) type in Rust. It will be replaced by `!` once it is stable.
///
/// As an uninhabited (empty) enum, `NeverType` is automatically [`Send`] and [`Sync`]: there are
/// no variants that could hold non-`Send`/`Sync` data, so no `unsafe impl` is required.
#[derive(Debug, Clone)]
pub enum NeverType {}

#[cfg(test)]
mod never_type_tests {
    use super::NeverType;

    fn assert_send_sync<T: Send + Sync>() {}

    #[test]
    fn never_type_is_auto_send_sync() {
        // Compile-time proof (this line simply must compile) that removing the redundant
        // `unsafe impl Send/Sync for NeverType` did not change auto-trait behavior: an
        // uninhabited enum is automatically `Send + Sync` without any `unsafe impl`.
        assert_send_sync::<NeverType>();
    }
}
