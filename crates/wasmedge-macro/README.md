# Overview

The [wasmedge-macro](https://crates.io/crates/wasmedge-macro) crate defines a group of procedural macros used by both [wasmedge-sdk](https://crates.io/crates/wasmedge-sdk) and [wasmedge-sys](https://crates.io/crates/wasmedge-sys) crates.

## Deprecation notice: `host_function` / `async_host_function` / `sys_*_host_function`

All six procedural macros currently defined by this crate — the public `host_function`
and `async_host_function`, plus the `#[doc(hidden)]` `sys_host_function`,
`sys_async_host_function`, `sys_wasi_host_function`, and `sys_async_wasi_host_function` —
expand to the **pre-0.14** host-function ABI (a 3-argument free function built around a
`Caller` type and a `HostFuncError` return type). That ABI was removed when
`wasmedge-sdk` reached 0.14 and `wasmedge-sys` reached 0.19: `Caller` no longer exists,
and while `HostFuncError` still exists in `wasmedge_types::error`, it is unrelated to
and unused by the current host-function signature, which uses `CoreError` instead.
**Code using any of these macros today does not compile** against
current `wasmedge-sdk` (>= 0.14) or `wasmedge-sys` (>= 0.19).

These macros are marked `#[deprecated]` in `wasmedge-macro` **0.7.0**.
No usage of any of them is known to exist in the wild, so the deprecation (and the
`wasmedge-sdk` re-export removal that came with it) carries no measurable downstream
breakage. See the doc comment on each macro in [`src/lib.rs`](src/lib.rs) for the full
rationale and the modern replacement: write the host function directly with the current
signature
(`fn(&mut Data, &mut Instance, &mut CallingFrame, Vec<WasmValue>) -> Result<Vec<WasmValue>, CoreError>`)
and register it with `ImportObjectBuilder::with_func` — no macro required.

See also

* [WasmEdge Runtime](https://wasmedge.org/)
