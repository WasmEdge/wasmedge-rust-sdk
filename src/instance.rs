//! Defines WasmEdge Instance.
use wasmedge_sys as sys;

/// Represents an instantiated module.
///
/// An [Instance] represents an instantiated module. In the instantiation process, A [module instance](crate::Instance) is created based on a [compiled module](crate::Module). From a [module instance] the exported [host function](crate::Func), [table](crate::Table), [memory](crate::Memory), and [global](crate::Global) instances can be fetched.
pub type Instance = sys::Instance;
