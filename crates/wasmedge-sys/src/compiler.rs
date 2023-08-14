//! Defines WasmEdge ahead-of-time compiler.

use crate::{ffi, utils, utils::check, Config, WasmEdgeResult};
use std::path::Path;
use wasmedge_types::error::WasmEdgeError;

/// Defines WasmEdge ahead-of-time(AOT) compiler and the relevant APIs.
#[derive(Debug)]
pub struct Compiler {
    pub(crate) inner: InnerCompiler,
}
impl Drop for Compiler {
    fn drop(&mut self) {
        if !self.inner.0.is_null() {
            unsafe { ffi::WasmEdge_CompilerDelete(self.inner.0) }
        }
    }
}
impl Compiler {
    /// Creates a new AOT [compiler](crate::Compiler).
    ///
    /// # Error
    ///
    /// If fail to create a AOT [compiler](crate::Compiler), then an error is returned.
    pub fn create(config: Option<&Config>) -> WasmEdgeResult<Self> {
        let ctx = match config {
            Some(config) => unsafe { ffi::WasmEdge_CompilerCreate(config.inner.0) },
            None => unsafe { ffi::WasmEdge_CompilerCreate(std::ptr::null_mut()) },
        };

        match ctx.is_null() {
            true => Err(Box::new(WasmEdgeError::CompilerCreate)),
            false => Ok(Self {
                inner: InnerCompiler(ctx),
            }),
        }
    }

    /// Compiles the input WASM from the given file path for the AOT mode and stores the result to the output file path.
    ///
    /// # Arguments
    ///
    /// * `wasm_file` - The input wasm file, of which the file extension should be one of `wasm` or`wat`.
    ///
    /// * `aot_file` - The generated AOT wasm file, of which the file extension should be `dylib` on macOS, `so` on Linux or `dll` on Windows.
    ///
    /// # Error
    ///
    /// If fail to compile, then an error is returned.
    pub fn compile_from_file(
        &self,
        wasm_file: impl AsRef<Path>,
        aot_file: impl AsRef<Path>,
    ) -> WasmEdgeResult<()> {
        match wasm_file.as_ref().extension() {
            Some(extension) => match extension.to_str() {
                Some("wasm") => self.compile_from_wasm_file(wasm_file, aot_file),
                Some("wat") => {
                    let bytes = wat::parse_file(wasm_file.as_ref())
                        .map_err(|_| WasmEdgeError::Operation("Failed to parse wat file".into()))?;
                    self.compile_from_bytes(bytes, aot_file)
                }
                _ => Err(Box::new(WasmEdgeError::Operation(
                    "The wasm file's extension should be `wasm` or `wat`".into(),
                ))),
            },
            None => Err(Box::new(WasmEdgeError::Operation(
                "The wasm file's extension should be `wasm` or `wat`".into(),
            ))),
        }
    }

    fn compile_from_wasm_file(
        &self,
        wasm_file: impl AsRef<Path>,
        aot_file: impl AsRef<Path>,
    ) -> WasmEdgeResult<()> {
        let in_path = utils::path_to_cstring(wasm_file.as_ref())?;
        let out_path = utils::path_to_cstring(aot_file.as_ref())?;
        unsafe {
            check(ffi::WasmEdge_CompilerCompile(
                self.inner.0,
                in_path.as_ptr(),
                out_path.as_ptr(),
            ))
        }
    }

    /// Compiles the input WASM from the given bytes for the AOT mode and stores the result to the output file path.
    ///
    /// # Argument
    ///
    /// * `wasm_bytes` - The in-memory WASM bytes.
    ///
    /// * `aot_file` - The generated AOT wasm file, of which the file extension should be `dylib` on macOS, `so` on Linux or `dll` on Windows.
    ///
    /// # Error
    ///
    /// If fail to compile, then an error is returned.
    pub fn compile_from_bytes(
        &self,
        wasm_bytes: impl AsRef<[u8]>,
        aot_file: impl AsRef<Path>,
    ) -> WasmEdgeResult<()> {
        let out_path = utils::path_to_cstring(aot_file.as_ref())?;
        unsafe {
            let ptr = libc::malloc(wasm_bytes.as_ref().len());
            let dst = ::core::slice::from_raw_parts_mut(
                ptr.cast::<std::mem::MaybeUninit<u8>>(),
                wasm_bytes.as_ref().len(),
            );
            let src = ::core::slice::from_raw_parts(
                wasm_bytes
                    .as_ref()
                    .as_ptr()
                    .cast::<std::mem::MaybeUninit<u8>>(),
                wasm_bytes.as_ref().len(),
            );
            dst.copy_from_slice(src);

            check(ffi::WasmEdge_CompilerCompileFromBuffer(
                self.inner.0,
                ptr as *const u8,
                wasm_bytes.as_ref().len() as u64,
                out_path.as_ptr(),
            ))?;

            libc::free(ptr);
        }

        Ok(())
    }

