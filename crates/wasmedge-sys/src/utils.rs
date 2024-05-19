//! Defines the versioning and logging functions.

use crate::{
    ffi::{self, WasmEdge_Result, WasmEdge_ResultGetCode, WasmEdge_ResultOK},
    WasmEdgeResult,
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

fn gen_runtime_error(code: ffi::WasmEdge_ErrCode) -> WasmEdgeResult<()> {
    match code {
        // Success or terminated (exit and return success)
        ffi::WasmEdge_ErrCode_Success => Ok(()),
        ffi::WasmEdge_ErrCode_Terminated => Err(Box::new(WasmEdgeError::Core(CoreError::Common(
            CoreCommonError::Terminated,
        )))),
        // Common errors
        ffi::WasmEdge_ErrCode_RuntimeError => Err(Box::new(WasmEdgeError::Core(
            CoreError::Common(CoreCommonError::RuntimeError),
        ))),
        ffi::WasmEdge_ErrCode_CostLimitExceeded => Err(Box::new(WasmEdgeError::Core(
            CoreError::Common(CoreCommonError::CostLimitExceeded),
        ))),
        ffi::WasmEdge_ErrCode_WrongVMWorkflow => Err(Box::new(WasmEdgeError::Core(
            CoreError::Common(CoreCommonError::WrongVMWorkflow),
        ))),
        ffi::WasmEdge_ErrCode_FuncNotFound => Err(Box::new(WasmEdgeError::Core(
            CoreError::Common(CoreCommonError::FuncNotFound),
        ))),
        ffi::WasmEdge_ErrCode_AOTDisabled => Err(Box::new(WasmEdgeError::Core(CoreError::Common(
            CoreCommonError::AOTDisabled,
        )))),
        ffi::WasmEdge_ErrCode_Interrupted => Err(Box::new(WasmEdgeError::Core(CoreError::Common(
            CoreCommonError::Interrupted,
        )))),
        ffi::WasmEdge_ErrCode_NotValidated => Err(Box::new(WasmEdgeError::Core(
            CoreError::Common(CoreCommonError::NotValidated),
        ))),
        ffi::WasmEdge_ErrCode_NonNullRequired => Err(Box::new(WasmEdgeError::Core(
            CoreError::Common(CoreCommonError::NonNullRequired),
        ))),
        ffi::WasmEdge_ErrCode_SetValueToConst => Err(Box::new(WasmEdgeError::Core(
            CoreError::Common(CoreCommonError::SetValueToConst),
        ))),
        ffi::WasmEdge_ErrCode_SetValueErrorType => Err(Box::new(WasmEdgeError::Core(
            CoreError::Common(CoreCommonError::SetValueErrorType),
        ))),
        ffi::WasmEdge_ErrCode_UserDefError => Err(Box::new(WasmEdgeError::Core(
            CoreError::Common(CoreCommonError::UserDefError),
        ))),

        // Load phase
        ffi::WasmEdge_ErrCode_IllegalPath => Err(Box::new(WasmEdgeError::Core(CoreError::Load(
            CoreLoadError::IllegalPath,
        )))),
        ffi::WasmEdge_ErrCode_ReadError => Err(Box::new(WasmEdgeError::Core(CoreError::Load(
            CoreLoadError::ReadError,
        )))),
        ffi::WasmEdge_ErrCode_UnexpectedEnd => Err(Box::new(WasmEdgeError::Core(CoreError::Load(
            CoreLoadError::UnexpectedEnd,
        )))),
        ffi::WasmEdge_ErrCode_MalformedMagic => Err(Box::new(WasmEdgeError::Core(
            CoreError::Load(CoreLoadError::MalformedMagic),
        ))),
        ffi::WasmEdge_ErrCode_MalformedVersion => Err(Box::new(WasmEdgeError::Core(
            CoreError::Load(CoreLoadError::MalformedVersion),
        ))),
        ffi::WasmEdge_ErrCode_MalformedSection => Err(Box::new(WasmEdgeError::Core(
            CoreError::Load(CoreLoadError::MalformedSection),
        ))),
        ffi::WasmEdge_ErrCode_SectionSizeMismatch => Err(Box::new(WasmEdgeError::Core(
            CoreError::Load(CoreLoadError::SectionSizeMismatch),
        ))),
        ffi::WasmEdge_ErrCode_LengthOutOfBounds => Err(Box::new(WasmEdgeError::Core(
            CoreError::Load(CoreLoadError::LengthOutOfBounds),
        ))),
        ffi::WasmEdge_ErrCode_JunkSection => Err(Box::new(WasmEdgeError::Core(CoreError::Load(
            CoreLoadError::JunkSection,
        )))),
        ffi::WasmEdge_ErrCode_IncompatibleFuncCode => Err(Box::new(WasmEdgeError::Core(
            CoreError::Load(CoreLoadError::IncompatibleFuncCode),
        ))),
        ffi::WasmEdge_ErrCode_IncompatibleDataCount => Err(Box::new(WasmEdgeError::Core(
            CoreError::Load(CoreLoadError::IncompatibleDataCount),
        ))),
        ffi::WasmEdge_ErrCode_DataCountRequired => Err(Box::new(WasmEdgeError::Core(
            CoreError::Load(CoreLoadError::DataCountRequired),
        ))),
        ffi::WasmEdge_ErrCode_MalformedImportKind => Err(Box::new(WasmEdgeError::Core(
            CoreError::Load(CoreLoadError::MalformedImportKind),
        ))),
        ffi::WasmEdge_ErrCode_MalformedExportKind => Err(Box::new(WasmEdgeError::Core(
            CoreError::Load(CoreLoadError::MalformedExportKind),
        ))),
        ffi::WasmEdge_ErrCode_ExpectedZeroByte => Err(Box::new(WasmEdgeError::Core(
            CoreError::Load(CoreLoadError::ExpectedZeroByte),
        ))),
        ffi::WasmEdge_ErrCode_InvalidMut => Err(Box::new(WasmEdgeError::Core(CoreError::Load(
            CoreLoadError::InvalidMut,
        )))),
        ffi::WasmEdge_ErrCode_TooManyLocals => Err(Box::new(WasmEdgeError::Core(CoreError::Load(
            CoreLoadError::TooManyLocals,
        )))),
        ffi::WasmEdge_ErrCode_MalformedValType => Err(Box::new(WasmEdgeError::Core(
            CoreError::Load(CoreLoadError::MalformedValType),
        ))),
        ffi::WasmEdge_ErrCode_MalformedElemType => Err(Box::new(WasmEdgeError::Core(
            CoreError::Load(CoreLoadError::MalformedElemType),
        ))),
        ffi::WasmEdge_ErrCode_MalformedRefType => Err(Box::new(WasmEdgeError::Core(
            CoreError::Load(CoreLoadError::MalformedRefType),
        ))),
        ffi::WasmEdge_ErrCode_MalformedUTF8 => Err(Box::new(WasmEdgeError::Core(CoreError::Load(
            CoreLoadError::MalformedUTF8,
        )))),
        ffi::WasmEdge_ErrCode_IntegerTooLarge => Err(Box::new(WasmEdgeError::Core(
            CoreError::Load(CoreLoadError::IntegerTooLarge),
        ))),
        ffi::WasmEdge_ErrCode_IntegerTooLong => Err(Box::new(WasmEdgeError::Core(
            CoreError::Load(CoreLoadError::IntegerTooLong),
        ))),
        ffi::WasmEdge_ErrCode_IllegalOpCode => Err(Box::new(WasmEdgeError::Core(CoreError::Load(
            CoreLoadError::IllegalOpCode,
        )))),
        ffi::WasmEdge_ErrCode_IllegalGrammar => Err(Box::new(WasmEdgeError::Core(
            CoreError::Load(CoreLoadError::IllegalGrammar),
        ))),
        ffi::WasmEdge_ErrCode_SharedMemoryNoMax => Err(Box::new(WasmEdgeError::Core(
            CoreError::Load(CoreLoadError::SharedMemoryNoMax),
        ))),
        ffi::WasmEdge_ErrCode_IntrinsicsTableNotFound => Err(Box::new(WasmEdgeError::Core(
            CoreError::Load(CoreLoadError::IntrinsicsTableNotFound),
        ))),
        ffi::WasmEdge_ErrCode_MalformedTable => Err(Box::new(WasmEdgeError::Core(
            CoreError::Load(CoreLoadError::MalformedTable),
        ))),

        // Validation phase
        ffi::WasmEdge_ErrCode_InvalidAlignment => Err(Box::new(WasmEdgeError::Core(
            CoreError::Validation(CoreValidationError::InvalidAlignment),
        ))),
        ffi::WasmEdge_ErrCode_TypeCheckFailed => Err(Box::new(WasmEdgeError::Core(
            CoreError::Validation(CoreValidationError::TypeCheckFailed),
        ))),
        ffi::WasmEdge_ErrCode_InvalidLabelIdx => Err(Box::new(WasmEdgeError::Core(
            CoreError::Validation(CoreValidationError::InvalidLabelIdx),
        ))),
        ffi::WasmEdge_ErrCode_InvalidLocalIdx => Err(Box::new(WasmEdgeError::Core(
            CoreError::Validation(CoreValidationError::InvalidLocalIdx),
        ))),
        ffi::WasmEdge_ErrCode_InvalidFieldIdx => Err(Box::new(WasmEdgeError::Core(
            CoreError::Validation(CoreValidationError::InvalidFieldIdx),
        ))),
        ffi::WasmEdge_ErrCode_InvalidFuncTypeIdx => Err(Box::new(WasmEdgeError::Core(
            CoreError::Validation(CoreValidationError::InvalidFuncTypeIdx),
        ))),
        ffi::WasmEdge_ErrCode_InvalidFuncIdx => Err(Box::new(WasmEdgeError::Core(
            CoreError::Validation(CoreValidationError::InvalidFuncIdx),
        ))),
        ffi::WasmEdge_ErrCode_InvalidTableIdx => Err(Box::new(WasmEdgeError::Core(
            CoreError::Validation(CoreValidationError::InvalidTableIdx),
        ))),
        ffi::WasmEdge_ErrCode_InvalidMemoryIdx => Err(Box::new(WasmEdgeError::Core(
            CoreError::Validation(CoreValidationError::InvalidMemoryIdx),
        ))),
        ffi::WasmEdge_ErrCode_InvalidGlobalIdx => Err(Box::new(WasmEdgeError::Core(
            CoreError::Validation(CoreValidationError::InvalidGlobalIdx),
        ))),
        ffi::WasmEdge_ErrCode_InvalidElemIdx => Err(Box::new(WasmEdgeError::Core(
            CoreError::Validation(CoreValidationError::InvalidElemIdx),
        ))),
        ffi::WasmEdge_ErrCode_InvalidDataIdx => Err(Box::new(WasmEdgeError::Core(
            CoreError::Validation(CoreValidationError::InvalidDataIdx),
        ))),
        ffi::WasmEdge_ErrCode_InvalidRefIdx => Err(Box::new(WasmEdgeError::Core(
            CoreError::Validation(CoreValidationError::InvalidRefIdx),
        ))),
        ffi::WasmEdge_ErrCode_ConstExprRequired => Err(Box::new(WasmEdgeError::Core(
            CoreError::Validation(CoreValidationError::ConstExprRequired),
        ))),
        ffi::WasmEdge_ErrCode_DupExportName => Err(Box::new(WasmEdgeError::Core(
            CoreError::Validation(CoreValidationError::DupExportName),
        ))),
        ffi::WasmEdge_ErrCode_ImmutableGlobal => Err(Box::new(WasmEdgeError::Core(
            CoreError::Validation(CoreValidationError::ImmutableGlobal),
        ))),
        ffi::WasmEdge_ErrCode_ImmutableField => Err(Box::new(WasmEdgeError::Core(
            CoreError::Validation(CoreValidationError::ImmutableField),
        ))),
        ffi::WasmEdge_ErrCode_ImmutableArray => Err(Box::new(WasmEdgeError::Core(
            CoreError::Validation(CoreValidationError::ImmutableArray),
        ))),
        ffi::WasmEdge_ErrCode_InvalidResultArity => Err(Box::new(WasmEdgeError::Core(
            CoreError::Validation(CoreValidationError::InvalidResultArity),
        ))),
        ffi::WasmEdge_ErrCode_MultiTables => Err(Box::new(WasmEdgeError::Core(
            CoreError::Validation(CoreValidationError::MultiTables),
        ))),
        ffi::WasmEdge_ErrCode_MultiMemories => Err(Box::new(WasmEdgeError::Core(
            CoreError::Validation(CoreValidationError::MultiMemories),
        ))),
        ffi::WasmEdge_ErrCode_InvalidLimit => Err(Box::new(WasmEdgeError::Core(
            CoreError::Validation(CoreValidationError::InvalidLimit),
        ))),
        ffi::WasmEdge_ErrCode_InvalidMemPages => Err(Box::new(WasmEdgeError::Core(
            CoreError::Validation(CoreValidationError::InvalidMemPages),
        ))),
        ffi::WasmEdge_ErrCode_InvalidStartFunc => Err(Box::new(WasmEdgeError::Core(
            CoreError::Validation(CoreValidationError::InvalidStartFunc),
        ))),
        ffi::WasmEdge_ErrCode_InvalidLaneIdx => Err(Box::new(WasmEdgeError::Core(
            CoreError::Validation(CoreValidationError::InvalidLaneIdx),
        ))),
        ffi::WasmEdge_ErrCode_InvalidUninitLocal => Err(Box::new(WasmEdgeError::Core(
            CoreError::Validation(CoreValidationError::InvalidUninitLocal),
        ))),
        ffi::WasmEdge_ErrCode_InvalidNotDefaultableField => Err(Box::new(WasmEdgeError::Core(
            CoreError::Validation(CoreValidationError::InvalidNotDefaultableField),
        ))),
        ffi::WasmEdge_ErrCode_InvalidNotDefaultableArray => Err(Box::new(WasmEdgeError::Core(
            CoreError::Validation(CoreValidationError::InvalidNotDefaultableArray),
        ))),
        ffi::WasmEdge_ErrCode_InvalidPackedField => Err(Box::new(WasmEdgeError::Core(
            CoreError::Validation(CoreValidationError::InvalidPackedField),
        ))),
        ffi::WasmEdge_ErrCode_InvalidPackedArray => Err(Box::new(WasmEdgeError::Core(
            CoreError::Validation(CoreValidationError::InvalidPackedArray),
        ))),
        ffi::WasmEdge_ErrCode_InvalidUnpackedField => Err(Box::new(WasmEdgeError::Core(
            CoreError::Validation(CoreValidationError::InvalidUnpackedField),
        ))),
        ffi::WasmEdge_ErrCode_InvalidUnpackedArray => Err(Box::new(WasmEdgeError::Core(
            CoreError::Validation(CoreValidationError::InvalidUnpackedArray),
        ))),
        ffi::WasmEdge_ErrCode_InvalidBrRefType => Err(Box::new(WasmEdgeError::Core(
            CoreError::Validation(CoreValidationError::InvalidBrRefType),
        ))),
        ffi::WasmEdge_ErrCode_ArrayTypesMismatch => Err(Box::new(WasmEdgeError::Core(
            CoreError::Validation(CoreValidationError::ArrayTypesMismatch),
        ))),
        ffi::WasmEdge_ErrCode_ArrayTypesNumtypeRequired => Err(Box::new(WasmEdgeError::Core(
            CoreError::Validation(CoreValidationError::ArrayTypesNumtypeRequired),
        ))),
        ffi::WasmEdge_ErrCode_InvalidSubType => Err(Box::new(WasmEdgeError::Core(
            CoreError::Validation(CoreValidationError::InvalidSubType),
        ))),

        // Instantiation phase
        ffi::WasmEdge_ErrCode_ModuleNameConflict => Err(Box::new(WasmEdgeError::Core(
            CoreError::Instantiation(CoreInstantiationError::ModuleNameConflict),
        ))),
        ffi::WasmEdge_ErrCode_IncompatibleImportType => Err(Box::new(WasmEdgeError::Core(
            CoreError::Instantiation(CoreInstantiationError::IncompatibleImportType),
        ))),
        ffi::WasmEdge_ErrCode_UnknownImport => Err(Box::new(WasmEdgeError::Core(
            CoreError::Instantiation(CoreInstantiationError::UnknownImport),
        ))),
        ffi::WasmEdge_ErrCode_DataSegDoesNotFit => Err(Box::new(WasmEdgeError::Core(
            CoreError::Instantiation(CoreInstantiationError::DataSegDoesNotFit),
        ))),
        ffi::WasmEdge_ErrCode_ElemSegDoesNotFit => Err(Box::new(WasmEdgeError::Core(
            CoreError::Instantiation(CoreInstantiationError::ElemSegDoesNotFit),
        ))),

        // Execution phase
        ffi::WasmEdge_ErrCode_WrongInstanceAddress => Err(Box::new(WasmEdgeError::Core(
            CoreError::Execution(CoreExecutionError::WrongInstanceAddress),
        ))),
        ffi::WasmEdge_ErrCode_WrongInstanceIndex => Err(Box::new(WasmEdgeError::Core(
            CoreError::Execution(CoreExecutionError::WrongInstanceIndex),
        ))),
        ffi::WasmEdge_ErrCode_InstrTypeMismatch => Err(Box::new(WasmEdgeError::Core(
            CoreError::Execution(CoreExecutionError::InstrTypeMismatch),
        ))),
        ffi::WasmEdge_ErrCode_FuncSigMismatch => Err(Box::new(WasmEdgeError::Core(
            CoreError::Execution(CoreExecutionError::FuncSigMismatch),
        ))),
        ffi::WasmEdge_ErrCode_DivideByZero => Err(Box::new(WasmEdgeError::Core(
            CoreError::Execution(CoreExecutionError::DivideByZero),
        ))),
        ffi::WasmEdge_ErrCode_IntegerOverflow => Err(Box::new(WasmEdgeError::Core(
            CoreError::Execution(CoreExecutionError::IntegerOverflow),
        ))),
        ffi::WasmEdge_ErrCode_InvalidConvToInt => Err(Box::new(WasmEdgeError::Core(
            CoreError::Execution(CoreExecutionError::InvalidConvToInt),
        ))),
        ffi::WasmEdge_ErrCode_TableOutOfBounds => Err(Box::new(WasmEdgeError::Core(
            CoreError::Execution(CoreExecutionError::TableOutOfBounds),
        ))),
        ffi::WasmEdge_ErrCode_MemoryOutOfBounds => Err(Box::new(WasmEdgeError::Core(
            CoreError::Execution(CoreExecutionError::MemoryOutOfBounds),
        ))),
        ffi::WasmEdge_ErrCode_ArrayOutOfBounds => Err(Box::new(WasmEdgeError::Core(
            CoreError::Execution(CoreExecutionError::ArrayOutOfBounds),
        ))),
        ffi::WasmEdge_ErrCode_Unreachable => Err(Box::new(WasmEdgeError::Core(
            CoreError::Execution(CoreExecutionError::Unreachable),
        ))),
        ffi::WasmEdge_ErrCode_UninitializedElement => Err(Box::new(WasmEdgeError::Core(
            CoreError::Execution(CoreExecutionError::UninitializedElement),
        ))),
        ffi::WasmEdge_ErrCode_UndefinedElement => Err(Box::new(WasmEdgeError::Core(
            CoreError::Execution(CoreExecutionError::UndefinedElement),
        ))),
        ffi::WasmEdge_ErrCode_IndirectCallTypeMismatch => Err(Box::new(WasmEdgeError::Core(
            CoreError::Execution(CoreExecutionError::IndirectCallTypeMismatch),
        ))),
        ffi::WasmEdge_ErrCode_HostFuncError => Err(Box::new(WasmEdgeError::Core(
            CoreError::Execution(CoreExecutionError::HostFuncFailed),
        ))),
        ffi::WasmEdge_ErrCode_RefTypeMismatch => Err(Box::new(WasmEdgeError::Core(
            CoreError::Execution(CoreExecutionError::RefTypeMismatch),
        ))),
        ffi::WasmEdge_ErrCode_UnalignedAtomicAccess => Err(Box::new(WasmEdgeError::Core(
            CoreError::Execution(CoreExecutionError::UnalignedAtomicAccess),
        ))),
        ffi::WasmEdge_ErrCode_ExpectSharedMemory => Err(Box::new(WasmEdgeError::Core(
            CoreError::Execution(CoreExecutionError::ExpectSharedMemory),
        ))),
        ffi::WasmEdge_ErrCode_CastNullToNonNull => Err(Box::new(WasmEdgeError::Core(
            CoreError::Execution(CoreExecutionError::CastNullToNonNull),
        ))),
        ffi::WasmEdge_ErrCode_AccessNullFunc => Err(Box::new(WasmEdgeError::Core(
            CoreError::Execution(CoreExecutionError::AccessNullFunc),
        ))),
        ffi::WasmEdge_ErrCode_AccessNullStruct => Err(Box::new(WasmEdgeError::Core(
            CoreError::Execution(CoreExecutionError::AccessNullStruct),
        ))),
        ffi::WasmEdge_ErrCode_AccessNullArray => Err(Box::new(WasmEdgeError::Core(
            CoreError::Execution(CoreExecutionError::AccessNullArray),
        ))),
        ffi::WasmEdge_ErrCode_AccessNullI31 => Err(Box::new(WasmEdgeError::Core(
            CoreError::Execution(CoreExecutionError::AccessNullI31),
        ))),
        ffi::WasmEdge_ErrCode_CastFailed => Err(Box::new(WasmEdgeError::Core(
            CoreError::Execution(CoreExecutionError::CastFailed),
        ))),

        // Component model phase
        ffi::WasmEdge_ErrCode_MalformedSort => Err(Box::new(WasmEdgeError::Core(
            CoreError::Component(CoreComponentError::MalformedSort),
        ))),
        ffi::WasmEdge_ErrCode_MalformedAliasTarget => Err(Box::new(WasmEdgeError::Core(
            CoreError::Component(CoreComponentError::MalformedAliasTarget),
        ))),
        ffi::WasmEdge_ErrCode_MalformedCoreInstance => Err(Box::new(WasmEdgeError::Core(
            CoreError::Component(CoreComponentError::MalformedCoreInstance),
        ))),
        ffi::WasmEdge_ErrCode_MalformedInstance => Err(Box::new(WasmEdgeError::Core(
            CoreError::Component(CoreComponentError::MalformedInstance),
        ))),
        ffi::WasmEdge_ErrCode_MalformedDefType => Err(Box::new(WasmEdgeError::Core(
            CoreError::Component(CoreComponentError::MalformedDefType),
        ))),
        ffi::WasmEdge_ErrCode_MalformedRecordType => Err(Box::new(WasmEdgeError::Core(
            CoreError::Component(CoreComponentError::MalformedRecordType),
        ))),
        ffi::WasmEdge_ErrCode_MalformedVariantType => Err(Box::new(WasmEdgeError::Core(
            CoreError::Component(CoreComponentError::MalformedVariantType),
        ))),
        ffi::WasmEdge_ErrCode_MalformedTupleType => Err(Box::new(WasmEdgeError::Core(
            CoreError::Component(CoreComponentError::MalformedTupleType),
        ))),
        ffi::WasmEdge_ErrCode_MalformedFlagsType => Err(Box::new(WasmEdgeError::Core(
            CoreError::Component(CoreComponentError::MalformedFlagsType),
        ))),
        ffi::WasmEdge_ErrCode_MalformedCanonical => Err(Box::new(WasmEdgeError::Core(
            CoreError::Component(CoreComponentError::MalformedCanonical),
        ))),
        ffi::WasmEdge_ErrCode_UnknownCanonicalOption => Err(Box::new(WasmEdgeError::Core(
            CoreError::Component(CoreComponentError::UnknownCanonicalOption),
        ))),
        ffi::WasmEdge_ErrCode_MalformedName => Err(Box::new(WasmEdgeError::Core(
            CoreError::Component(CoreComponentError::MalformedName),
        ))),
        c => Err(Box::new(WasmEdgeError::Core(CoreError::UnknownError(
            c as _,
        )))),
    }
}

impl From<CoreError> for WasmEdge_Result {
    fn from(val: CoreError) -> WasmEdge_Result {
        let code = match val {
            // Common errors
            CoreError::Common(e) => match e {
                CoreCommonError::Terminated => ffi::WasmEdge_ErrCode_Terminated,
                CoreCommonError::RuntimeError => ffi::WasmEdge_ErrCode_RuntimeError,
                CoreCommonError::CostLimitExceeded => ffi::WasmEdge_ErrCode_CostLimitExceeded,
                CoreCommonError::WrongVMWorkflow => ffi::WasmEdge_ErrCode_WrongVMWorkflow,
                CoreCommonError::FuncNotFound => ffi::WasmEdge_ErrCode_FuncNotFound,
                CoreCommonError::AOTDisabled => ffi::WasmEdge_ErrCode_AOTDisabled,
                CoreCommonError::Interrupted => ffi::WasmEdge_ErrCode_Interrupted,
                CoreCommonError::UserDefError => ffi::WasmEdge_ErrCode_UserDefError,
                CoreCommonError::NotValidated => ffi::WasmEdge_ErrCode_NotValidated,
                CoreCommonError::NonNullRequired => ffi::WasmEdge_ErrCode_NonNullRequired,
                CoreCommonError::SetValueToConst => ffi::WasmEdge_ErrCode_SetValueToConst,
                CoreCommonError::SetValueErrorType => ffi::WasmEdge_ErrCode_SetValueErrorType,
            },

            // Load phase
            CoreError::Load(e) => match e {
                CoreLoadError::IllegalPath => ffi::WasmEdge_ErrCode_IllegalPath,
                CoreLoadError::ReadError => ffi::WasmEdge_ErrCode_ReadError,
                CoreLoadError::UnexpectedEnd => ffi::WasmEdge_ErrCode_UnexpectedEnd,
                CoreLoadError::MalformedMagic => ffi::WasmEdge_ErrCode_MalformedMagic,
                CoreLoadError::MalformedVersion => ffi::WasmEdge_ErrCode_MalformedVersion,
                CoreLoadError::MalformedSection => ffi::WasmEdge_ErrCode_MalformedSection,
                CoreLoadError::SectionSizeMismatch => ffi::WasmEdge_ErrCode_SectionSizeMismatch,
                CoreLoadError::LengthOutOfBounds => ffi::WasmEdge_ErrCode_LengthOutOfBounds,
                CoreLoadError::JunkSection => ffi::WasmEdge_ErrCode_JunkSection,
                CoreLoadError::IncompatibleFuncCode => ffi::WasmEdge_ErrCode_IncompatibleFuncCode,
                CoreLoadError::IncompatibleDataCount => ffi::WasmEdge_ErrCode_IncompatibleDataCount,
                CoreLoadError::DataCountRequired => ffi::WasmEdge_ErrCode_DataCountRequired,
                CoreLoadError::MalformedImportKind => ffi::WasmEdge_ErrCode_MalformedImportKind,
                CoreLoadError::MalformedExportKind => ffi::WasmEdge_ErrCode_MalformedExportKind,
                CoreLoadError::ExpectedZeroByte => ffi::WasmEdge_ErrCode_ExpectedZeroByte,
                CoreLoadError::InvalidMut => ffi::WasmEdge_ErrCode_InvalidMut,
                CoreLoadError::TooManyLocals => ffi::WasmEdge_ErrCode_TooManyLocals,
                CoreLoadError::MalformedValType => ffi::WasmEdge_ErrCode_MalformedValType,
                CoreLoadError::MalformedElemType => ffi::WasmEdge_ErrCode_MalformedElemType,
                CoreLoadError::MalformedRefType => ffi::WasmEdge_ErrCode_MalformedRefType,
                CoreLoadError::MalformedUTF8 => ffi::WasmEdge_ErrCode_MalformedUTF8,
                CoreLoadError::IntegerTooLarge => ffi::WasmEdge_ErrCode_IntegerTooLarge,
                CoreLoadError::IntegerTooLong => ffi::WasmEdge_ErrCode_IntegerTooLong,
                CoreLoadError::IllegalOpCode => ffi::WasmEdge_ErrCode_IllegalOpCode,
                CoreLoadError::IllegalGrammar => ffi::WasmEdge_ErrCode_IllegalGrammar,
                CoreLoadError::SharedMemoryNoMax => ffi::WasmEdge_ErrCode_SharedMemoryNoMax,
                CoreLoadError::IntrinsicsTableNotFound => {
                    ffi::WasmEdge_ErrCode_IntrinsicsTableNotFound
                }
                CoreLoadError::MalformedTable => ffi::WasmEdge_ErrCode_MalformedTable,
            },

            // Validation phase
            CoreError::Validation(e) => match e {
                CoreValidationError::InvalidAlignment => ffi::WasmEdge_ErrCode_InvalidAlignment,
                CoreValidationError::TypeCheckFailed => ffi::WasmEdge_ErrCode_TypeCheckFailed,
                CoreValidationError::InvalidLabelIdx => ffi::WasmEdge_ErrCode_InvalidLabelIdx,
                CoreValidationError::InvalidLocalIdx => ffi::WasmEdge_ErrCode_InvalidLocalIdx,
                CoreValidationError::InvalidFieldIdx => ffi::WasmEdge_ErrCode_InvalidFieldIdx,
                CoreValidationError::InvalidFuncTypeIdx => ffi::WasmEdge_ErrCode_InvalidFuncTypeIdx,
                CoreValidationError::InvalidFuncIdx => ffi::WasmEdge_ErrCode_InvalidFuncIdx,
                CoreValidationError::InvalidTableIdx => ffi::WasmEdge_ErrCode_InvalidTableIdx,
                CoreValidationError::InvalidMemoryIdx => ffi::WasmEdge_ErrCode_InvalidMemoryIdx,
                CoreValidationError::InvalidGlobalIdx => ffi::WasmEdge_ErrCode_InvalidGlobalIdx,
                CoreValidationError::InvalidElemIdx => ffi::WasmEdge_ErrCode_InvalidElemIdx,
                CoreValidationError::InvalidDataIdx => ffi::WasmEdge_ErrCode_InvalidDataIdx,
                CoreValidationError::InvalidRefIdx => ffi::WasmEdge_ErrCode_InvalidRefIdx,
                CoreValidationError::ConstExprRequired => ffi::WasmEdge_ErrCode_ConstExprRequired,
                CoreValidationError::DupExportName => ffi::WasmEdge_ErrCode_DupExportName,
                CoreValidationError::ImmutableGlobal => ffi::WasmEdge_ErrCode_ImmutableGlobal,
                CoreValidationError::ImmutableField => ffi::WasmEdge_ErrCode_ImmutableField,
                CoreValidationError::ImmutableArray => ffi::WasmEdge_ErrCode_ImmutableArray,
                CoreValidationError::InvalidResultArity => ffi::WasmEdge_ErrCode_InvalidResultArity,
                CoreValidationError::MultiTables => ffi::WasmEdge_ErrCode_MultiTables,
                CoreValidationError::MultiMemories => ffi::WasmEdge_ErrCode_MultiMemories,
                CoreValidationError::InvalidLimit => ffi::WasmEdge_ErrCode_InvalidLimit,
                CoreValidationError::InvalidMemPages => ffi::WasmEdge_ErrCode_InvalidMemPages,
                CoreValidationError::InvalidStartFunc => ffi::WasmEdge_ErrCode_InvalidStartFunc,
                CoreValidationError::InvalidLaneIdx => ffi::WasmEdge_ErrCode_InvalidLaneIdx,
                CoreValidationError::InvalidUninitLocal => ffi::WasmEdge_ErrCode_InvalidUninitLocal,
                CoreValidationError::InvalidNotDefaultableField => {
                    ffi::WasmEdge_ErrCode_InvalidNotDefaultableField
                }
                CoreValidationError::InvalidNotDefaultableArray => {
                    ffi::WasmEdge_ErrCode_InvalidNotDefaultableArray
                }
                CoreValidationError::InvalidPackedField => ffi::WasmEdge_ErrCode_InvalidPackedField,
                CoreValidationError::InvalidPackedArray => ffi::WasmEdge_ErrCode_InvalidPackedArray,
                CoreValidationError::InvalidUnpackedField => {
                    ffi::WasmEdge_ErrCode_InvalidUnpackedField
                }
                CoreValidationError::InvalidUnpackedArray => {
                    ffi::WasmEdge_ErrCode_InvalidUnpackedArray
                }
                CoreValidationError::InvalidBrRefType => ffi::WasmEdge_ErrCode_InvalidBrRefType,
                CoreValidationError::ArrayTypesMismatch => ffi::WasmEdge_ErrCode_ArrayTypesMismatch,
                CoreValidationError::ArrayTypesNumtypeRequired => {
                    ffi::WasmEdge_ErrCode_ArrayTypesNumtypeRequired
                }
                CoreValidationError::InvalidSubType => ffi::WasmEdge_ErrCode_InvalidSubType,
            },

            // Instantiation phase
            CoreError::Instantiation(e) => match e {
                CoreInstantiationError::ModuleNameConflict => {
                    ffi::WasmEdge_ErrCode_ModuleNameConflict
                }
                CoreInstantiationError::IncompatibleImportType => {
                    ffi::WasmEdge_ErrCode_IncompatibleImportType
                }
                CoreInstantiationError::UnknownImport => ffi::WasmEdge_ErrCode_UnknownImport,
                CoreInstantiationError::DataSegDoesNotFit => {
                    ffi::WasmEdge_ErrCode_DataSegDoesNotFit
                }
                CoreInstantiationError::ElemSegDoesNotFit => {
                    ffi::WasmEdge_ErrCode_ElemSegDoesNotFit
                }
            },

            // Execution phase
            CoreError::Execution(e) => match e {
                CoreExecutionError::WrongInstanceAddress => {
                    ffi::WasmEdge_ErrCode_WrongInstanceAddress
                }
                CoreExecutionError::WrongInstanceIndex => ffi::WasmEdge_ErrCode_WrongInstanceIndex,
                CoreExecutionError::InstrTypeMismatch => ffi::WasmEdge_ErrCode_InstrTypeMismatch,
                CoreExecutionError::FuncSigMismatch => ffi::WasmEdge_ErrCode_FuncSigMismatch,
                CoreExecutionError::DivideByZero => ffi::WasmEdge_ErrCode_DivideByZero,
                CoreExecutionError::IntegerOverflow => ffi::WasmEdge_ErrCode_IntegerOverflow,
                CoreExecutionError::InvalidConvToInt => ffi::WasmEdge_ErrCode_InvalidConvToInt,
                CoreExecutionError::TableOutOfBounds => ffi::WasmEdge_ErrCode_TableOutOfBounds,
                CoreExecutionError::MemoryOutOfBounds => ffi::WasmEdge_ErrCode_MemoryOutOfBounds,
                CoreExecutionError::ArrayOutOfBounds => ffi::WasmEdge_ErrCode_ArrayOutOfBounds,
                CoreExecutionError::Unreachable => ffi::WasmEdge_ErrCode_Unreachable,
                CoreExecutionError::UninitializedElement => {
                    ffi::WasmEdge_ErrCode_UninitializedElement
                }
                CoreExecutionError::UndefinedElement => ffi::WasmEdge_ErrCode_UndefinedElement,
                CoreExecutionError::IndirectCallTypeMismatch => {
                    ffi::WasmEdge_ErrCode_IndirectCallTypeMismatch
                }
                CoreExecutionError::HostFuncFailed => ffi::WasmEdge_ErrCode_HostFuncError,
                CoreExecutionError::RefTypeMismatch => ffi::WasmEdge_ErrCode_RefTypeMismatch,
                CoreExecutionError::UnalignedAtomicAccess => {
                    ffi::WasmEdge_ErrCode_UnalignedAtomicAccess
                }
                CoreExecutionError::ExpectSharedMemory => ffi::WasmEdge_ErrCode_ExpectSharedMemory,
                CoreExecutionError::CastNullToNonNull => ffi::WasmEdge_ErrCode_CastNullToNonNull,
                CoreExecutionError::AccessNullFunc => ffi::WasmEdge_ErrCode_AccessNullFunc,
                CoreExecutionError::AccessNullStruct => ffi::WasmEdge_ErrCode_AccessNullStruct,
                CoreExecutionError::AccessNullArray => ffi::WasmEdge_ErrCode_AccessNullArray,
                CoreExecutionError::AccessNullI31 => ffi::WasmEdge_ErrCode_AccessNullI31,
                CoreExecutionError::CastFailed => ffi::WasmEdge_ErrCode_CastFailed,
            },

            CoreError::Component(e) => match e {
                CoreComponentError::MalformedSort => ffi::WasmEdge_ErrCode_MalformedSort,
                CoreComponentError::MalformedAliasTarget => {
                    ffi::WasmEdge_ErrCode_MalformedAliasTarget
                }
                CoreComponentError::MalformedCoreInstance => {
                    ffi::WasmEdge_ErrCode_MalformedCoreInstance
                }
                CoreComponentError::MalformedInstance => ffi::WasmEdge_ErrCode_MalformedInstance,
                CoreComponentError::MalformedDefType => ffi::WasmEdge_ErrCode_MalformedDefType,
                CoreComponentError::MalformedRecordType => {
                    ffi::WasmEdge_ErrCode_MalformedRecordType
                }
                CoreComponentError::MalformedVariantType => {
                    ffi::WasmEdge_ErrCode_MalformedVariantType
                }
                CoreComponentError::MalformedTupleType => ffi::WasmEdge_ErrCode_MalformedTupleType,
                CoreComponentError::MalformedFlagsType => ffi::WasmEdge_ErrCode_MalformedFlagsType,
                CoreComponentError::MalformedCanonical => ffi::WasmEdge_ErrCode_MalformedCanonical,
                CoreComponentError::UnknownCanonicalOption => {
                    ffi::WasmEdge_ErrCode_UnknownCanonicalOption
                }
                CoreComponentError::MalformedName => ffi::WasmEdge_ErrCode_MalformedName,
            },
            CoreError::UnknownError(c) => c as ffi::WasmEdge_ErrCode,
        };
        unsafe { ffi::WasmEdge_ResultGen(ffi::WasmEdge_ErrCategory_WASM, code as _) }
    }
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
        .collect::<Vec<*const std::os::raw::c_char>>();

    unsafe {
        ffi::WasmEdge_Driver_Compiler(c_args.len() as std::os::raw::c_int, c_args.as_mut_ptr())
    }
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
        .collect::<Vec<*const std::os::raw::c_char>>();

    unsafe { ffi::WasmEdge_Driver_Tool(c_args.len() as std::os::raw::c_int, c_args.as_mut_ptr()) }
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
        .collect::<Vec<*const std::os::raw::c_char>>();

    unsafe {
        ffi::WasmEdge_Driver_UniTool(c_args.len() as std::os::raw::c_int, c_args.as_mut_ptr())
    }
}
