# Upgrade to 0.17.0

This document summarizes what changed between the last version actually published
to crates.io (`wasmedge-sdk` 0.14.0, paired with `wasmedge-sys` 0.19.4) and
`wasmedge-sdk` 0.17.0 (paired with `wasmedge-sys` 0.21.0, `wasmedge-types` 0.7.0,
`wasmedge-macro` 0.7.0, and `async-wasi` 0.3.0 — all still built against WasmEdge
C API 0.17.1). See [CHANGELOG.md](../CHANGELOG.md) for the full, itemized list this
document summarizes.

**Unlike the 0.13.x → 0.14.0 upgrade** (see
[Upgrade_to_0.14.0.md](Upgrade_to_0.14.0.md), which changed several function
signatures), **this release has no public API shape changes.** If your code
compiles against `wasmedge-sdk` 0.14.0, it compiles against 0.17.0 without
modification. This was machine-checked throughout development with
`cargo semver-checks` against every crate's last-published baseline — see
"Verification" below. The two things worth reading before upgrading are the
MSRV/edition bump and the one observable (but non-breaking-in-shape) error
value change, both below.

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

## Coming in a future release (not in 0.17.0)

The following nominally-breaking cleanups are **planned for a future 0.21.0
(`wasmedge-sys`) / 0.7.0 (`wasmedge-macro`) release**, not this one. They're
listed here so downstream users can see them coming; none of them are in
0.17.0/0.21.0 as shipped by this upgrade guide's own release.

- **`wasmedge-sys::io` removal.** The `WasmFnIO` trait and `I1`..`I32` marker
  types in `wasmedge_sys::io` (170 lines) have zero references anywhere in
  this workspace. A crates.io-wide search found exactly one real dependent
  (`wasmedge-bindgen-host`, pinned to `^0.7.0` of `wasmedge-sys`, so
  unaffected by a change past that range) and no other public usage. Planned
  for removal alongside the `wasmedge-sys` 0.21.0 release.
- **`Memory::mut_slice` receiver change.** `wasmedge_sys::Memory::mut_slice`
  currently has the signature `fn mut_slice<T>(&self) -> Option<&mut [T]>` —
  a safe function hand-out of an aliasable `&mut` from a shared `&self`,
  which is unsound. Planned fix: change the receiver to `&mut self` (matching
  the sibling `get_ref_mut`), which is a receiver-only change with no
  behavior difference for any caller that already needs mutable access.
  `wasmedge-sdk` doesn't call this method, and a crates.io-wide search found
  no external callers either.
- **`host_function`/`async_host_function` macro deprecation.** These six
  procedural macros currently expand to a pre-0.14 function shape
  (referencing a `Caller` type that hasn't existed in this crate since the
  0.14.0 API redesign), so **any use of them today already fails to
  compile** — they have had zero working callers since 0.14.0 shipped.
  Planned: mark them `#[deprecated]` and remove `wasmedge-sdk`'s re-export of
  `host_function`/`async_host_function`, once the version bump that makes
  removing that re-export non-breaking-in-practice lands.

## Verification

Every commit that landed the changes summarized above was gated on, in
addition to the normal build/test/clippy/fmt suite:

```bash
cargo semver-checks check-release -p <crate> --baseline-version <last-published-version>
# wasmedge-sys / wasmedge-sdk additionally need:
cargo semver-checks check-release -p wasmedge-sys --baseline-version 0.19.4 --features standalone
cargo semver-checks check-release -p wasmedge-sdk --baseline-version 0.14.0 --features standalone
```

run against each crate's last-published-to-crates.io baseline
(`wasmedge-sdk` 0.14.0 / `wasmedge-sys` 0.19.4 / `wasmedge-types` 0.6.0 /
`async-wasi` 0.2.1). `wasmedge-macro` is a proc-macro crate, which
`cargo-semver-checks` cannot analyze by nature.