    /// Provides a raw pointer to the inner Compiler context.
    #[cfg(feature = "ffi")]
    #[cfg_attr(docsrs, doc(cfg(feature = "ffi")))]
    pub fn as_ptr(&self) -> *const ffi::WasmEdge_CompilerContext {
        self.inner.0 as *const _
    }
}

#[derive(Debug)]
pub(crate) struct InnerCompiler(pub(crate) *mut ffi::WasmEdge_CompilerContext);
unsafe impl Send for InnerCompiler {}
unsafe impl Sync for InnerCompiler {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AsImport, CallingFrame, Compiler, Config, Executor, FuncType, Function, ImportModule,
        Loader, Store, Validator, WasmValue,
    };
    use std::{
        io::Read,
        sync::{Arc, Mutex},
        thread,
    };
    use wasmedge_macro::sys_host_function;
    use wasmedge_types::{
        error::{CoreError, CoreLoadError, HostFuncError},
        wat2wasm, CompilerOptimizationLevel, CompilerOutputFormat, NeverType,
    };

    #[test]
    #[allow(clippy::assertions_on_result_states)]
    fn test_compiler() {
        {
            let result = Config::create();
            assert!(result.is_ok());
            let config = result.unwrap();

            // create a AOT Compiler without configuration
            let result = Compiler::create(None);
            assert!(result.is_ok());

            // create a AOT Compiler with a given configuration
            let result = Compiler::create(Some(&config));
            assert!(result.is_ok());
            let compiler = result.unwrap();

            // compile a file for universal WASM output format
            let in_path = std::env::current_dir()
                .unwrap()
                .ancestors()
                .nth(2)
                .unwrap()
                .join("examples/wasmedge-sys/data/test.wat");
            #[cfg(target_os = "linux")]
            let out_path = std::path::PathBuf::from("test_aot.so");
            #[cfg(target_os = "macos")]
            let out_path = std::path::PathBuf::from("test_aot.dylib");
            #[cfg(target_os = "windows")]
            let out_path = std::path::PathBuf::from("test_aot.dll");
            assert!(!out_path.exists());
            let result = compiler.compile_from_file(in_path, &out_path);
            assert!(result.is_ok());
            assert!(out_path.exists());
            assert!(std::fs::remove_file(out_path).is_ok());

            // compile a virtual file
            let result = compiler.compile_from_file("not_exist.wasm", "not_exist_ast.wasm");
            assert!(result.is_err());
            assert_eq!(
                result.unwrap_err(),
                Box::new(WasmEdgeError::Core(CoreError::Load(
                    CoreLoadError::IllegalPath
                )))
            );
        }

        {
            let result = Config::create();
            assert!(result.is_ok());
            let mut config = result.unwrap();
            // compile file for shared library output format
            config.set_aot_compiler_output_format(CompilerOutputFormat::Native);

            let result = Compiler::create(Some(&config));
            assert!(result.is_ok());
            let compiler = result.unwrap();
            let in_path = std::env::current_dir()
                .unwrap()
                .ancestors()
                .nth(2)
                .unwrap()
                .join("examples/wasmedge-sys/data/test.wat");
            #[cfg(target_os = "linux")]
            let out_path = std::path::PathBuf::from("test_aot_from_file.so");
            #[cfg(target_os = "macos")]
            let out_path = std::path::PathBuf::from("test_aot_from_file.dylib");
            #[cfg(target_os = "windows")]
            let out_path = std::path::PathBuf::from("test_aot_from_file.dll");
            assert!(!out_path.exists());
            let result = compiler.compile_from_file(in_path, &out_path);
            assert!(result.is_ok());
            assert!(out_path.exists());

            // read buffer
            let result = std::fs::File::open(&out_path);
            assert!(result.is_ok());
            let mut f = result.unwrap();
            let mut buffer = [0u8; 4];
            let result = f.read(&mut buffer);
            assert!(result.is_ok());
            let wasm_magic: [u8; 4] = [0x00, 0x61, 0x73, 0x6D];
            assert_ne!(buffer, wasm_magic);

            // cleanup
            assert!(std::fs::remove_file(out_path).is_ok());
        }

        {
            let result = wat2wasm(
                br#"(module
                    (export "fib" (func $fib))
                    (func $fib (param $n i32) (result i32)
                     (if
                      (i32.lt_s
                       (get_local $n)
                       (i32.const 2)
                      )
                      (return
                       (i32.const 1)
                      )
                     )
                     (return
                      (i32.add
                       (call $fib
                        (i32.sub
                         (get_local $n)
                         (i32.const 2)
                        )
                       )
                       (call $fib
                        (i32.sub
                         (get_local $n)
                         (i32.const 1)
                        )
                       )
                      )
                     )
                    )
                   )
              "#,
            );
            assert!(result.is_ok());
            let wasm_bytes = result.unwrap();

            let result = Config::create();
            assert!(result.is_ok());
            let mut config = result.unwrap();
            config.set_aot_optimization_level(CompilerOptimizationLevel::O0);
            config.set_aot_compiler_output_format(CompilerOutputFormat::Native);

            let result = Compiler::create(Some(&config));
            assert!(result.is_ok());
            let compiler = result.unwrap();
            #[cfg(target_os = "linux")]
            let out_path = std::path::PathBuf::from("test_aot_from_bytes.so");
            #[cfg(target_os = "macos")]
            let out_path = std::path::PathBuf::from("test_aot_from_bytes.dylib");
            #[cfg(target_os = "windows")]
            let out_path = std::path::PathBuf::from("test_aot_from_bytes.dll");
            assert!(!out_path.exists());
            let result = compiler.compile_from_bytes(wasm_bytes, &out_path);
            assert!(result.is_ok());
            assert!(out_path.exists());

            // cleanup
            assert!(std::fs::remove_file(out_path).is_ok());
        }
    }

    #[test]
    #[ignore]
    #[allow(clippy::assertions_on_result_states)]
    fn test_compiler_send() {
        let result = Config::create();
        assert!(result.is_ok());
        let config = result.unwrap();

        // create a AOT Compiler without configuration
        let result = Compiler::create(None);
        assert!(result.is_ok());

        // create a AOT Compiler with a given configuration
        let result = Compiler::create(Some(&config));
        assert!(result.is_ok());
        let compiler = result.unwrap();

        let handle = thread::spawn(move || {
            // compile a file for universal WASM output format
            let in_path = std::env::current_dir()
                .unwrap()
                .join("examples/data/fibonacci.wat");
            #[cfg(target_os = "linux")]
            let out_path = std::path::PathBuf::from("test_aot_fib_send.so");
            #[cfg(target_os = "macos")]
            let out_path = std::path::PathBuf::from("test_aot_fib_send.dylib");
            #[cfg(target_os = "windows")]
            let out_path = std::path::PathBuf::from("test_aot_fib_send.dll");
            assert!(!out_path.exists());
            let result = compiler.compile_from_file(in_path, &out_path);
            assert!(result.is_ok());
            assert!(out_path.exists());
            assert!(std::fs::remove_file(out_path).is_ok());
        });

        handle.join().unwrap();
    }

    #[test]
    #[ignore]
    #[allow(clippy::assertions_on_result_states)]
    fn test_compiler_sync() {
        let result = Config::create();
        assert!(result.is_ok());
        let config = result.unwrap();

        // create a AOT Compiler without configuration
        let result = Compiler::create(None);
        assert!(result.is_ok());

        // create a AOT Compiler with a given configuration
        let result = Compiler::create(Some(&config));
        assert!(result.is_ok());
        let compiler = Arc::new(Mutex::new(result.unwrap()));

        let compiler_cloned = Arc::clone(&compiler);
        let handle = thread::spawn(move || {
            let result = compiler_cloned.lock();
            assert!(result.is_ok());
            let compiler = result.unwrap();

            // compile a file for universal WASM output format
            let in_path = std::env::current_dir()
                .unwrap()
                .join("examples/data/fibonacci.wat");
            let out_path = std::path::PathBuf::from("fibonacci_sync_thread_aot.wasm");
            assert!(!out_path.exists());
            let result = compiler.compile_from_file(in_path, &out_path);
            assert!(result.is_ok());
            assert!(out_path.exists());
            assert!(std::fs::remove_file(out_path).is_ok());
        });

        {
            let result = compiler.lock();
            assert!(result.is_ok());
            let compiler_main = result.unwrap();
            // compile a file for universal WASM output format
            let in_path = std::env::current_dir()
                .unwrap()
                .join("examples/data/fibonacci.wat");
            #[cfg(target_os = "linux")]
            let out_path = std::path::PathBuf::from("test_aot_fib_sync.so");
            #[cfg(target_os = "macos")]
            let out_path = std::path::PathBuf::from("test_aot_fib_sync.dylib");
            #[cfg(target_os = "windows")]
            let out_path = std::path::PathBuf::from("test_aot_fib_sync.dll");
            assert!(!out_path.exists());
            let result = compiler_main.compile_from_file(in_path, &out_path);
            assert!(result.is_ok());
            assert!(out_path.exists());
            assert!(std::fs::remove_file(out_path).is_ok());
        }

        handle.join().unwrap();
    }

    #[test]
    fn test_aot() -> Result<(), Box<dyn std::error::Error>> {
        // create a Config context
        let mut config = Config::create()?;
        // enable options
        config.tail_call(true);
        config.annotations(true);
        config.memory64(true);
        config.threads(true);
        config.exception_handling(true);
        config.function_references(true);
        config.set_aot_optimization_level(CompilerOptimizationLevel::O0);
        config.set_aot_compiler_output_format(CompilerOutputFormat::Native);
        config.interruptible(true);

        // create an executor
        let mut executor = Executor::create(Some(&config), None)?;

        // create a store
        let mut store = Store::create()?;

        let import = create_spec_test_module();
        executor.register_import_module(&mut store, &import)?;

        // set the AOT compiler options
        let result = Compiler::create(Some(&config));
        assert!(result.is_ok());
        let compiler = result.unwrap();

        // compile a file for universal WASM output format
        let in_path = std::env::current_dir()
            .unwrap()
            .ancestors()
            .nth(2)
            .unwrap()
            .join("examples/wasmedge-sys/data/fibonacci.wat");
        #[cfg(target_os = "macos")]
        let out_path = std::path::PathBuf::from("fibonacci_aot.dylib");
        #[cfg(target_os = "linux")]
        let out_path = std::path::PathBuf::from("fibonacci_aot.so");
        #[cfg(target_os = "windows")]
        let out_path = std::path::PathBuf::from("fibonacci_aot.dll");
        assert!(!out_path.exists());
        let result = compiler.compile_from_file(in_path, &out_path);
        assert!(result.is_ok());
        assert!(out_path.exists());

        {
            // register the wasm module as named module
            let extern_module = Loader::create(Some(&config))?.from_file(&out_path)?;
            Validator::create(Some(&config))?.validate(&extern_module)?;
            let extern_instance =
                executor.register_named_module(&mut store, &extern_module, "extern")?;

            let fib = extern_instance.get_func("fib")?;
            let returns = executor.call_func(&fib, [WasmValue::from_i32(5)])?;
            assert_eq!(returns[0].to_i32(), 8);
        }

        {
            // register the wasm module as active module
            let active_module = Loader::create(Some(&config))?.from_file(&out_path)?;
            Validator::create(Some(&config))?.validate(&active_module)?;
            let active_instance = executor.register_active_module(&mut store, &active_module)?;

            let fib = active_instance.get_func("fib")?;
            let returns = executor.call_func(&fib, [WasmValue::from_i32(5)])?;
            assert_eq!(returns[0].to_i32(), 8);
        }

        // remove the wasm file by the compiler
        assert!(std::fs::remove_file(&out_path).is_ok());

        Ok(())
    }

    #[cfg(feature = "aot")]
    fn create_spec_test_module() -> ImportModule<NeverType> {
        // create an ImportObj module
        let result = ImportModule::<NeverType>::create("spectest", None);
        assert!(result.is_ok());
        let mut import = result.unwrap();

        // create a host function
        let result = FuncType::create([], []);
        assert!(result.is_ok());
        let func_ty = result.unwrap();
        let result =
            Function::create_sync_func::<NeverType>(&func_ty, Box::new(spec_test_print), None, 0);
        assert!(result.is_ok());
        let host_func = result.unwrap();
        // add host function "print"
        import.add_func("print", host_func);
        import
    }

    #[cfg(feature = "aot")]
    #[sys_host_function]
    fn spec_test_print(
        _frame: CallingFrame,
        _inputs: Vec<WasmValue>,
    ) -> Result<Vec<WasmValue>, HostFuncError> {
        Ok(vec![])
    }
}
