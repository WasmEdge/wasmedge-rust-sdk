//! Defines WasmEdge AST Module, ImportType, and ExportType.

use crate::{config::Config, ExternalInstanceType, WasmEdgeResult};
use std::{borrow::Cow, marker::PhantomData, path::Path};
use wasmedge_sys as sys;

/// Defines compiled in-memory representation of an input WASM binary.
///
/// A [Module] is a compiled in-memory representation of an input WebAssembly binary. In the instantiation process, a [Module] is instatiated to a module [instance](crate::Instance), from which the exported [function](crate::Func), [table](crate::Table), [memory](crate::Memory), and [global](crate::Global) instances can be fetched.
#[derive(Debug, Clone)]
pub struct Module {
    pub(crate) inner: sys::Module,
}
impl Module {
    /// Returns a validated module from a file.
    ///
    /// # Arguments
    ///
    /// * `config` - The global configuration.
    ///
    /// * `file` - A wasm file or an AOT wasm file.
    ///
    /// # Error
    ///
    /// If fail to load and valiate a module from a file, returns an error.
    pub fn from_file(config: Option<&Config>, file: impl AsRef<Path>) -> WasmEdgeResult<Self> {
        let inner_config = config.map(|cfg| &cfg.inner);

        // load module
        let inner_module = sys::Loader::create(inner_config)?.from_file(file.as_ref())?;

        // validate module
        sys::Validator::create(inner_config)?.validate(&inner_module)?;

        Ok(Self {
            inner: inner_module,
        })
    }

    /// Loads a WebAssembly binary module from in-memory bytes.
    ///
    /// # Arguments
    ///
    /// * `config` - The global configuration.
    ///
    /// * `bytes` - The in-memory bytes to be parsed.
    ///
    /// # Error
    ///
    /// If fail to load and valiate the WebAssembly module from the given in-memory bytes, returns an error.
    pub fn from_bytes(config: Option<&Config>, bytes: impl AsRef<[u8]>) -> WasmEdgeResult<Self> {
        let inner_config = config.map(|cfg| &cfg.inner);

        // load module
        let inner_module = sys::Loader::create(inner_config)?.from_bytes(bytes.as_ref())?;

        // validate module
        sys::Validator::create(inner_config)?.validate(&inner_module)?;

        Ok(Self {
            inner: inner_module,
        })
    }

    /// Returns the count of the imported WasmEdge instances in the [module](crate::Module).
    pub fn count_of_imports(&self) -> u32 {
        self.inner.count_of_imports()
    }

    /// Returns the [import types](crate::ImportType) of all imported WasmEdge instances in the [module](crate::Module).
    pub fn imports(&self) -> Vec<ImportType> {
        let mut imports = Vec::new();
        for inner_import in self.inner.imports() {
            let import = ImportType {
                inner: inner_import,
                _marker: PhantomData,
            };
            imports.push(import);
        }
        imports
    }

    /// Returns the count of the exported WasmEdge instances from the [module](crate::Module).
    pub fn count_of_exports(&self) -> u32 {
        self.inner.count_of_exports()
    }

    /// Returns the [export types](crate::ExportType) of all exported WasmEdge instances (including funcs, tables, globals and memories) from the [module](crate::Module).
    pub fn exports(&self) -> Vec<ExportType> {
        let mut exports = Vec::new();
        for inner_export in self.inner.exports() {
            let export = ExportType {
                inner: inner_export,
                _marker: PhantomData,
            };
            exports.push(export);
        }
        exports
    }

    /// Gets the [export type](crate::ExportType) by the name of a specific exported WasmEdge instance, such as func, table, global or memory instance.
    ///
    /// # Argument
    ///
    /// * `name` - The name of the target exported WasmEdge instance, such as func, table, global or memory instance.
    pub fn get_export(&self, name: impl AsRef<str>) -> Option<ExternalInstanceType> {
        let exports = self
            .exports()
            .into_iter()
            .filter(|x| x.name() == name.as_ref())
            .collect::<Vec<_>>();
        match exports.is_empty() {
            true => None,
            false => exports[0].ty().ok(),
        }
    }
}

