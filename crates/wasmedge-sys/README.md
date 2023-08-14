# Overview

The [wasmedge-sys](https://crates.io/crates/wasmedge-sys) crate defines a group of low-level Rust APIs for WasmEdge, a light-weight, high-performance, and extensible WebAssembly runtime for cloud-native, edge, and decentralized applications.

For developers, it is recommended that the APIs in `wasmedge-sys` are used to construct high-level libraries, while `wasmedge-sdk` is for building up business applications.

* Notice that [wasmedge-sys](https://crates.io/crates/wasmedge-sys) requires **Rust v1.69 or above** in the **stable** channel.

## Build

This crate depends on the WasmEdge C API. In linux/macOS the crate can download the API at build time by enabling the `standalone` feature. Otherwise the API needs to be installed in your system first. Please refer to [Get Started](https://github.com/WasmEdge/wasmedge-rust-sdk#get-started) for more information.

## See also

* [WasmEdge Runtime Official Website](https://wasmedge.org/)
* [WasmEdge Docs](https://wasmedge.org/book/en/)
* [WasmEdge C API Documentation](https://github.com/WasmEdge/WasmEdge/blob/master/docs/c_api.md)
