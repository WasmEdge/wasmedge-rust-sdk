# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

Targets `wasmedge-sdk` 0.17.0 / `wasmedge-sys` 0.21.0 / `wasmedge-types` 0.7.0 /
`wasmedge-macro` 0.7.0 / `async-wasi` 0.3.0, all still pinned against WasmEdge C API
0.17.1. [See the upgrade guide.](docs/Upgrade_to_0.17.0.md)

**A note on the gap since 0.13.5-newapi:** this repository's manifests were bumped
to `0.14.1` and then `0.16.1` (with matching bumps in `wasmedge-sys` up to `0.20.0`)
without a corresponding CHANGELOG update, and — as discovered while preparing this
release — **those versions were never actually published to crates.io**. The real
latest published versions going into this release are `wasmedge-sdk` 0.14.0 and
`wasmedge-sys` 0.19.4. Whatever changed between 0.13.5-newapi and 0.14.0, and
between 0.14.0 and this modernization branch's starting point, has not been
reconstructed here — it predates this effort and there is no reliable record of it.
The compatibility matrices in [README.md](README.md#compatibility-matrix) and
[lib.rs](src/lib.rs) annotate the unpublished `0.16.1`/`0.14.1` rows rather than
deleting them, for historical accuracy.

### Added

- `wasmedge-sdk`: `vm::SyncInst` (and `vm::AsyncInst` under the `async` feature) are
  now re-exported at the crate root as `wasmedge_sdk::SyncInst`/`AsyncInst`. Callers
  needed this trait to name their `HashMap<String, &mut dyn SyncInst>` instance maps
  but previously had to reach into the `#[doc(hidden)]` `vm` module to name it; the
  old path still works.

### Changed

- **MSRV is now 1.85** (previously stated as 1.71 in docs but not enforced anywhere)
  and **all five crates moved to Rust edition 2024** (`wasmedge-sdk`, `wasmedge-sys`,
  `wasmedge-types`, `wasmedge-macro`, `async-wasi`).
- Dependency upgrades: `thiserror` 1 -> 2, `socket2` 0.4 -> 0.6, `getrandom` 0.2 -> 0.4,
  `bindgen` 0.69 -> 0.72 (needed for edition-2024-shaped bindings), `reqwest`
  (`wasmedge-sys` build-dependency only) 0.11 -> 0.12 with `rustls-tls-webpki-roots`
  preserved (no change in trusted roots or proxy/`WASMEDGE_STANDALONE_ARCHIVE`
  behavior). `path-absolutize` was replaced with a small local
  `std::path`-based normalization helper, removing that dependency entirely.
- If you set `WASMEDGE_RUST_BINDGEN_PATH` to use an external `bindgen` executable,
  it must now be **bindgen-cli 0.71 or newer**: the build script passes
  `--rust-edition 2024`, a flag older releases don't recognize.
- `Cargo.lock` is now committed to the repository (previously gitignored), and
  `cargo semver-checks`, `cargo-deny`, a `cargo doc -D warnings` docs gate, and a
  `cargo hack --each-feature` job were added to CI (`guardrails.yml`).
- Documentation and crate metadata truth pass: refreshed README/lib.rs badges,
  compatibility matrices, and MSRV notice; fixed the License link (was pointing at
  `tensorflow/rust`); pointed every crate's `documentation` manifest field and the
  README/lib.rs API Reference link at docs.rs instead of a 19-months-stale gh-pages
  mirror (the second-state async-enabled mirror is unchanged); added
  `[package.metadata.docs.rs]` and `keywords` to `wasmedge-types`, `wasmedge-macro`,
  and `async-wasi`.

### Deprecated

- `wasmedge-macro`: all six procedural macros — the public `host_function` and
  `async_host_function`, plus the `#[doc(hidden)]` `sys_host_function`,
  `sys_async_host_function`, `sys_wasi_host_function`, and
  `sys_async_wasi_host_function` — are now `#[deprecated(since = "0.7.0")]`. They
  expand to the pre-0.14 host-function ABI (a `Caller`-based three-argument free
  function) that no longer compiles against `wasmedge-sdk` >= 0.14 /
  `wasmedge-sys` >= 0.19, so they have had zero working callers since 0.14.0.
  Write the host function directly and register it with
  `ImportObjectBuilder::with_func` (or, at the `wasmedge-sys` layer,
  `Function::create_sync_func` + `ImportModule::add_func`).

### Fixed

All of the following are internal correctness fixes with no public API shape
change (see [the upgrade guide](docs/Upgrade_to_0.17.0.md) for the one exception,
called out below):

- `async-wasi`:
  - `fd_prestat_dir_name` could read past the end of its buffer using a
    guest-controlled length, panicking instead of erroring.
  - `path_unlink_file` checked the `PATH_REMOVE_DIRECTORY` rights bit instead of
    `PATH_UNLINK_FILE`.
  - `path_open` returned `EEXIST` instead of `NOENT` for a missing path when
    `O_CREAT` was not set.
  - Socket timeouts (`tv_usec`) were interpreted as nanoseconds instead of
    microseconds, so any timeout at or above ~1 second would panic constructing
    the `Duration`.
  - `sock_getaddrinfo` wrote into `sa_data` without checking its length, a
    guest-triggerable panic.
  - Socket registration used `mem::zeroed::<Socket>()` to fake ownership during a
    swap; replaced with a proper `Option`-based state machine.
  - `SocketWritable::poll` didn't register a waker on `Pending`, relying entirely
    on an outer 10s timeout and silently dropping its `Result`; it now uses
    `tokio::sync::Notify` for real wakeups.
  - As a side effect of the `socket2` 0.4 -> 0.6 upgrade above, `async-wasi` (and
    therefore the whole workspace) now **compiles natively on macOS** — it
    previously failed with an `E0599` from `socket2` 0.4.10's narrower API.
- `wasmedge-sys` (memory safety, all internal):
  - `Statistics` double-freed when cloned (`Drop` was implemented on the wrong
    type) and `Executor::create` could leave a dangling statistics pointer.
  - `InnerFunc`/`InnerModule` (`Copy`) and `InnerInstance`/`InnerExecutor`
    (`Clone`) let any internal clone silently double-free the underlying FFI
    handle; the derives are gone.
  - `Loader::from_bytes` and `Compiler::compile_from_bytes` leaked their
    intermediate buffer on the error path and mishandled a `malloc(0)` edge case.
  - `ImportModule::create` didn't check for a null context pointer.
  - `ImportModule::from_raw` double-freed a borrowed module name.
  - `Executor::call_func_ref` panicked via `.unwrap()` on a type mismatch; it now
    returns `FuncError::Type`.
  - `box_future` relied on every closure it boxes being zero-sized (via
    `mem::zeroed::<F>()`) without checking it; it's now guarded by a
    `const { assert!(size_of::<F>() == 0) }`.
  - **Observable behavior change:** `Validator::create` returned the wrong error
    variant (`CompilerCreate`) on failure; it now correctly returns
    `ValidatorCreate`. This changes the `Err` value callers see (not the function
    signature), so `cargo semver-checks` does not flag it — see the upgrade guide.
- `wasmedge-types`: `GlobalError::UnmatchedValType`'s `Display` impl was an empty
  `#[error("")]`; it now has a real message.
- ~20 broken rustdoc intra-doc links across `wasmedge-sdk` and `wasmedge-sys`
  (mostly `crate::Func`/`Table`/`Memory`/`Global`/`Executor`, none of which exist
  in the sdk's public surface) now resolve to the real `wasmedge-sys` types.

### Removed

- **Breaking (nominal):** `wasmedge-sdk` no longer re-exports the `host_function`
  and `async_host_function` attribute macros (`wasmedge_sdk::host_function` /
  `wasmedge_sdk::async_host_function`). They re-exported the now-deprecated
  `wasmedge-macro` macros above, which expand to code that does not compile
  against the current API, so no working caller could reference them. Removed
  while riding the 0.17.0 major-version bump so it is not a surprise on a
  patch/minor release. Import from `wasmedge_macro` directly if you still need
  the (deprecated) names.
- Dead code: `src/dock.rs` and `src/executor.rs` (883 LOC, unreachable, referenced
  types that no longer exist), `crates/async-wasi/src/snapshots/common/vfs/sync.rs`
  (1148 LOC, an orphaned module never declared by its parent), 15 bit-rotted
  `examples/wasmedge-sys/*` files using deleted APIs, and a `#[cfg(not(feature =
  "async"))]` test module in `src/compiler.rs` that — precisely because `async` is
  a default feature — never actually compiled under CI/default builds and silently
  broke `--no-default-features` builds (it referenced the already-removed
  `VmBuilder`).
- Unused dependencies: `anyhow`, `num-derive`, `num-traits`, `cfg-if` from
  `wasmedge-sdk`; `paste` (RUSTSEC-2024-0436, unmaintained), `rand`, `lazy_static`,
  `parking_lot`, `thiserror`, `cfg-if`, `wasmedge-macro`, the `cmake` build-dependency,
  and the `anyhow` dev-dependency from `wasmedge-sys`; `serde`, `serde_json`, and
  `parking_lot` from `async-wasi`; the `extra-traits` feature of `syn` from
  `wasmedge-macro`.

## [0.13.5-newapi] - 2024-04-30

[The sdk has changed a lot, please read this document.](docs/Upgrade_to_0.14.0.md)
### 🚜 Refactor

- [BREAKING] Significant refactoring was done to fix memory leaks ([#98](https://github.com/WasmEdge/wasmedge-rust-sdk/pull/98))

## [0.13.2] - 2023-11-15

### 🐛 Bug Fixes

- Fix the static build to link against `zstd` ([#91](https://github.com/orhun/git-cliff/issues/91))

## [0.13.1] - 2023-11-14

- Update the `wat` dep. This update is to fix [#88](https://github.com/WasmEdge/wasmedge-rust-sdk/issues/88).

## [0.13.0] - 2023-11-07

### ⛰️  Features

- New API `PluginManager::nn_preload`. This API is used to initialize the `wasi_nn` plug-in with given preloads ([#74](https://github.com/WasmEdge/wasmedge-rust-sdk/pull/74))

- Implement `FromStr` trait for `NNPreload` struct ([#81](https://github.com/WasmEdge/wasmedge-rust-sdk/pull/81))

### 🚜 Refactor

- [BREAKING] Update the argument types ([#82](https://github.com/WasmEdge/wasmedge-rust-sdk/pull/82)):
  - `VmBuilder::with_plugin`
  - `VmBuilder::with_plugin_wasi_nn`
  - `VmBuilder::with_plugin_wasi_crypto`
  - `VmBuilder::with_plugin_wasmedge_process`

- [BREAKING] Update the argument types ([#76](https://github.com/WasmEdge/wasmedge-rust-sdk/pull/76)):
  - `Executor::run_func_with_timeout` and `Executor::run_func_async_with_timeout`
  - `Vm::run_func_with_timeout` and `Vm::run_func_async_with_timeout`
  - `Func::run_with_timeout` and `Func::run_async_with_timeout`

### 🐛 Bug Fixes

- Introduce new C-API `WasmEdge_FunctionInstanceGetData` to fix the memory leak issue caused by host data ([#84](https://github.com/WasmEdge/wasmedge-rust-sdk/pull/84))

### Ci

- Support `macos-13` and remove `macos-11` from the `ci-build` and `standalone` workflows ([#84](https://github.com/WasmEdge/wasmedge-rust-sdk/pull/84))

## [0.12.2] - 2023-09-22

### 🚜 Refactor

- Disable `timeout` related APIs for the `musl` environment ([#71](https://github.com/WasmEdge/wasmedge-rust-sdk/pull/71))

## [0.12.1] - 2023-09-14

### 🐛 Bug Fixes

- *(wasmedge-sys)* Update the `sha256sum` of WasmEdge 0.13.4 ([#66](https://github.com/WasmEdge/wasmedge-rust-sdk/pull/66))

## [0.12.0] - 2023-09-10

### ⛰️  Features

- New `timeout` APIs ([#61](https://github.com/WasmEdge/wasmedge-rust-sdk/pull/61))
  - Add `Vm::run_func_with_timeout` and `Vm::run_func_async_with_timeout`. These APIs are used to run a host function with a timeout
  - Add `Executor::run_func_with_timeout` and `Executor::run_func_async_with_timeout`. These APIs are used to run a host function with a timeout
  - Add `Func::run_with_timeout` and `Func::run_async_with_timeout`. These APIs are used to run a host function with a timeout
- New API `Store::register_plugin_module`. This API is used to register a `PluginInstance`` into a store instance ([#53](https://github.com/WasmEdge/wasmedge-rust-sdk/pull/53))
- New type alias `PluginInstance` ([#53](https://github.com/WasmEdge/wasmedge-rust-sdk/pull/53))

### 🚜 Refactor

- [BREAKING] Merge `async` mod into `wasi` mod ([#55](https://github.com/WasmEdge/wasmedge-rust-sdk/pull/55))
- [BREAKING] Update the return type of `PluginManager::find` from `Option<Plugin>` to `WasmEdgeResult<Plugin>` ([#53](https://github.com/WasmEdge/wasmedge-rust-sdk/pull/53))
- [BREAKING] Update the return type of `Plugin::mod_instance` from `Option<Instance>` to `WasmEdgeResult<PluginInstance>` ([#53](https://github.com/WasmEdge/wasmedge-rust-sdk/pull/53))

### 📚 Documentation

- Update WasmEdge RustSDK API Document ([#55](https://github.com/WasmEdge/wasmedge-rust-sdk/pull/55))

## [0.11.2] - 2023-08-07

### ⛰️  Features

- New API `WasiContext::generate`. This API provides more flexible argument types than the existed `WasiContext::new` ([#49](https://github.com/WasmEdge/wasmedge-rust-sdk/pull/49))

### 🚜 Refactor

- Improve `host_function` and `async_host_function` proc-macros ([#49](https://github.com/WasmEdge/wasmedge-rust-sdk/pull/49))
- Improve build script ([#48](https://github.com/WasmEdge/wasmedge-rust-sdk/pull/48))
  - Options to specify the type of linking required for the different dependencies using environment variables.
  - Adds an option to use an external `rust-bindgen` using environment variables.
  - Adds support for `musl libc`

### 📚 Documentation

- Update `README.md` ([#50](https://github.com/WasmEdge/wasmedge-rust-sdk/pull/50))
- Update Rust SDK API Document ([#50](https://github.com/WasmEdge/wasmedge-rust-sdk/pull/50))

### Ci

- Disable the publish of the async API document ([#50](https://github.com/WasmEdge/wasmedge-rust-sdk/pull/50))

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