/// Defines the types of the imported instances.
#[derive(Debug)]
pub struct ImportType<'module> {
    inner: sys::ImportType<'module>,
    _marker: PhantomData<&'module Module>,
}
impl<'module> ImportType<'module> {
    /// Returns the imported name of the WasmEdge instance.
    pub fn name(&self) -> Cow<'_, str> {
        self.inner.name()
    }

    /// Returns the name of the module hosting the imported WasmEdge instance.
    pub fn module_name(&self) -> Cow<'_, str> {
        self.inner.module_name()
    }

    /// Returns the type of the imported WasmEdge instance, which is one of the types defined in [ExternalInstanceType](wasmedge_types::ExternalInstanceType).
    pub fn ty(&self) -> WasmEdgeResult<ExternalInstanceType> {
        let ty = self.inner.ty()?;
        Ok(ty)
    }
}

/// Defines the types of the exported instances.
#[derive(Debug)]
pub struct ExportType<'module> {
    inner: sys::ExportType<'module>,
    _marker: PhantomData<&'module Module>,
}
impl<'module> ExportType<'module> {
    /// Returns the exported name of the WasmEdge instance.
    pub fn name(&self) -> Cow<'_, str> {
        self.inner.name()
    }

    /// Returns the type of the exported WasmEdge instance, which is one of the types defined in [ExternalInstanceType](wasmedge_types::ExternalInstanceType).
    pub fn ty(&self) -> WasmEdgeResult<ExternalInstanceType> {
        let ty = self.inner.ty()?;
        Ok(ty)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        error::{CoreError, CoreLoadError, WasmEdgeError},
        wat2wasm,
    };

    #[test]
    #[allow(clippy::assertions_on_result_states)]
    fn test_module_from_wasm() {
        // load wasm module from a specified wasm file
        let file = std::env::current_dir()
            .unwrap()
            .join("examples/wasmedge-sys/data/fibonacci.wat");

        let result = Module::from_file(None, file);
        assert!(result.is_ok());

        // attempt to load a non-existent wasm file
        let result = Module::from_file(None, "not_exist_file.wasm");
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            Box::new(WasmEdgeError::Core(CoreError::Load(
                CoreLoadError::IllegalPath
            )))
        );
    }

    #[test]
    #[allow(clippy::assertions_on_result_states)]
    fn test_module_from_wat() {
        // load wasm module from a specified wasm file
        let file = std::env::current_dir()
            .unwrap()
            .join("examples/wasmedge-sys/data/fibonacci.wat");

        let result = Module::from_file(None, file);
        assert!(result.is_ok());
    }

    #[test]
    #[allow(clippy::assertions_on_result_states)]
    fn test_module_from_bytes() {
        // read the wasm bytes
        let wasm_bytes = wat2wasm(
            br#"
            (module
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
        )
        .unwrap();

        let result = Module::from_bytes(None, wasm_bytes);
        assert!(result.is_ok());

        // attempt to load an empty buffer
        let result = Module::from_bytes(None, []);
        assert_eq!(
            result.unwrap_err(),
            Box::new(WasmEdgeError::Core(CoreError::Load(
                CoreLoadError::UnexpectedEnd
            ))),
        );
    }

    #[test]
    #[allow(clippy::assertions_on_result_states)]
    fn test_module_clone() {
        // read the wasm bytes
        let wasm_bytes = wat2wasm(
            br#"
            (module
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
        )
        .unwrap();

        let result = Module::from_bytes(None, wasm_bytes);
        assert!(result.is_ok());
        let module = result.unwrap();
        assert_eq!(module.exports().len(), 2);

        // clone the module
        let module_clone = module.clone();
        assert_eq!(module.exports().len(), module_clone.exports().len());
    }
}
