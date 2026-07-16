//! Defines the versioning and logging functions.

use crate::{
    WasmEdgeResult,
    ffi::{self, WasmEdge_Result, WasmEdge_ResultGetCode, WasmEdge_ResultOK},
};
use std::{
    ffi::{CStr, CString},
    path::Path,
};
use wasmedge_types::error::{
    CoreCommonError, CoreComponentError, CoreError, CoreExecutionError, CoreInstantiationError,
    CoreLoadError, CoreValidationError, WasmEdgeError,
};

#[cfg(unix)]
pub(crate) fn path_to_cstring(path: &Path) -> WasmEdgeResult<CString> {
    use std::os::unix::ffi::OsStrExt;
    CString::new(path.as_os_str().as_bytes())
        .map_err(|err| Box::new(WasmEdgeError::FoundNulByte(err)))
}

#[cfg(windows)]
pub(crate) fn path_to_cstring(path: &Path) -> WasmEdgeResult<CString> {
    match path.to_str() {
        Some(s) => CString::new(s).map_err(|err| Box::new(WasmEdgeError::FoundNulByte(err))),
        None => Err(Box::new(WasmEdgeError::WindowsPathConversion(
            path.to_string_lossy().to_string(),
        ))),
    }
}

/// Logs the debug information.
pub fn log_debug_info() {
    unsafe { ffi::WasmEdge_LogSetDebugLevel() }
}

/// Logs the error information.
pub fn log_error_info() {
    unsafe { ffi::WasmEdge_LogSetErrorLevel() }
}

/// Sets the logging system off.
pub fn log_off() {
    unsafe { ffi::WasmEdge_LogOff() }
}

// Checks the result of a `FFI` function.
// bindgen maps C enums to i32 on MSVC but u32 on unix: these casts are load-bearing on Windows
#[allow(trivial_numeric_casts)]
pub(crate) fn check(result: WasmEdge_Result) -> WasmEdgeResult<()> {
    let category = unsafe { ffi::WasmEdge_ResultGetCategory(result) };
    let code = unsafe {
        if !WasmEdge_ResultOK(result) {
            WasmEdge_ResultGetCode(result)
        } else {
            0u32
        }
    } as ffi::WasmEdge_ErrCode;

    match category {
        ffi::WasmEdge_ErrCategory_UserLevelError => Err(Box::new(WasmEdgeError::User(code as _))),
        ffi::WasmEdge_ErrCategory_WASM => gen_runtime_error(code),
        _ => panic!("Invalid category value: {category}"),
    }
}

// The single source of truth mapping [`CoreError`] variants to WasmEdge C-API
// error codes. `core_error_codes!` expands the table below into BOTH directions
// -- `gen_runtime_error` (code -> variant) and `impl From<CoreError> for
// WasmEdge_Result` (variant -> code) -- so the two can never drift. The
// `error_code_table_round_trips` test walks every listed pair as a guard.
//
// bindgen maps C enums to i32 on MSVC but u32 on unix, so the numeric casts the
// macro emits are load-bearing on Windows; the generated items therefore carry
// `#[allow(trivial_numeric_casts)]`.
macro_rules! core_error_codes {
    (
        $(
            $group:ident($inner:ident) {
                $($case:ident => $code:ident),* $(,)?
            }
        ),* $(,)?
    ) => {
        #[allow(trivial_numeric_casts)]
        fn gen_runtime_error(code: ffi::WasmEdge_ErrCode) -> WasmEdgeResult<()> {
            match code {
                // Success or terminated (exit and return success)
                ffi::WasmEdge_ErrCode_Success => Ok(()),
                $($(
                    ffi::$code => Err(Box::new(WasmEdgeError::Core(
                        CoreError::$group($inner::$case),
                    ))),
                )*)*
                c => Err(Box::new(WasmEdgeError::Core(CoreError::UnknownError(
                    c as _,
                )))),
            }
        }

        impl From<CoreError> for WasmEdge_Result {
            #[allow(trivial_numeric_casts)]
            fn from(val: CoreError) -> WasmEdge_Result {
                let code = match val {
                    $($(
                        CoreError::$group($inner::$case) => ffi::$code,
                    )*)*
                    CoreError::UnknownError(c) => c as ffi::WasmEdge_ErrCode,
                };
                unsafe { ffi::WasmEdge_ResultGen(ffi::WasmEdge_ErrCategory_WASM, code as _) }
            }
        }

        #[cfg(test)]
        const CORE_ERROR_CODE_TABLE: &[(CoreError, ffi::WasmEdge_ErrCode)] = &[
            $($(
                (CoreError::$group($inner::$case), ffi::$code),
            )*)*
        ];
    };
}

