//! Defines WasmEdge Driver and CoreVersion types
use wasmedge_sys::utils;

/// Defines WasmEdge Driver functions
#[derive(Debug)]
pub struct Driver {}
impl Driver {
    /// Triggers the WasmEdge AOT compiler tool
    pub fn aot_compiler<I, V>(args: I) -> i32
    where
        I: IntoIterator<Item = V>,
        V: AsRef<str>,
    {
        utils::driver_aot_compiler(args)
    }

    /// Triggers the WasmEdge runtime tool
    pub fn runtime_tool<I, V>(args: I) -> i32
    where
        I: IntoIterator<Item = V>,
        V: AsRef<str>,
    {
        utils::driver_runtime_tool(args)
    }

    /// Triggers the WasmEdge unified tool
    pub fn unified_tool<I, V>(args: I) -> i32
    where
        I: IntoIterator<Item = V>,
        V: AsRef<str>,
    {
        utils::driver_unified_tool(args)
    }
}

/// The version info of WasmEdge core
pub struct CoreVersion {}
impl CoreVersion {
    /// Returns the major version value of WasmEdge core.
    pub fn major() -> u32 {
        wasmedge_sys::utils::version_major_value()
    }

    /// Returns the minor version value of WasmEdge core.
    pub fn minor() -> u32 {
        wasmedge_sys::utils::version_minor_value()
    }

    /// Returns the patch version value of WasmEdge core.
    pub fn patch() -> u32 {
        wasmedge_sys::utils::version_patch_value()
    }

    /// Returns the version string of WasmEdge core.
    pub fn version_string() -> String {
        wasmedge_sys::utils::version_string()
    }
}
