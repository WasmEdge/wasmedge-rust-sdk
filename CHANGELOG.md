# Changelog

All notable changes to this project will be documented in this file.

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
