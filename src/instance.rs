//! Defines WasmEdge Instance.
use wasmedge_sys as sys;

/// Represents an instantiated module.
///
/// An [Instance] represents an instantiated module. In the instantiation process, A [module instance](crate::Instance) is created based on a [compiled module](crate::Module). From a [module instance] the exported [host function](sys::Function), [table](sys::Table), [memory](sys::Memory), and [global](sys::Global) instances can be fetched.
pub type Instance = sys::Instance;
