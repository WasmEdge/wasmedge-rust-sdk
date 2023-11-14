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

#[cfg(not(feature = "async"))]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        config::{CompilerConfigOptions, ConfigBuilder},
        params, wat2wasm, CompilerOutputFormat, VmBuilder, WasmVal,
    };
    use std::io::Read;

    #[test]
    fn test_compiler_compile_from_file() -> Result<(), Box<dyn std::error::Error>> {
        // compile from file
        {
            let config = ConfigBuilder::default()
                .with_compiler_config(
                    CompilerConfigOptions::new().out_format(CompilerOutputFormat::Native),
                )
                .build()?;

            let compiler = Compiler::new(Some(&config))?;
            let wasm_file = std::env::current_dir()
                .unwrap()
                .join("examples/wasmedge-sys/data/fibonacci.wat");
            let out_dir = std::env::current_dir()?;
            let aot_filename = "aot_fibonacci_1";
            let aot_file_path = compiler.compile_from_file(wasm_file, aot_filename, out_dir)?;
            assert!(aot_file_path.exists());
            #[cfg(target_os = "macos")]
            assert!(aot_file_path.ends_with("aot_fibonacci_1.dylib"));
            #[cfg(target_os = "linux")]
            assert!(aot_file_path.ends_with("aot_fibonacci_1.so"));
            #[cfg(target_os = "windows")]
            assert!(aot_file_path.ends_with("aot_fibonacci_1.dll"));

            // read buffer
            let mut aot_file = std::fs::File::open(&aot_file_path)?;
            let mut buffer = [0u8; 4];
            aot_file.read_exact(&mut buffer)?;
            let wasm_magic: [u8; 4] = [0x00, 0x61, 0x73, 0x6D];
            assert_ne!(buffer, wasm_magic);

            let res =
                VmBuilder::new()
                    .build()?
                    .run_func_from_file(&aot_file_path, "fib", params!(5))?;
            assert_eq!(res[0].to_i32(), 8);

            // cleanup
            assert!(std::fs::remove_file(aot_file_path).is_ok());
        }

        // compile from bytes
        {
            let wasm_bytes = wat2wasm(
                br#"(module
                    (type (;0;) (func (param i32) (result i32)))
                    (type (;1;) (func))
                    (func (;0;) (type 0) (param i32) (result i32)
                      (local i32 i32 i32)
                      i32.const 1
                      local.set 1
                      block  ;; label = @1
                        local.get 0
                        i32.const 2
                        i32.lt_s
                        br_if 0 (;@1;)
                        local.get 0
                        i32.const -1
                        i32.add
                        local.tee 1
                        i32.const 7
                        i32.and
                        local.set 2
                        block  ;; label = @2
                          block  ;; label = @3
                            local.get 0
                            i32.const -2
                            i32.add
                            i32.const 7
                            i32.ge_u
                            br_if 0 (;@3;)
                            i32.const 1
                            local.set 0
                            i32.const 1
                            local.set 1
                            br 1 (;@2;)
                          end
                          local.get 1
                          i32.const -8
                          i32.and
                          local.set 3
                          i32.const 1
                          local.set 0
                          i32.const 1
                          local.set 1
                          loop  ;; label = @3
                            local.get 1
                            local.get 0
                            i32.add
                            local.tee 0
                            local.get 1
                            i32.add
                            local.tee 1
                            local.get 0
                            i32.add
                            local.tee 0
                            local.get 1
                            i32.add
                            local.tee 1
                            local.get 0
                            i32.add
                            local.tee 0
                            local.get 1
                            i32.add
                            local.tee 1
                            local.get 0
                            i32.add
                            local.tee 0
                            local.get 1
                            i32.add
                            local.set 1
                            local.get 3
                            i32.const -8
                            i32.add
                            local.tee 3
                            br_if 0 (;@3;)
                          end
                        end
                        local.get 2
                        i32.eqz
                        br_if 0 (;@1;)
                        local.get 1
                        local.set 3
                        loop  ;; label = @2
                          local.get 3
                          local.get 0
                          i32.add
                          local.set 1
                          local.get 3
                          local.set 0
                          local.get 1
                          local.set 3
                          local.get 2
                          i32.const -1
                          i32.add
                          local.tee 2
                          br_if 0 (;@2;)
                        end
                      end
                      local.get 1)
                    (func (;1;) (type 1))
                    (func (;2;) (type 1)
                      call 1
                      call 1)
                    (func (;3;) (type 0) (param i32) (result i32)
                      local.get 0
                      call 0
                      call 2)
                    (table (;0;) 1 1 funcref)
                    (memory (;0;) 16)
                    (global (;0;) (mut i32) (i32.const 1048576))
                    (export "memory" (memory 0))
                    (export "fib" (func 3)))
              "#,
            )?;

            // create a aot compiler
            let config = ConfigBuilder::default()
                .with_compiler_config(
                    CompilerConfigOptions::new().out_format(CompilerOutputFormat::Native),
                )
                .build()?;
            let compiler = Compiler::new(Some(&config))?;

            // compile wasm bytes into a shared library file
            let out_dir = std::env::current_dir()?;
            let aot_filename = "aot_fibonacci_2";
            let aot_file_path = compiler.compile_from_bytes(wasm_bytes, aot_filename, out_dir)?;
            assert!(aot_file_path.exists());
            #[cfg(target_os = "macos")]
            assert!(aot_file_path.ends_with("aot_fibonacci_2.dylib"));
            #[cfg(target_os = "linux")]
            assert!(aot_file_path.ends_with("aot_fibonacci_2.so"));
            #[cfg(target_os = "windows")]
            assert!(aot_file_path.ends_with("aot_fibonacci_2.dll"));

            // read buffer
            let mut aot_file = std::fs::File::open(&aot_file_path)?;
            let mut buffer = [0u8; 4];
            aot_file.read_exact(&mut buffer)?;
            let wasm_magic: [u8; 4] = [0x00, 0x61, 0x73, 0x6D];
            assert_ne!(buffer, wasm_magic);

            let res =
                VmBuilder::new()
                    .build()?
                    .run_func_from_file(&aot_file_path, "fib", params!(5))?;
            assert_eq!(res[0].to_i32(), 8);

            // cleanup
            assert!(std::fs::remove_file(aot_file_path).is_ok());
        }

        Ok(())
    }
}
