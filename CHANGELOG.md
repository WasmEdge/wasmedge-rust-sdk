# Changelog

All notable changes to this project will be documented in this file.

## [0.11.0] - 2023-07-31

### ⛰️  Features

- Add `Func::wrap_async_func_with_type` ([#43](https://github.com/WasmEdge/wasmedge-rust-sdk/pull/43))
- Add `WasiInstance::exit_code` in `async` mod ([#43](https://github.com/WasmEdge/wasmedge-rust-sdk/pull/43))
- Add `WasiInstance::name` in `wasi` mod ([#42](https://github.com/WasmEdge/wasmedge-rust-sdk/pull/42))
- Add `WasiContext` ([#39](https://github.com/WasmEdge/wasmedge-rust-sdk/pull/39))
- Add `VmBuilder::with_wasi_context` ([#39](https://github.com/WasmEdge/wasmedge-rust-sdk/pull/39))

### 🚜 Refactor

- [BREAKING] Update `Func::new`
  - Rename `Func::new` to `Func::wrap_with_type` ([#43](https://github.com/WasmEdge/wasmedge-rust-sdk/pull/43))
  - Change the type of the `data` argument to `Option<Box<T>>` ([#39](https://github.com/WasmEdge/wasmedge-rust-sdk/pull/39))
- [BREAKING] Update `Func::wrap_func`
  - Rename `Func::wrap_func` to `Func::wrap` ([#43](https://github.com/WasmEdge/wasmedge-rust-sdk/pull/43))
  - The type of the `data` argument is changed to `Option<Box<T>>` ([#39](https://github.com/WasmEdge/wasmedge-rust-sdk/pull/39))
- [BREAKING] Update async `WasiInstance`
  - Move `WasiInstance` for `async` scenarios from `wasi` mod to `async` mod ([#39](https://github.com/WasmEdge/wasmedge-rust-sdk/pull/39))
  - Remove the implementation of `AsInstance` trait for `WasiInstance` defined in `async` mod ([#39](https://github.com/WasmEdge/wasmedge-rust-sdk/pull/39))
  - Remove `WasiInstance::initialize` defined in `async` mod ([#39](https://github.com/WasmEdge/wasmedge-rust-sdk/pull/39))
- [BREAKING] Update `WasiInstance`
  - Remove the implementation of `AsInstance` trait for `WasiInstance` defined in `wasi` mod ([#42](https://github.com/WasmEdge/wasmedge-rust-sdk/pull/42))
- [BREAKING] Move `AsyncState` into `async` mod ([#39](https://github.com/WasmEdge/wasmedge-rust-sdk/pull/39))
- [BREAKING] Remove `HostFn<T>` and `AsyncHostFn<T>` ([#39](https://github.com/WasmEdge/wasmedge-rust-sdk/pull/39))
- [BREAKING] Update `ImportObjectBuilder`
  - Add `?Size` and `Clone` trait bounds on generic type of `ImportObjectBuilder::build` ([#41](https://github.com/WasmEdge/wasmedge-rust-sdk/pull/41))
  - Change the type of the `data` argument of `ImportObjectBuilder::with_func` to `Option<Box<D>>` ([#39](https://github.com/WasmEdge/wasmedge-rust-sdk/pull/39))
  - Change the type of the `data` argument of `ImportObjectBuilder::with_func_by_type` to `Option<Box<D>>` ([#39](https://github.com/WasmEdge/wasmedge-rust-sdk/pull/39))
- [BREAKING] Update `ImportObject`
  - Add generic type to `ImportObject` ([#41](https://github.com/WasmEdge/wasmedge-rust-sdk/pull/41))
  - Rename `as_raw_ptr` to `as_ptr` ([#41](https://github.com/WasmEdge/wasmedge-rust-sdk/pull/41))
- [BREAKING] Update `PluginModuleBuilder`
  - Change the type of the `data` argument of `PluginModuleBuilder::with_func` to `Option<Box<D>>` ([#39](https://github.com/WasmEdge/wasmedge-rust-sdk/pull/39))
  - Add `?Sized` trait bound on the generic type of `PluginModuleBuilder<T>` ([#42](https://github.com/WasmEdge/wasmedge-rust-sdk/pull/42))
  - Update `PluginModuleBuilder::build` ([#42](https://github.com/WasmEdge/wasmedge-rust-sdk/pull/42))
- [BREAKING] Update `PluginModule` ([#42](https://github.com/WasmEdge/wasmedge-rust-sdk/pull/42))
- [BREAKING] Add generic type to `Store::register_import_module` ([#41](https://github.com/WasmEdge/wasmedge-rust-sdk/pull/41))
- [BREAKING] Update `async_host_function` proc-macro ([#43](https://github.com/WasmEdge/wasmedge-rust-sdk/pull/43))
- Update `Vm`
  - Remove `imports` field from `Vm` ([#41](https://github.com/WasmEdge/wasmedge-rust-sdk/pull/41))
  - [BREAKING] Update the signature of `Vm::register_import_module` ([#41](https://github.com/WasmEdge/wasmedge-rust-sdk/pull/41))
  - Update `Vm::build` for async scenarios ([#42](https://github.com/WasmEdge/wasmedge-rust-sdk/pull/42))
  - Enable `Vm::wasi_module` and `Vm::wasi_module_mut` for async scenarios ([#43](https://github.com/WasmEdge/wasmedge-rust-sdk/pull/43))
- Update `VmBuilder::build` ([#39](https://github.com/WasmEdge/wasmedge-rust-sdk/pull/39))
- Improve the `standalone` deployment mode ([#40](https://github.com/WasmEdge/wasmedge-rust-sdk/pull/40))

### 📚 Documentation

- Update `README.md` ([#43](https://github.com/WasmEdge/wasmedge-rust-sdk/pull/43))
- Update Rust SDK API Document ([#44](https://github.com/WasmEdge/wasmedge-rust-sdk/pull/44))

### Ci

- Add steps for publishing async API document in `release-wasmedge-sdk` workflow ([#44](https://github.com/WasmEdge/wasmedge-rust-sdk/pull/44))

## [0.10.0] - 2023-07-21

### ⛰️  Features

- Support closures in `Func` and `ImportObjectBuilder` ([#20](https://github.com/WasmEdge/wasmedge-rust-sdk/pull/20))
  - [BREAKING] Update `Func::new` method
  - [BREAKING] Update `Func::wrap_func` method
  - [BREAKING] Update `Func::wrap_async_func` method
  - [BREAKING] Update `ImportObjectBuilder::with_func` method
  - [BREAKING] Update `ImportObjectBuilder::with_func_by_type` method
  - [BREAKING] Update `ImportObjectBuilder::with_async_func` method

- Support `host_data` in `ImportObjectBuilder::with_async_func` and `ImportObjectBuilder::build` methods ([#21](https://github.com/WasmEdge/wasmedge-rust-sdk/pull/21))

- Support standalone static libraries ([#22](https://github.com/WasmEdge/wasmedge-rust-sdk/pull/22) [#24](https://github.com/WasmEdge/wasmedge-rust-sdk/pull/24))

### 🚜 Refactor

- [BREAKING] Rename `Func::wrap` to `Func::wrap_func` ([#20](https://github.com/WasmEdge/wasmedge-rust-sdk/pull/20))
- [BREAKING] Rename `Func::wrap_async` to `Func::wrap_async_func` ([#20](https://github.com/WasmEdge/wasmedge-rust-sdk/pull/20))
- [BREAKING] Rename `ImportObjectBuilder::with_func_async` to `ImportObjectBuilder::with_async_func` ([#20](https://github.com/WasmEdge/wasmedge-rust-sdk/pull/20))
- Remove the `host_data` field in `ImportObjectBuilder` ([#21](https://github.com/WasmEdge/wasmedge-rust-sdk/pull/21))
  - [BREAKING] Update `ImportObjectBuilder::with_async_func` method
- Remove the generic type in `ImportObject` ([#21](https://github.com/WasmEdge/wasmedge-rust-sdk/pull/21))
  - [BREAKING] Update `VmBuilder::build` method
  - [BREAKING] Remove the generic type in `Vm`

### 📚 Documentation

- Update README and rustdoc ([#28](https://github.com/WasmEdge/wasmedge-rust-sdk/pull/28))

## [0.9.0] - 2023-06-30

### ⛰️  Features

- Introduce `NeverType` type ([WasmEdge #2497](https://github.com/WasmEdge/WasmEdge/pull/2497))
  - [BREAKING] Update `Func::new` method
  - [BREAKING] Update `Func::wrap` method
  - [BREAKING] Update `ImportObjectBuilder::with_func` method
  - [BREAKING] Update `ImportObjectBuilder::with_func_by_type` method
- Support async wasi ([WasmEdge #2528](https://github.com/WasmEdge/WasmEdge/pull/2528))
  - [BREAKING] Update `Executor::run_func_async` method
  - [BREAKING] Update `Executor::run_func_ref_async` method
  - [BREAKING] Update `Func::run_async` method
  - [BREAKING] Update `FuncRef::run_async` method
  - [BREAKING] Update `ImportObjectBuilder::with_func_async` method
  - [BREAKING] Update `Vm::run_func_async` method
  - [BREAKING] Update `Vm::run_func_from_module_async` method
  - [BREAKING] Update `Vm::run_func_from_file_async` method
  - [BREAKING] Update `Vm::run_func_from_bytes_async` method
- Migrate WasmEdge Rust SDK into [WasmEdge/wasmedge-rust-sdk](https://github.com/WasmEdge/wasmedge-rust-sdk) ([#1](https://github.com/WasmEdge/wasmedge-rust-sdk/pull/1))
- Migrate async-wasi into Rust SDK ([#2](https://github.com/WasmEdge/wasmedge-rust-sdk/pull/2))
- Implement a separate VmBuilder::build method for `async` cases ([#3](https://github.com/WasmEdge/wasmedge-rust-sdk/pull/3))
- Support new WasmEdge C-API: `WasmEdge_Driver_UniTool` ([#6](https://github.com/WasmEdge/wasmedge-rust-sdk/pull/6))
- Support new C-APIs: `WasmEdge_ModuleInstanceCreateWithData` and `WasmEdge_ModuleInstanceGetHostData` ([#13](https://github.com/WasmEdge/wasmedge-rust-sdk/pull/13))
  - [BREAKING] Update `VmDock` type
  - [BREAKING] Update `Param::settle` method
  - [BREAKING] Update `Param::allocate` method
  - [BREAKING] Update `ImportObjectBuilder` type
  - [BREAKING] Update `ImportObjectBuilder::with_func` method
  - [BREAKING] Update `ImportObjectBuilder::with_func_by_type` method
  - [BREAKING] Update `ImportObjectBuilder::with_global` method
  - [BREAKING] Update `ImportObjectBuilder::with_memory` method
  - [BREAKING] Update `ImportObjectBuilder::with_table` method
  - [BREAKING] Update `ImportObjectBuilder::build` method
  - [BREAKING] Update `ImportObject` type
  - [BREAKING] Update `Store::register_import_module` method
  - [BREAKING] Update `VmBuilder::build` method
  - [BREAKING] Update `Vm` type
  - [BREAKING] Update `Vm::register_import_module` method
- Implement `PluginModule` and `PluginModuleBuilder` ([#14](https://github.com/WasmEdge/wasmedge-rust-sdk/pull/14))
  - [BREAKING] Update `ImportObjectBuilder::with_func` method
  - [BREAKING] Update `ImportObjectBuilder::with_func_by_type` method
  - [BREAKING] Update `ImportObjectBuilder::with_func_async` method
  - [BREAKING] Update `ImportObjectBuilder::with_host_data` method

### 📚 Documentation

- Update README ([#7](https://github.com/WasmEdge/wasmedge-rust-sdk/pull/7))

### ⚙️ Miscellaneous Tasks

- Remove the deprecated examples ([#4](https://github.com/WasmEdge/wasmedge-rust-sdk/pull/4))
- Remove the deprecated examples ([#8](https://github.com/WasmEdge/wasmedge-rust-sdk/pull/8))
- Release preparation: bump versions and update docs ([#15](https://github.com/WasmEdge/wasmedge-rust-sdk/pull/15))
- Update documentation url ([#17](https://github.com/WasmEdge/wasmedge-rust-sdk/pull/17))

### Ci

- Update the release workflows ([#5](https://github.com/WasmEdge/wasmedge-rust-sdk/pull/5))
- Add `standlone` workflow ([#9](https://github.com/WasmEdge/wasmedge-rust-sdk/pull/9))
- Support `macOS` and `Fedora` in the `standalone` workflow ([#11](https://github.com/WasmEdge/wasmedge-rust-sdk/pull/11))
- Update the `release-async-wasi` workflow ([#16](https://github.com/WasmEdge/wasmedge-rust-sdk/pull/16))
