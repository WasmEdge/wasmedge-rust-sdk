# Upgrade to 0.17.0

This document summarizes what changed between the last version actually published
to crates.io (`wasmedge-sdk` 0.14.0, paired with `wasmedge-sys` 0.19.4) and
`wasmedge-sdk` 0.17.0 (paired with `wasmedge-sys` 0.21.0, `wasmedge-types` 0.7.0,
`wasmedge-macro` 0.7.0, and `async-wasi` 0.3.0 — all still built against WasmEdge
C API 0.17.1). See [CHANGELOG.md](../CHANGELOG.md) for the full, itemized list this
document summarizes.

**Unlike the 0.13.x → 0.14.0 upgrade** (see
[Upgrade_to_0.14.0.md](Upgrade_to_0.14.0.md), which changed several function
signatures), **this release keeps the public API essentially stable.** It makes
only a small, well-scoped set of nominally-breaking changes — see [Breaking
changes in 0.17.0](#breaking-changes-in-0170) below — none of which affect code
that compiles against `wasmedge-sdk` 0.14.0 today, because each targets an item
that was already dead, `#[doc(hidden)]`, non-compiling, or, in the case of
`Memory::mut_slice`, an unsound signature with no discovered callers. Everything
else is source-compatible. The public surface was tracked throughout development with
`cargo semver-checks` against every crate's last-published crates.io baseline —
see "Verification" below. Beyond the breaking set, the two things worth reading
before upgrading are the MSRV/edition bump and the one observable (but
non-breaking-in-shape) error value change, both below.

## MSRV and edition

| | 0.14.0 era | 0.17.0 |
|---|---|---|
| MSRV | 1.71 (documented in lib.rs, never enforced) | **1.85**, enforced via `rust-version` in every crate's `Cargo.toml` |
| Edition | 2021 | **2024**, across all five crates |

If your toolchain is older than 1.85 you'll need to upgrade it; 1.85 is also the
first Rust release with `edition = "2024"` support. Cargo's own MSRV-aware
resolver (`resolver = "3"`, enabled workspace-wide) means a dependency graph
that requires Rust ≥1.85 will not silently get selected for a consumer on an
older toolchain, as long as *their* project also opts into resolver v3 (or a
recent-enough Cargo version that infers it).

If you use the `WASMEDGE_RUST_BINDGEN_PATH` environment variable to point at an
external `bindgen` executable instead of the vendored `bindgen` crate, it must
now be **bindgen-cli 0.71 or newer**: the build script asks it to emit
edition-2024-shaped bindings via `--rust-edition 2024`, a flag older releases
don't recognize.

## Behavior fixes

These are internal correctness fixes uncovered during the edition/dependency
modernization work. None of them change a function's signature. Most fix a
guest-triggerable panic or an internal memory-safety bug that never had an
externally visible effect (or the SDK never hit the code path); one changes an
`Err` value you might match on (called out separately below).

### `async-wasi`

- `fd_prestat_dir_name` could read past the end of its buffer using a
  guest-controlled length, panicking instead of returning an error.
- `path_unlink_file` checked the `PATH_REMOVE_DIRECTORY` rights bit instead of
  `PATH_UNLINK_FILE`.
- `path_open` returned `EEXIST` instead of `NOENT` for a missing path when
  `O_CREAT` was not set.
- Socket timeouts (`tv_usec`) were interpreted as nanoseconds instead of
  microseconds, so any timeout at or above roughly one second would panic
  while constructing the `Duration`.
- `sock_getaddrinfo` wrote into a fixed-size `sa_data` buffer without checking
  its length — a guest-triggerable panic.
- Socket registration briefly faked ownership of a `Socket` via
  `mem::zeroed()` during an internal swap; replaced with a proper
  `Option`-based state machine.
- `SocketWritable::poll` didn't register a waker when returning `Pending`,
  relying entirely on an outer 10-second timeout and silently dropping its
  `Result`; it now uses `tokio::sync::Notify` for real wakeups.
- As a side effect of the `socket2` 0.6 upgrade (see CHANGELOG), `async-wasi`
  — and therefore the whole workspace — now **compiles natively on macOS**,
  which previously failed with an `E0599` from `socket2` 0.4.10's API.

### `wasmedge-sys`

- `Statistics` double-freed its inner context when cloned, and `Executor`
  could be left holding a dangling statistics pointer after `Statistics` was
  dropped. Fixed by moving the `Drop` impl to the reference-counted inner
  type.
- Several FFI handle newtypes (`InnerFunc`, `InnerModule`: `Copy`;
  `InnerInstance`, `InnerExecutor`: `Clone`) allowed an internal `.clone()` to
  silently double-free the underlying WasmEdge context. The derives are gone;
  everywhere that cloned them now borrows instead.
- `Loader::from_bytes` and `Compiler::compile_from_bytes` leaked their
  intermediate buffer on the error path and mishandled a `malloc(0)` edge
  case.
- `ImportModule::create` didn't check whether the underlying context pointer
  was null before using it.
- `ImportModule::from_raw` double-freed a borrowed module name.
- `Executor::call_func_ref` panicked via `.unwrap()` on a function-type
  mismatch; it now returns `FuncError::Type`.
- `box_future`'s async dispatch relied on every closure it boxes being
  zero-sized (via `mem::zeroed::<F>()`) without verifying it; it's now guarded
  by a `const { assert!(size_of::<F>() == 0) }` that fails to compile instead
  of silently miscompiling if that assumption is ever violated.

### `wasmedge-types`

- `GlobalError::UnmatchedValType`'s `Display` implementation was an empty
  `#[error("")]`, so `.to_string()` on that error produced an empty string.
  It now has a real message.

## The one observable error-value change

`wasmedge_sys::Validator::create` used to construct its `Err` case with the
`CompilerCreate` error variant when the underlying WasmEdge validator context
failed to create — a copy/paste mistake, since validator creation has nothing
to do with the AOT compiler. It now returns the `ValidatorCreate` variant,
which actually describes the failure.

- **What doesn't change:** the function's signature
  (`fn create(config: Option<&Config>) -> WasmEdgeResult<Self>`), and the fact
  that it returns `Err` in this situation.
- **What changes:** if you `match` on the specific error variant (rather than
  propagating the `Err` with `?` or matching only on the outer `WasmEdgeError`
  variant), you will now see `ValidatorCreate` where you used to see
  `CompilerCreate`.
- **Why this isn't flagged as a breaking change:** `cargo semver-checks`
  verifies type and function *shapes*, not which enum variant a function
  happens to construct at runtime — that's not something static analysis of
  the crate's public signatures can see. We're calling it out here explicitly
  because it's the one place where "no API changes" needs an asterisk, and
  because a caller matching on `CompilerCreate` in this specific path (however
  unlikely, given it was always the wrong variant) would need to update that
  match arm.

## Breaking changes in 0.17.0

This release rolls three long-planned, nominally-breaking cleanups into the
major-version bump (`wasmedge-sys` 0.21.0 / `wasmedge-macro` 0.7.0 /
`wasmedge-sdk` 0.17.0). Each is "nominal" because no code that actually compiles
against the last published release depends on the removed or changed item.

- **`host_function` / `async_host_function` macro deprecation + re-export
  removal.** All six procedural macros in `wasmedge-macro` — the public
  `host_function`/`async_host_function` and the four `#[doc(hidden)]` `sys_*`
  variants — are now `#[deprecated(since = "0.7.0")]`, and `wasmedge-sdk` no
  longer re-exports `host_function`/`async_host_function`. They expand to the
  pre-0.14 function shape (referencing a `Caller` type that hasn't existed since
  the 0.14.0 API redesign), so **any use of them already failed to compile** —
  zero working callers since 0.14.0. Write the host function directly with
  today's signature and register it with `ImportObjectBuilder::with_func` (or, at
  the `wasmedge-sys` layer, `Function::create_sync_func` + `ImportModule::add_func`).
  `cargo-semver-checks` cannot analyze proc-macros, so this produces no automated
  finding; it is called out here instead.