core_error_codes! {
    Common(CoreCommonError) {
        Terminated => WasmEdge_ErrCode_Terminated,
        RuntimeError => WasmEdge_ErrCode_RuntimeError,
        CostLimitExceeded => WasmEdge_ErrCode_CostLimitExceeded,
        WrongVMWorkflow => WasmEdge_ErrCode_WrongVMWorkflow,
        FuncNotFound => WasmEdge_ErrCode_FuncNotFound,
        AOTDisabled => WasmEdge_ErrCode_AOTDisabled,
        Interrupted => WasmEdge_ErrCode_Interrupted,
        NotValidated => WasmEdge_ErrCode_NotValidated,
        NonNullRequired => WasmEdge_ErrCode_NonNullRequired,
        SetValueToConst => WasmEdge_ErrCode_SetValueToConst,
        SetValueErrorType => WasmEdge_ErrCode_SetValueErrorType,
        UserDefError => WasmEdge_ErrCode_UserDefError,
    },
    Load(CoreLoadError) {
        IllegalPath => WasmEdge_ErrCode_IllegalPath,
        ReadError => WasmEdge_ErrCode_ReadError,
        UnexpectedEnd => WasmEdge_ErrCode_UnexpectedEnd,
        MalformedMagic => WasmEdge_ErrCode_MalformedMagic,
        MalformedVersion => WasmEdge_ErrCode_MalformedVersion,
        MalformedSection => WasmEdge_ErrCode_MalformedSection,
        SectionSizeMismatch => WasmEdge_ErrCode_SectionSizeMismatch,
        LengthOutOfBounds => WasmEdge_ErrCode_LengthOutOfBounds,
        JunkSection => WasmEdge_ErrCode_JunkSection,
        IncompatibleFuncCode => WasmEdge_ErrCode_IncompatibleFuncCode,
        IncompatibleDataCount => WasmEdge_ErrCode_IncompatibleDataCount,
        DataCountRequired => WasmEdge_ErrCode_DataCountRequired,
        MalformedImportKind => WasmEdge_ErrCode_MalformedImportKind,
        MalformedExportKind => WasmEdge_ErrCode_MalformedExportKind,
        ExpectedZeroByte => WasmEdge_ErrCode_ExpectedZeroByte,
        InvalidMut => WasmEdge_ErrCode_InvalidMut,
        TooManyLocals => WasmEdge_ErrCode_TooManyLocals,
        MalformedValType => WasmEdge_ErrCode_MalformedValType,
        MalformedElemType => WasmEdge_ErrCode_MalformedElemType,
        MalformedRefType => WasmEdge_ErrCode_MalformedRefType,
        MalformedUTF8 => WasmEdge_ErrCode_MalformedUTF8,
        IntegerTooLarge => WasmEdge_ErrCode_IntegerTooLarge,
        IntegerTooLong => WasmEdge_ErrCode_IntegerTooLong,
        IllegalOpCode => WasmEdge_ErrCode_IllegalOpCode,
        IllegalGrammar => WasmEdge_ErrCode_IllegalGrammar,
        SharedMemoryNoMax => WasmEdge_ErrCode_SharedMemoryNoMax,
        IntrinsicsTableNotFound => WasmEdge_ErrCode_IntrinsicsTableNotFound,
        MalformedTable => WasmEdge_ErrCode_MalformedTable,
    },
    Validation(CoreValidationError) {
        InvalidAlignment => WasmEdge_ErrCode_InvalidAlignment,
        TypeCheckFailed => WasmEdge_ErrCode_TypeCheckFailed,
        InvalidLabelIdx => WasmEdge_ErrCode_InvalidLabelIdx,
        InvalidLocalIdx => WasmEdge_ErrCode_InvalidLocalIdx,
        InvalidFieldIdx => WasmEdge_ErrCode_InvalidFieldIdx,
        InvalidFuncTypeIdx => WasmEdge_ErrCode_InvalidFuncTypeIdx,
        InvalidFuncIdx => WasmEdge_ErrCode_InvalidFuncIdx,
        InvalidTableIdx => WasmEdge_ErrCode_InvalidTableIdx,
        InvalidMemoryIdx => WasmEdge_ErrCode_InvalidMemoryIdx,
        InvalidGlobalIdx => WasmEdge_ErrCode_InvalidGlobalIdx,
        InvalidElemIdx => WasmEdge_ErrCode_InvalidElemIdx,
        InvalidDataIdx => WasmEdge_ErrCode_InvalidDataIdx,
        InvalidRefIdx => WasmEdge_ErrCode_InvalidRefIdx,
        ConstExprRequired => WasmEdge_ErrCode_ConstExprRequired,
        DupExportName => WasmEdge_ErrCode_DupExportName,
        ImmutableGlobal => WasmEdge_ErrCode_ImmutableGlobal,
        ImmutableField => WasmEdge_ErrCode_ImmutableField,
        ImmutableArray => WasmEdge_ErrCode_ImmutableArray,
        InvalidResultArity => WasmEdge_ErrCode_InvalidResultArity,
        MultiTables => WasmEdge_ErrCode_MultiTables,
        MultiMemories => WasmEdge_ErrCode_MultiMemories,
        InvalidLimit => WasmEdge_ErrCode_InvalidLimit,
        InvalidMemPages => WasmEdge_ErrCode_InvalidMemPages,
        InvalidStartFunc => WasmEdge_ErrCode_InvalidStartFunc,
        InvalidLaneIdx => WasmEdge_ErrCode_InvalidLaneIdx,
        InvalidUninitLocal => WasmEdge_ErrCode_InvalidUninitLocal,
        InvalidNotDefaultableField => WasmEdge_ErrCode_InvalidNotDefaultableField,
        InvalidNotDefaultableArray => WasmEdge_ErrCode_InvalidNotDefaultableArray,
        InvalidPackedField => WasmEdge_ErrCode_InvalidPackedField,
        InvalidPackedArray => WasmEdge_ErrCode_InvalidPackedArray,
        InvalidUnpackedField => WasmEdge_ErrCode_InvalidUnpackedField,
        InvalidUnpackedArray => WasmEdge_ErrCode_InvalidUnpackedArray,
        InvalidBrRefType => WasmEdge_ErrCode_InvalidBrRefType,
        ArrayTypesMismatch => WasmEdge_ErrCode_ArrayTypesMismatch,
        ArrayTypesNumtypeRequired => WasmEdge_ErrCode_ArrayTypesNumtypeRequired,
        InvalidSubType => WasmEdge_ErrCode_InvalidSubType,
    },
    Instantiation(CoreInstantiationError) {
        ModuleNameConflict => WasmEdge_ErrCode_ModuleNameConflict,
        IncompatibleImportType => WasmEdge_ErrCode_IncompatibleImportType,
        UnknownImport => WasmEdge_ErrCode_UnknownImport,
        DataSegDoesNotFit => WasmEdge_ErrCode_DataSegDoesNotFit,
        ElemSegDoesNotFit => WasmEdge_ErrCode_ElemSegDoesNotFit,
    },
    Execution(CoreExecutionError) {
        WrongInstanceAddress => WasmEdge_ErrCode_WrongInstanceAddress,
        WrongInstanceIndex => WasmEdge_ErrCode_WrongInstanceIndex,
        InstrTypeMismatch => WasmEdge_ErrCode_InstrTypeMismatch,
        FuncSigMismatch => WasmEdge_ErrCode_FuncSigMismatch,
        DivideByZero => WasmEdge_ErrCode_DivideByZero,
        IntegerOverflow => WasmEdge_ErrCode_IntegerOverflow,
        InvalidConvToInt => WasmEdge_ErrCode_InvalidConvToInt,
        TableOutOfBounds => WasmEdge_ErrCode_TableOutOfBounds,
        MemoryOutOfBounds => WasmEdge_ErrCode_MemoryOutOfBounds,
        ArrayOutOfBounds => WasmEdge_ErrCode_ArrayOutOfBounds,
        Unreachable => WasmEdge_ErrCode_Unreachable,
        UninitializedElement => WasmEdge_ErrCode_UninitializedElement,
        UndefinedElement => WasmEdge_ErrCode_UndefinedElement,
        IndirectCallTypeMismatch => WasmEdge_ErrCode_IndirectCallTypeMismatch,
        HostFuncFailed => WasmEdge_ErrCode_HostFuncError,
        RefTypeMismatch => WasmEdge_ErrCode_RefTypeMismatch,
        UnalignedAtomicAccess => WasmEdge_ErrCode_UnalignedAtomicAccess,
        ExpectSharedMemory => WasmEdge_ErrCode_ExpectSharedMemory,
        CastNullToNonNull => WasmEdge_ErrCode_CastNullToNonNull,
        AccessNullFunc => WasmEdge_ErrCode_AccessNullFunc,
        AccessNullStruct => WasmEdge_ErrCode_AccessNullStruct,
        AccessNullArray => WasmEdge_ErrCode_AccessNullArray,
        AccessNullI31 => WasmEdge_ErrCode_AccessNullI31,
        CastFailed => WasmEdge_ErrCode_CastFailed,
    },
    Component(CoreComponentError) {
        MalformedSort => WasmEdge_ErrCode_MalformedSort,
        MalformedAliasTarget => WasmEdge_ErrCode_MalformedAliasTarget,
        MalformedCoreInstance => WasmEdge_ErrCode_MalformedCoreInstance,
        MalformedInstance => WasmEdge_ErrCode_MalformedInstance,
        MalformedDefType => WasmEdge_ErrCode_MalformedDefType,
        MalformedRecordType => WasmEdge_ErrCode_MalformedRecordType,
        MalformedVariantType => WasmEdge_ErrCode_MalformedVariantType,
        MalformedTupleType => WasmEdge_ErrCode_MalformedTupleType,
        MalformedFlagsType => WasmEdge_ErrCode_MalformedFlagsType,
        MalformedCanonical => WasmEdge_ErrCode_MalformedCanonical,
        UnknownCanonicalOption => WasmEdge_ErrCode_UnknownCanonicalOption,
        MalformedName => WasmEdge_ErrCode_MalformedName,
    },
}

