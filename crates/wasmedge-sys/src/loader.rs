//! Defines WasmEdge Loader struct.

use crate::{
    Config, WasmEdgeResult,
    ast_module::{InnerModule, Module},
    ffi, utils,
    utils::check,
};
use std::{path::Path, sync::Arc};
use wasmedge_types::error::WasmEdgeError;

/// [Loader](crate::Loader) is used to load WASM modules from the given WASM files or buffers.
#[derive(Debug)]
pub struct Loader {
    pub(crate) inner: InnerLoader,
}
impl Loader {
    /// Create a new [Loader] to be associated with the given global configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - A global configuration.
    ///
    /// # Error
    ///
    /// If fail to create a [Loader](crate), then an error is returned.
    pub fn create(config: Option<&Config>) -> WasmEdgeResult<Self> {
        let ctx = match config {
            Some(config) => unsafe { ffi::WasmEdge_LoaderCreate(config.inner.0) },
            None => unsafe { ffi::WasmEdge_LoaderCreate(std::ptr::null_mut()) },
        };

        if ctx.is_null() {
            Err(Box::new(WasmEdgeError::LoaderCreate))
        } else {
            Ok(Self {
                inner: InnerLoader(ctx),
            })
        }
    }

    /// Loads a WASM module from a WASM file.
    ///
    /// # Arguments
    ///
    /// * `file` - A wasm file or an AOT wasm file.
    ///
    /// # Error
    ///
    /// If fail to load, then an error is returned.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let file = "path/to/foo.wasm"
    /// let module = loader.from_file(file)?;
    /// ```
    pub fn from_file(&self, file: impl AsRef<Path>) -> WasmEdgeResult<Arc<Module>> {
        match file.as_ref().extension() {
            Some(extension) => match extension.to_str() {
                Some("wasm") => self.load_from_wasm_or_aot_file(&file),
                #[cfg(target_os = "macos")]
                Some("dylib") => self.load_from_wasm_or_aot_file(&file),
                #[cfg(target_os = "linux")]
                Some("so") => self.load_from_wasm_or_aot_file(&file),
                #[cfg(target_os = "windows")]
                Some("dll") => self.load_from_wasm_or_aot_file(&file),
                Some("wat") => {
                    let bytes = wat::parse_file(file.as_ref())
                        .map_err(|_| WasmEdgeError::Operation("Failed to parse wat file".into()))?;
                    self.from_bytes(bytes)
                }
                _ => Err(Box::new(WasmEdgeError::Operation(
                    "The source file's extension should be one of `wasm`, `wat`, `dylib` on macOS, `so` on Linux or `dll` on Windows.".into(),
                ))),
            },
            None => self.load_from_wasm_or_aot_file(&file),
        }
    }

    fn load_from_wasm_or_aot_file(&self, file: impl AsRef<Path>) -> WasmEdgeResult<Arc<Module>> {
        let c_path = utils::path_to_cstring(file.as_ref())?;
        let mut mod_ctx = std::ptr::null_mut();
        unsafe {
            check(ffi::WasmEdge_LoaderParseFromFile(
                self.inner.0,
                &mut mod_ctx,
                c_path.as_ptr(),
            ))?;
        }

        if mod_ctx.is_null() {
            Err(Box::new(WasmEdgeError::ModuleCreate))
        } else {
            Ok(Module {
                inner: InnerModule(mod_ctx),
            }
            .into())
        }
    }

    /// Loads a WASM module from a in-memory bytes.
    ///
    /// # Arguments
    ///
    /// * `bytes` - A in-memory WASM bytes.
    ///
    /// # Error
    ///
    /// If fail to load, then an error is returned.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let bytes = b"\0asm\x01\0\0\0";
    /// let module = loader.from_bytes(&bytes)?;
    /// ```
    ///
    /// Note that the text format is not accepted:
    ///
    /// ```ignore
    /// assert!(loader.from_bytes(b"(module)").is_err());
    /// ```
    pub fn from_bytes(&self, bytes: impl AsRef<[u8]>) -> WasmEdgeResult<Arc<Module>> {
        let bytes = bytes.as_ref();
        let mut mod_ctx: *mut ffi::WasmEdge_ASTModuleContext = std::ptr::null_mut();

        // SAFETY: `WasmEdge_LoaderParseFromBuffer` borrows `bytes` for the call and takes no
        // ownership; an empty slice is a valid non-null, aligned, len-0 pointer.
        unsafe {
            check(ffi::WasmEdge_LoaderParseFromBuffer(
                self.inner.0,
                &mut mod_ctx,
                bytes.as_ptr(),
                bytes.len() as u32,
            ))?;
        }

        if mod_ctx.is_null() {
            Err(Box::new(WasmEdgeError::ModuleCreate))
        } else {
            Ok(Module {
                inner: InnerModule(mod_ctx),
            }
            .into())
        }
    }
}
impl Drop for Loader {
    fn drop(&mut self) {
        unsafe { ffi::WasmEdge_LoaderDelete(self.inner.0) }
    }
}

