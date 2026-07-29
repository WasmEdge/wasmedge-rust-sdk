# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

Rust bindings for the [WasmEdge](https://wasmedge.org/) WebAssembly runtime. The crate stack wraps the
WasmEdge **C API** (`libwasmedge`), so almost every build/test problem is really a "where is the C library"
problem — read the build section before debugging a link error.

## Crate layout

```
wasmedge-sdk        (src/)                   idiomatic API: Vm, Store, Module, ImportObjectBuilder, Instance
  └─ wasmedge-sys   (crates/wasmedge-sys)    unsafe 1:1 wrappers + bindgen-generated `ffi` module
       └─ libwasmedge                        the C library (dynamically or statically linked)
wasmedge-types      (crates/wasmedge-types)  shared value/type/error definitions used by both layers
wasmedge-macro      (crates/wasmedge-macro)  proc macros: host_function, async_host_function, sys_* variants
async-wasi          (crates/async-wasi)      pure-Rust async WASI preview_1 over tokio (Linux-only in practice)
test-wasm/                                   guest module for tests; excluded from the workspace
```

`wasmedge-sys` is meant for building libraries; `wasmedge-sdk` is meant for applications.

## Build prerequisites

`crates/wasmedge-sys/build.rs` must locate `libwasmedge` headers + library. Two paths:

- **No local install** — `--features bundled` (= `standalone` + `static`): downloads a pinned prebuilt archive
  and links it statically. CI exercises this on both Ubuntu and macOS arm64 (`rust-static-lib.yml`).
  Linux needs `libzstd-dev libfmt-dev` (`zstd-devel fmt-devel` on Fedora).
- **System install** — searched in order (`SEARCH_LOCATIONS`, build.rs:34): `$WASMEDGE_INCLUDE_DIR`/`$WASMEDGE_LIB_DIR`
  → `$WASMEDGE_DIR` → `$WASMEDGE_BUILD_DIR` → `$HOME/.wasmedge` → `/usr/local` → `$HOME/.local`.
  Dynamic linking also needs `LD_LIBRARY_PATH` (`DYLD_LIBRARY_PATH` on macOS) set to `$WASMEDGE_DIR/lib` at
  **run** time.

The downloaded runtime version is pinned by `WASMEDGE_RELEASE_VERSION` (build.rs:12) plus a sha256 table
`REMOTE_ARCHIVES` (build.rs:13); workflows pin the same value in `WASMEDGE_VERSION`. Bumping the runtime means
updating the constant, every sha, and the workflow envs together.

Feature meanings: `standalone` = download at build time (system search is skipped), `static` = link
`libwasmedge.a`, `bundled` = both, `ffi` = re-export raw bindings, `aot`/`wasi_crypto`/`wasi_nn`/`wasmedge_process`
= optional API surface, `async` = fiber-based async + async-wasi (Linux only, but on by default).

## Commands

Substitute `--features bundled` below with nothing if you have a system libwasmedge.

```bash
# Build
cargo build --release --features bundled

# Full test run — macOS/Windows (async-wasi does not compile there)
cargo test --release --features bundled,aot,ffi --workspace --exclude async-wasi \
  -- --nocapture --test-threads=1 --skip test_vmbuilder

# Full test run — Linux, with async
cargo test --release --features bundled,aot,async,ffi --workspace \
  -- --nocapture --test-threads=1 --skip test_vmbuilder

# One test (substring match), or exact
cargo test --release --features bundled -p wasmedge-sys test_memory -- --nocapture --test-threads=1
cargo test --release --features bundled -p wasmedge-sdk vm::tests::test_vm_run_func_from_bytes -- --exact --nocapture

# Integration tests — build the guest module first (separate crate, separate target)
cd test-wasm && cargo build --release --target wasm32-unknown-unknown && cd ..
cargo test --test integration_test --release --features bundled -- --nocapture

# Some wasmedge-sys async tests need prebuilt WASI guests
cd examples/wasmedge-sys
rustup target add wasm32-wasip1
rustc async_hello.rs --target=wasm32-wasip1 -o async_hello.wasm
rustc hello.rs --target=wasm32-wasip1 -o hello.wasm

# Lint — Linux gate runs both variants; macOS/Windows gate uses only `aot,ffi`
cargo clippy --lib --examples --features aot,wasi_nn,wasmedge_process,ffi -- -D warnings
cargo clippy --lib --examples --features aot,async,wasi_nn,wasmedge_process,ffi -- -D warnings

# Format — MUST be nightly; .rustfmt.toml uses nightly-only options that stable
# silently ignores while still exiting 0. CI pins this toolchain.
cargo +nightly-2026-07-16 fmt --all -- --check

# Docs — `docsrs` cfg enables doc_cfg, so nightly is required
RUSTDOCFLAGS="--cfg docsrs" cargo +nightly doc -p wasmedge-sdk --workspace --no-deps \
  --features aot,wasi_crypto,wasi_nn,wasmedge_process,ffi
```

Test-run invariants, all enforced by CI:

- `--test-threads=1` everywhere — do not drop it.
- `--skip test_vmbuilder` on Linux: that test is `#[cfg(target_os = "linux")]` and requires the `wasi_crypto`
  plugin to be installed. Only the scheduled canary (`ci-build.yml`, which builds WasmEdge with
  `-DWASMEDGE_PLUGIN_WASI_CRYPTO=On`) runs it.
- `--exclude async-wasi` off Linux: async-wasi uses Linux-only socket APIs (`SO_BINDTODEVICE` family).

## Architecture notes

**Post-0.14 API model.** There is no `VmBuilder`. You build a `HashMap<String, &mut dyn SyncInst>` of host
instances, pass it to `Store::new(config, map)`, then `Vm::new(store)`. The store *borrows* those instances
(`Store<'inst, T>`, src/store.rs:11), so every import object and WASI module must outlive the `Vm` — this is why
call sites look like `instances.insert(name, wasi.as_mut())`. WASI is an ordinary module instance created by
`wasi::WasiModule::create(args, envs, preopens)`, not a config flag. `docs/Upgrade_to_0.14.0.md` is the
migration reference from the old `VmBuilder`/`HostRegistration` world and is the best explanation of the
current shape.

`Store` keeps two disjoint maps: host-provided `instances` and wasm-instantiated `wasm_instance_map`.
`Vm::run_func` resolves a module name against the first, then the second (src/store.rs:115, src/vm.rs:134).

**Ownership across FFI (wasmedge-sys).** Owned handles are newtypes over a raw ctx pointer whose `Drop` calls
`WasmEdge_*Delete` (e.g. `Instance`/`InnerInstance`, crates/wasmedge-sys/src/instance/module.rs:15). Borrowed
children — a memory, table, or global exported by an instance — are returned as `InnerRef<D, &T>` /
`InnerRef<D, &mut T>`: `ManuallyDrop<D>` plus a `PhantomData<Ref>` that ties the child's lifetime to the parent
without ever running the child's destructor (crates/wasmedge-sys/src/instance/mod.rs:26). Any new accessor that
hands out a sub-object of an instance must follow this pattern, or it will double-free.

`AsInstance` (crates/wasmedge-sys/src/instance/module.rs:46) is the shared surface — implemented for `Instance`,
blanket-implemented for `AsRef<Instance> + AsMut<Instance>`, and refined by the SDK marker traits `SyncInst`
(src/vm.rs:9) and `AsyncInst` (src/async/vm.rs:12).

**Host functions.** `SyncFn<Data>` is
`fn(&mut Data, &mut Instance, &mut CallingFrame, Vec<WasmValue>) -> Result<Vec<WasmValue>, CoreError>`
(crates/wasmedge-sys/src/instance/function.rs:16). `ImportObjectBuilder::with_func::<Args, Rets>(name, f)`
derives the wasm `FuncType` from the **turbofish type parameters** via `WasmValTypeList`, not from `f`'s body
(src/import.rs:34) — a mismatch between the declared types and what the function actually reads/returns is a
runtime trap, not a compile error. Host data is boxed once into the import object and handed to every function
as `&mut Data`.

**Async.** Every async item is gated on `all(feature = "async", target_os = "linux")` (src/lib.rs:121,
crates/wasmedge-sys/src/lib.rs:67), so on macOS/Windows the modules simply disappear even though `async` is a
default feature of `wasmedge-sdk`. `async-wasi` implements the WASI preview_1 syscalls in Rust
(`snapshots/preview_1`); `AsyncWasiModule` (crates/wasmedge-sys/src/async/module.rs) binds each syscall to a
host function, and blocking calls suspend the wasm stack via fibers (`fiber-for-wasmedge`).

**Errors.** `WasmEdgeResult<T> = Result<T, Box<WasmEdgeError>>` from `wasmedge-types`; traps raised inside the C
runtime surface as `CoreError`.

## Conventions

- Conventional Commits; `CHANGELOG.md` is generated by git-cliff (`cliff.toml`). DCO sign-off (`git commit -s`)
  is required and PRs rebase onto `main` (CONTRIBUTING.md).
- The version compatibility table is duplicated in `README.md`, `src/lib.rs`, and
  `crates/wasmedge-sys/src/lib.rs` — update all three on a version bump.
- Releases are manual: `workflow_dispatch` on `.github/workflows/release-crates.yml`, dry-run first, publish
  only from `main` (crates.io trusted publishing / OIDC). A single `cargo publish -p … -p …` call publishes the
  whole release set and works out the inter-crate order itself, so it needs cargo ≥ 1.90; `resolve_release_set`
  asks crates.io which versions already exist and drops those from the set. Uploads are not atomic — if one
  fails partway, re-running the workflow to publish the remainder is safe.

## Gotchas

- `Cargo.lock` is gitignored and has never been tracked. CI's `--locked` steps only pass because an earlier
  unlocked step in the same job generates it; on a fresh clone `cargo test --locked` fails outright.
- MSRV 1.71 is documented in prose only (README + lib.rs docs) — no `rust-version` key in any manifest.
- All crates are edition 2021, but `.rustfmt.toml` deliberately pins `style_edition = "2015"`.
- `src/dock.rs` (~500 lines) is dead code — the module is commented out at src/lib.rs:103.
- `test-wasm/` is excluded from the workspace and targets `wasm32-unknown-unknown`; workspace-wide cargo
  commands do not touch it.
- `.github/workflows/ci-build.yml` is a scheduled canary against WasmEdge development HEAD (Mondays 03:00 UTC
  plus manual dispatch), not a PR gate — every other workflow builds against a released runtime.