- **`wasmedge_sys::io` removal.** The `WasmFnIO` trait and the `I1`..`I32` marker
  types (~170 LOC) are gone. The module was `#[doc(hidden)]` and had zero
  references anywhere in this workspace; a crates.io-wide search found exactly one
  real dependent (`wasmedge-bindgen-host`, pinned to `^0.7.0` of `wasmedge-sys`,
  so unaffected past that range). Being `#[doc(hidden)]`, its removal likewise
  produces no `cargo-semver-checks` finding.
- **`Memory::mut_slice` receiver change.** `wasmedge_sys::Memory::mut_slice` now
  takes `&mut self` instead of `&self` (matching the sibling `get_ref_mut`).
  Handing out an aliasable `&mut [T]` from a shared `&self` was unsound.
  `wasmedge-sdk` does not call this method, no in-workspace caller passed a shared
  borrow, and a crates.io-wide search found no external callers. This is the one
  change `cargo-semver-checks` does flag — a `method_receiver_ref_became_mut`
  finding against the `wasmedge-sys` 0.19.4 baseline.

## The FFI surface tracks the WasmEdge C API

Between the last published `wasmedge-sys` (0.19.4, built against WasmEdge C API
0.14.1) and this release (0.21.0, built against WasmEdge C API 0.17.1), the raw
bindgen-generated symbols under `wasmedge_sys::ffi` changed to match the newer C
API. `cargo-semver-checks` reports three such deltas against the 0.19.4 baseline
— none introduced by this release's own Rust code, all a consequence of the
runtime upgrade already shipped in this train:

- `WasmEdge_ErrCode_InvalidStoreAlignment` was renamed to
  `WasmEdge_ErrCode_InvalidAlignment`.
- `WasmEdge_TypeCode_String` was removed when the type-code set was reworked for
  the 0.17 type system.
- the `WasmEdge_Limit` struct became an opaque `WasmEdge_LimitContext` with
  accessor functions (`WasmEdge_LimitCreate`, `WasmEdge_LimitGetMin`, ...).

Users of the safe wrappers are unaffected — most code goes through those. If you
use `wasmedge_sys::ffi` symbols directly, check them against the WasmEdge 0.17.1
C API headers.

## Verification

Throughout development each crate's public surface was tracked with
`cargo-semver-checks` against its last-published-to-crates.io baseline
(`wasmedge-sdk` 0.14.0 / `wasmedge-sys` 0.19.4 / `wasmedge-types` 0.6.0 /
`async-wasi` 0.2.1):

```bash
cargo semver-checks check-release -p wasmedge-types --baseline-version 0.6.0
cargo semver-checks check-release -p async-wasi     --baseline-version 0.2.1
# wasmedge-sys / wasmedge-sdk link against libwasmedge, so add the standalone
# feature. On a host whose target the 0.19.4 / 0.14.0 baseline build script does
# not support under the full feature union (e.g. macOS arm64, which those old
# releases can't build with static), isolate the feature with
# --only-explicit-features:
cargo semver-checks check-release -p wasmedge-sys --baseline-version 0.19.4 --only-explicit-features --features standalone
cargo semver-checks check-release -p wasmedge-sdk --baseline-version 0.14.0 --only-explicit-features --features standalone
```

Because the version bumps are 0.x-major (0.6→0.7, 0.19→0.21, 0.14→0.17),
`cargo-semver-checks` runs in breaking-allowed mode and only reports.
`wasmedge-types`, `async-wasi`, and `wasmedge-sdk` report **no** breaking
findings against their baselines; `wasmedge-sys` reports exactly the intentional
`Memory::mut_slice` receiver change plus the three `wasmedge_sys::ffi` C-API
symbol deltas noted above, and nothing else. `wasmedge-macro` is a proc-macro
crate, which `cargo-semver-checks` cannot analyze, so its macro deprecation and
the `wasmedge-sdk` re-export removal are verified by inspection.