/// Returns the major version value.
pub fn version_major_value() -> u32 {
    unsafe { ffi::WasmEdge_VersionGetMajor() }
}

/// Returns the minor version value.
pub fn version_minor_value() -> u32 {
    unsafe { ffi::WasmEdge_VersionGetMinor() }
}

/// Returns the patch version value.
pub fn version_patch_value() -> u32 {
    unsafe { ffi::WasmEdge_VersionGetPatch() }
}

/// Returns the version string.
pub fn version_string() -> String {
    unsafe {
        CStr::from_ptr(ffi::WasmEdge_VersionGet())
            .to_string_lossy()
            .into_owned()
    }
}

/// Triggers the WasmEdge AOT compiler tool
pub fn driver_aot_compiler<I, V>(args: I) -> i32
where
    I: IntoIterator<Item = V>,
    V: AsRef<str>,
{
    // create a vector of zero terminated strings
    let args = args
        .into_iter()
        .map(|arg| CString::new(arg.as_ref()).unwrap())
        .collect::<Vec<CString>>();

    // convert the strings to raw pointers
    let mut c_args = args
        .iter()
        .map(|arg| arg.as_ptr())
        .collect::<Vec<*const core::ffi::c_char>>();

    unsafe { ffi::WasmEdge_Driver_Compiler(c_args.len() as core::ffi::c_int, c_args.as_mut_ptr()) }
}

