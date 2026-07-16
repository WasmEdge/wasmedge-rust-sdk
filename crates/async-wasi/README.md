# Overview

The [async-wasi](https://crates.io/crates/async-wasi) crate implements the [WASI](https://wasi.dev/) (WebAssembly System Interface) `wasi_snapshot_preview1` spec for asynchronous host environments, so that WASI syscalls made by a Wasm guest (file, clock, and networking access) can run without blocking the host's async runtime. It is used internally by [wasmedge-sys](https://crates.io/crates/wasmedge-sys) and [wasmedge-sdk](https://crates.io/crates/wasmedge-sdk) to back their async WASI support; most users will depend on one of those crates instead of this one directly.

* [async-wasi](https://crates.io/crates/async-wasi) requires **Rust v1.85 or above** in the **stable** channel.

## Features

* `async_tokio` (enabled by default) — implements the async socket and I/O primitives on top of [Tokio](https://tokio.rs/). This is currently the only supported async runtime backend.

## API Reference

* [async-wasi API Reference](https://docs.rs/async-wasi)

## See also

* [WasmEdge Runtime Official Website](https://wasmedge.org/)
* [WASI](https://wasi.dev/)
