use crate::{io::WasmValTypeList, FuncType, WasmEdgeResult};
pub use sys::AsInstance;
use sys::Function;
use wasmedge_sys::{self as sys};

/// Creates a [import object](crate::ImportObject).
///
#[derive(Debug)]
pub struct ImportObjectBuilder<Data> {
    import_object: ImportObject<Data>,
}
impl<Data> ImportObjectBuilder<Data> {
    /// Creates a new [ImportObjectBuilder].
    pub fn new(name: &str, data: Data) -> WasmEdgeResult<Self> {
        let import_object = ImportObject::create(name, Box::new(data))?;
        Ok(Self { import_object })
    }

    /// Adds a [host function](crate::Func) to the [ImportObject] to create.
    ///
    /// N.B. that this function can be used in thread-safe scenarios.
    ///
    /// # Arguments
    ///
    /// * `name` - The exported name of the [host function](crate::Func) to add.
    ///
    /// * `real_func` - The native function.
    ///
    /// * `data` - The host context data used in this function.
    ///
    /// # error
    ///
    /// If fail to create or add the [host function](crate::Func), then an error is returned.
    pub fn with_func<Args, Rets>(
        &mut self,
        name: impl AsRef<str>,
        real_func: sys::SyncFn<Data>,
    ) -> WasmEdgeResult<&mut Self>
    where
        Args: WasmValTypeList,
        Rets: WasmValTypeList,
    {
        let args = Args::wasm_types();
        let returns = Rets::wasm_types();
        let ty = FuncType::new(args.to_vec(), returns.to_vec());
        let func = unsafe {
            Function::create_sync_func(&ty, real_func, self.import_object.get_host_data_mut(), 0)
        }?;
        self.import_object.add_func(name, func);

        Ok(self)
    }

    /// Adds a [host function](crate::Func) to the [ImportObject] to create.
    ///
    /// N.B. that this function can be used in thread-safe scenarios.
    ///
    /// # Arguments
    ///
    /// * `name` - The exported name of the [host function](crate::Func) to add.
    ///
    /// * `ty` - The function type.
    ///
    /// * `real_func` - The native function.
    ///
    /// * `data` - The host context data used in this function.
    ///
    /// # error
    ///
    /// If fail to create or add the [host function](crate::Func), then an error is returned.
    pub fn with_func_by_type(
        &mut self,
        name: impl AsRef<str>,
        ty: FuncType,
        real_func: sys::SyncFn<Data>,
    ) -> WasmEdgeResult<&mut Self> {
        let func = unsafe {
            Function::create_sync_func(&ty, real_func, self.import_object.get_host_data_mut(), 0)
        }?;
        self.import_object.add_func(name, func);
        Ok(self)
    }

    /// Adds a [global](crate::Global) to the [ImportObject] to create.
    ///
    /// # Arguments
    ///
    /// * `name` - The exported name of the [global](crate::Global) to add.
    ///
    /// * `global` - The wasm [global instance](crate::Global) to add.
    ///
    pub fn with_global(mut self, name: impl AsRef<str>, global: sys::Global) -> Self {
        self.import_object.add_global(name, global);
        self
    }

    /// Adds a [memory](crate::Memory) to the [ImportObject] to create.
    ///
    /// # Arguments
    ///
    /// * `name` - The exported name of the [memory](crate::Memory) to add.
    ///
    /// * `memory` - The wasm [memory instance](crate::Memory) to add.
    ///
    pub fn with_memory(mut self, name: impl AsRef<str>, memory: sys::Memory) -> Self {
        self.import_object.add_memory(name, memory);
        self
    }

    /// Adds a [table](crate::Table) to the [ImportObject] to create.
    ///
    /// # Arguments
    ///
    /// * `name` - The exported name of the [table](crate::Table) to add.
    ///
    /// * `table` - The wasm [table instance](crate::Table) to add.
    ///
    pub fn with_table(mut self, name: impl AsRef<str>, table: sys::Table) -> Self {
        self.import_object.add_table(name, table);
        self
    }

    /// Creates a new [ImportObject].
    ///
    /// # Argument
    ///
    /// * `name` - The name of the [ImportObject] to create.
    ///
    /// * `host_data` - The host context data to be stored in the module instance.
    ///
    /// # Error
    ///
    /// If fail to create the [ImportObject], then an error is returned.
    pub fn build(self) -> ImportObject<Data> {
        self.import_object
    }
}

/// Defines an import object that contains the required import data used when instantiating a [module](crate::Module).
///
/// An [ImportObject] instance is created with [ImportObjectBuilder](crate::ImportObjectBuilder).
pub type ImportObject<T> = sys::ImportModule<T>;
