#![doc(
    html_logo_url = "https://github.com/cncf/artwork/blob/master/projects/wasm-edge-runtime/icon/color/wasm-edge-runtime-icon-color.png?raw=true",
    html_favicon_url = "https://raw.githubusercontent.com/cncf/artwork/49169bdbc88a7ce3c4a722c641cc2d548bd5c340/projects/wasm-edge-runtime/icon/color/wasm-edge-runtime-icon-color.svg"
)]

//! # Overview
//! The [wasmedge-sys](https://crates.io/crates/wasmedge-sys) crate defines a group of low-level Rust APIs for WasmEdge, a light-weight, high-performance, and extensible WebAssembly runtime for cloud-native, edge, and decentralized applications.
//!
//! For developers, it is strongly recommended that the APIs in `wasmedge-sys` are used to construct high-level libraries, while `wasmedge-sdk` is for building up business applications.
//!
//! * Notice that [wasmedge-sys](https://crates.io/crates/wasmedge-sys) requires **Rust v1.71 or above** in the **stable** channel.
//!

//! ## Build
//!
//! To use or build the `wasmedge-sys` crate, the `WasmEdge` library is required. Please refer to [WasmEdge Installation and Uninstallation](https://wasmedge.org/book/en/quick_start/install.html) to install the `WasmEdge` library.
//!
//! * The following table provides the versioning information about each crate of WasmEdge Rust bindings.
//!
//!   | wasmedge-sdk  | WasmEdge lib  | wasmedge-sys  | wasmedge-types| wasmedge-macro| async-wasi|
//!   | :-----------: | :-----------: | :-----------: | :-----------: | :-----------: | :-------: |
//!   | 0.14.0+       | 0.14.1        | 0.19.3        | 0.6.0         | 0.6.1         | 0.2.1     |
//!   | 0.14.0+       | 0.14.0        | 0.19.2        | 0.6.0         | 0.6.1         | 0.2.1     |
//!   | 0.13.2        | 0.13.5        | 0.17.5        | 0.4.4         | 0.6.1         | 0.1.0     |
//!   | 0.13.1        | 0.13.5        | 0.17.4        | 0.4.4         | 0.6.1         | 0.1.0     |
//!   | 0.13.0        | 0.13.5        | 0.17.3        | 0.4.4         | 0.6.1         | 0.1.0     |
//!   | 0.12.2        | 0.13.4        | 0.17.2        | 0.4.4         | 0.6.1         | 0.1.0     |
//!   | 0.12.1        | 0.13.4        | 0.17.1        | 0.4.4         | 0.6.1         | 0.1.0     |
//!   | 0.12.0        | 0.13.4        | 0.17.0        | 0.4.4         | 0.6.1         | 0.1.0     |
//!   | 0.11.2        | 0.13.3        | 0.16.2        | 0.4.3         | 0.6.1         | 0.1.0     |
//!   | 0.11.0        | 0.13.3        | 0.16.0        | 0.4.3         | 0.6.0         | 0.0.3     |
//!   | 0.10.1        | 0.13.3        | 0.15.1        | 0.4.2         | 0.5.0         | 0.0.2     |
//!   | 0.10.0        | 0.13.2        | 0.15.0        | 0.4.2         | 0.5.0         | 0.0.2     |
//!   | 0.9.0         | 0.13.1        | 0.14.0        | 0.4.2         | 0.4.0         | 0.0.1     |
//!   | 0.9.0         | 0.13.0        | 0.14.0        | 0.4.2         | 0.4.0         | 0.0.1     |
//!   | 0.8.1         | 0.12.1        | 0.13.1        | 0.4.1         | 0.3.0         | -         |
//!   | 0.8.0         | 0.12.0        | 0.13.0        | 0.4.1         | 0.3.0         | -         |
//!   | 0.7.1         | 0.11.2        | 0.12.2        | 0.3.1         | 0.3.0         | -         |
//!   | 0.7.0         | 0.11.2        | 0.12          | 0.3.1         | 0.3.0         | -         |
//!   | 0.6.0         | 0.11.2        | 0.11          | 0.3.0         | 0.2.0         | -         |
//!   | 0.5.0         | 0.11.1        | 0.10          | 0.3.0         | 0.1.0         | -         |
//!   | 0.4.0         | 0.11.0        | 0.9           | 0.2.1         | -             | -         |
//!   | 0.3.0         | 0.10.1        | 0.8           | 0.2           | -             | -         |
//!   | 0.1.0         | 0.10.0        | 0.7           | 0.1           | -             | -         |
//!
//! ## API Reference
//!
//! * [wasmedge-sys API Reference](https://wasmedge.github.io/wasmedge-rust-sdk/wasmedge_sys/index.html)
//! * [wasmedge-sys Async API Reference](https://second-state.github.io/wasmedge-async-rust-sdk/wasmedge_sys/index.html)
//!
//! ## See also
//!
//! * [WasmEdge Runtime Official Website](https://wasmedge.org/)
//! * [WasmEdge Docs](https://wasmedge.org/book/en/)
//! * [WasmEdge C API Documentation](https://github.com/WasmEdge/WasmEdge/blob/master/docs/c_api.md)

#![deny(rust_2018_idioms, unreachable_pub)]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[allow(warnings)]
/// Foreign function interfaces generated from WasmEdge C-API.
pub mod ffi {
    include!(concat!(env!("OUT_DIR"), "/wasmedge.rs"));
}
#[doc(hidden)]
pub mod ast_module;
#[cfg(all(feature = "async", target_os = "linux"))]
#[cfg_attr(docsrs, doc(cfg(all(feature = "async", target_os = "linux"))))]
pub mod r#async;
#[doc(hidden)]
#[cfg(feature = "aot")]
pub mod compiler;
#[doc(hidden)]
pub mod config;
#[doc(hidden)]
pub mod executor;
pub mod frame;
pub mod instance;
#[doc(hidden)]
pub mod io;
#[doc(hidden)]
pub mod loader;
pub mod plugin;
#[doc(hidden)]
pub mod statistics;
#[doc(hidden)]
pub mod store;
pub mod types;
pub mod utils;
#[doc(hidden)]
pub mod validator;

#[doc(inline)]
pub use ast_module::{ExportType, ImportType, Module};
#[doc(inline)]
#[cfg(feature = "aot")]
#[cfg_attr(docsrs, doc(cfg(feature = "aot")))]
pub use compiler::Compiler;
#[doc(inline)]
pub use config::Config;
#[doc(inline)]
pub use executor::Executor;
#[doc(inline)]
pub use frame::CallingFrame;
#[doc(inline)]
pub use instance::module::WasiModule;
#[doc(inline)]
pub use instance::{
    function::{FuncRef, Function, SyncFn},
    global::Global,
    memory::Memory,
    module::{AsInstance, ImportModule, Instance},
    table::Table,
    FuncType, GlobalType, MemoryType, TableType,
};
#[doc(inline)]
pub use loader::Loader;
#[doc(inline)]
pub use statistics::Statistics;
#[doc(inline)]
pub use store::Store;
#[doc(inline)]
pub use types::WasmValue;
#[doc(inline)]
pub use validator::Validator;

use wasmedge_types::WasmEdgeResult;

/// Type of wasi context that is used to configure the wasi environment.
#[cfg(all(feature = "async", target_os = "linux"))]
#[cfg_attr(docsrs, doc(cfg(all(feature = "async", target_os = "linux"))))]
pub type WasiCtx = ::async_wasi::snapshots::WasiCtx;
