//! Defines WasmEdge ahead-of-time compiler.

use crate::{config::Config, WasmEdgeResult};
use std::path::{Path, PathBuf};
use wasmedge_sys as sys;

/// Defines WasmEdge ahead-of-time(AOT) compiler and the relevant APIs.
#[derive(Debug)]
pub struct Compiler {
    pub(crate) inner: sys::Compiler,
}
impl Compiler {
    /// Creates a new AOT compiler.
    ///
    /// # Error
    ///
    /// If fail to create a AOT compiler, then an error is returned.
    pub fn new(config: Option<&Config>) -> WasmEdgeResult<Self> {
        let inner = match config {
            Some(cfg) => sys::Compiler::create(Some(&cfg.inner))?,
            None => sys::Compiler::create(None)?,
        };

        Ok(Self { inner })
    }

    /// Compiles the given wasm file into a shared library file (*.so in Linux, *.dylib in macOS, or *.dll in Windows). The file path of the generated shared library file will be returned if the method works successfully.
    ///
    /// # Arguments
    ///
    /// * `wasm_file` - The target wasm file.
    ///
    /// * `filename` - The filename of the generated shared library file.
    ///
    /// * `out_dir` - The target directory to save the generated shared library file.
    ///
    /// # Error
    ///
    /// If fail to compile, then an error is returned.
    pub fn compile_from_file(
        &self,
        wasm_file: impl AsRef<Path>,
        filename: impl AsRef<str>,
        out_dir: impl AsRef<Path>,
    ) -> WasmEdgeResult<PathBuf> {
        #[cfg(target_os = "linux")]
        let extension = "so";
        #[cfg(target_os = "macos")]
        let extension = "dylib";
        #[cfg(target_os = "windows")]
        let extension = "dll";
        let aot_file = out_dir
            .as_ref()
            .join(format!("{}.{}", filename.as_ref(), extension));
        self.inner.compile_from_file(wasm_file, &aot_file)?;

        Ok(aot_file)
    }

    /// Compiles the given wasm bytes into a shared library file (*.so in Linux, *.dylib in macOS, or *.dll in Windows). The file path of the generated shared library file will be returned if the method works successfully.
    ///
    /// # Argument
    ///
    /// * `bytes` - A in-memory WASM bytes.
    ///
    /// * `filename` - The filename of the generated shared library file.
    ///
    /// * `out_dir` - The target directory to save the generated shared library file.
    ///
    /// # Error
    ///
    /// If fail to compile, then an error is returned.
    pub fn compile_from_bytes(
        &self,
        bytes: impl AsRef<[u8]>,
        filename: impl AsRef<str>,
        out_dir: impl AsRef<Path>,
    ) -> WasmEdgeResult<PathBuf> {
        #[cfg(target_os = "linux")]
        let extension = "so";
        #[cfg(target_os = "macos")]
        let extension = "dylib";
        #[cfg(target_os = "windows")]
        let extension = "dll";
        let aot_file = out_dir
            .as_ref()
            .join(format!("{}.{}", filename.as_ref(), extension));
        self.inner.compile_from_bytes(bytes, &aot_file)?;

        Ok(aot_file)
    }
}