#[derive(Debug)]
pub(crate) struct InnerLoader(pub(crate) *mut ffi::WasmEdge_LoaderContext);
// SAFETY: opaque owned handle; upstream C API leaves thread affinity undocumented (assumed, pre-existing).
unsafe impl Send for InnerLoader {}
unsafe impl Sync for InnerLoader {}

#[cfg(test)]
mod tests {
    use super::Loader;
    use crate::Config;
    use std::{
        sync::{Arc, Mutex},
        thread,
    };
    use wasmedge_types::error::{CoreError, CoreLoadError, WasmEdgeError};

    // Empty input must not hit a malloc(0)/UB edge; malformed input must return Err without leaking.
    #[test]
    #[allow(clippy::assertions_on_result_states)]
    fn test_from_bytes_empty_and_error_paths_no_ub() {
        let loader = Loader::create(None).unwrap();

        let result = loader.from_bytes([]);
        assert!(result.is_err());

        let result = loader.from_bytes(b"(module)");
        assert!(result.is_err());
    }

    #[test]
    #[allow(clippy::assertions_on_result_states)]
    fn test_loader() {
        // create a Loader instance without configuration
        let result = Loader::create(None);
        assert!(result.is_ok());

        // create a Loader instance with configuration
        let result = Config::create();
        assert!(result.is_ok());
        let mut config = result.unwrap();
        config.reference_types(true);
        let result = Loader::create(Some(&config));
        assert!(result.is_ok());
        let loader = result.unwrap();

        // load from file
        {
            // load .wasm file
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../examples/wasmedge-sys/data/fibonacci.wat");
            let result = loader.from_file(path);
            assert!(result.is_ok());
            let module = result.unwrap();
            assert!(!module.inner.0.is_null());

            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../examples/wasmedge-sys/data/fibonacci.wat");
            let result = loader.from_file(path);
            assert!(result.is_ok());

            let result = loader.from_file("not_exist_file.wasm");
            assert!(result.is_err());
            assert_eq!(
                result.unwrap_err(),
                Box::new(WasmEdgeError::Core(CoreError::Load(
                    CoreLoadError::IllegalPath
                )))
            );
        }

        // load from buffer
        {
            let buffer = b"\0asm\x01\0\0\0";
            let result = loader.from_bytes(buffer);
            assert!(result.is_ok());
            let module = result.unwrap();
            assert!(!module.inner.0.is_null());

            // the text format is not accepted
            let result = loader.from_bytes(b"(module)");
            assert!(result.is_err());
            assert_eq!(
                result.unwrap_err(),
                Box::new(WasmEdgeError::Core(CoreError::Load(
                    CoreLoadError::MalformedMagic
                )))
            );

            // empty is not accepted
            let result = loader.from_bytes([]);
            assert!(result.is_err());
            assert_eq!(
                result.unwrap_err(),
                Box::new(WasmEdgeError::Core(CoreError::Load(
                    CoreLoadError::UnexpectedEnd
                )))
            );
        }
    }

    #[test]
    #[allow(clippy::assertions_on_result_states)]
    fn test_loader_send() {
        // create a Loader instance without configuration
        let result = Loader::create(None);
        assert!(result.is_ok());

        // create a Loader instance with configuration
        let result = Config::create();
        assert!(result.is_ok());
        let mut config = result.unwrap();
        config.reference_types(true);
        let result = Loader::create(Some(&config));
        assert!(result.is_ok());
        let loader = result.unwrap();

        let handle = thread::spawn(move || {
            assert!(!loader.inner.0.is_null());
            println!("{:?}", loader.inner);
        });

        handle.join().unwrap();
    }

    #[test]
    #[allow(clippy::assertions_on_result_states)]
    fn test_loader_sync() {
        // create a Loader instance without configuration
        let result = Loader::create(None);
        assert!(result.is_ok());

        // create a Loader instance with configuration
        let result = Config::create();
        assert!(result.is_ok());
        let mut config = result.unwrap();
        config.reference_types(true);
        let result = Loader::create(Some(&config));
        assert!(result.is_ok());
        let loader = Arc::new(Mutex::new(result.unwrap()));

        let loader_cloned = Arc::clone(&loader);
        let handle = thread::spawn(move || {
            let result = loader_cloned.lock();
            assert!(result.is_ok());
            let loader = result.unwrap();

            assert!(!loader.inner.0.is_null());
        });

        handle.join().unwrap();
    }
}