/// Triggers the WasmEdge runtime tool
pub fn driver_runtime_tool<I, V>(args: I) -> i32
where
    I: IntoIterator<Item = V>,
    V: AsRef<str>,
{
    // create a vector of zero terminated strings
    let args = args
        .into_iter()
        .map(|arg| CString::new(arg.as_ref()).unwrap())
        .collect::<Vec<CString>>();

    // convert the strings to raw pointers
    let mut c_args = args
        .iter()
        .map(|arg| arg.as_ptr())
        .collect::<Vec<*const core::ffi::c_char>>();

    unsafe { ffi::WasmEdge_Driver_Tool(c_args.len() as core::ffi::c_int, c_args.as_mut_ptr()) }
}

/// Triggers the WasmEdge unified tool
pub fn driver_unified_tool<I, V>(args: I) -> i32
where
    I: IntoIterator<Item = V>,
    V: AsRef<str>,
{
    // create a vector of zero terminated strings
    let args = args
        .into_iter()
        .map(|arg| CString::new(arg.as_ref()).unwrap())
        .collect::<Vec<CString>>();

    // convert the strings to raw pointers
    let mut c_args = args
        .iter()
        .map(|arg| arg.as_ptr())
        .collect::<Vec<*const core::ffi::c_char>>();

    unsafe { ffi::WasmEdge_Driver_UniTool(c_args.len() as core::ffi::c_int, c_args.as_mut_ptr()) }
}

#[cfg(test)]
mod tests {
    // bindgen maps the C `WasmEdge_ErrCode` enum to i32 on MSVC but u32 on unix,
    // so the numeric casts below are load-bearing on Windows even where they look
    // trivial on unix.
    #![allow(trivial_numeric_casts)]

    use super::*;

    // Reads the raw error code stored in a `WasmEdge_Result`. Unlike `check`,
    // this does not fold `Terminated` into success via `WasmEdge_ResultOK`; it
    // returns the exact code so the round-trip can be verified for every variant.
    fn result_code(result: WasmEdge_Result) -> ffi::WasmEdge_ErrCode {
        unsafe { WasmEdge_ResultGetCode(result) as ffi::WasmEdge_ErrCode }
    }

    // Drift guard for `core_error_codes!`: every listed (variant, code) pair must
    // map identically in both directions.
    #[test]
    fn error_code_table_round_trips() {
        for (variant, code) in CORE_ERROR_CODE_TABLE {
            // variant -> code
            let produced = result_code(WasmEdge_Result::from(variant.clone()));
            assert_eq!(
                produced, *code,
                "variant {variant:?} encoded as code {produced}, table says {code}"
            );
            // code -> variant
            match gen_runtime_error(*code) {
                Err(err) => assert_eq!(
                    *err,
                    WasmEdgeError::Core(variant.clone()),
                    "code {code} decoded to {err:?}, expected {variant:?}"
                ),
                Ok(()) => panic!("code {code} unexpectedly decoded to Ok(())"),
            }
        }
    }

    #[test]
    fn success_is_ok_and_unknown_round_trips() {
        // Success is the only WASM-category code that is not an error.
        assert!(gen_runtime_error(ffi::WasmEdge_ErrCode_Success).is_ok());

        // A code outside the table round-trips through `UnknownError`.
        let synthetic: ffi::WasmEdge_ErrCode = 0x00FF_FFFF; // 24-bit max, unused
        assert!(
            !CORE_ERROR_CODE_TABLE.iter().any(|(_, c)| *c == synthetic),
            "synthetic code collides with a real mapping"
        );
        match gen_runtime_error(synthetic) {
            Err(err) => assert_eq!(
                *err,
                WasmEdgeError::Core(CoreError::UnknownError(synthetic as _))
            ),
            Ok(()) => panic!("unknown code decoded to Ok(())"),
        }
        assert_eq!(
            result_code(WasmEdge_Result::from(CoreError::UnknownError(
                synthetic as _
            ))),
            synthetic
        );
    }
}
