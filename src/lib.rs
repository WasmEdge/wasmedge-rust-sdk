#![doc(
    html_logo_url = "https://github.com/cncf/artwork/blob/master/projects/wasm-edge-runtime/icon/color/wasm-edge-runtime-icon-color.png?raw=true",
    html_favicon_url = "https://raw.githubusercontent.com/cncf/artwork/49169bdbc88a7ce3c4a722c641cc2d548bd5c340/projects/wasm-edge-runtime/icon/color/wasm-edge-runtime-icon-color.svg"
)]
// If the version of rust used is less than v1.63, please uncomment the follow attribute.
// #![feature(explicit_generic_args_with_impl_trait)]
#![allow(clippy::vec_init_then_push)]
#![cfg_attr(docsrs, feature(doc_cfg))]

//! # Overview
//!
//! WasmEdge Rust SDK provides idiomatic [Rust](https://www.rust-lang.org/) language bindings for [WasmEdge](https://wasmedge.org/)
//!
//! **Notice:** This project is still under active development and not guaranteed to have a stable API.
//!
//! - [WasmEdge website](https://wasmedge.org/)
//! - [WasmEdge Docs](https://wasmedge.org/docs/)
//! - [WasmEdge GitHub Page](https://github.com/WasmEdge/WasmEdge)
//! - [WasmEdge Rust SDK GitHub Page](https://github.com/WasmEdge/wasmedge-rust-sdk)
//! - [WasmEdge Rust SDK Examples](https://github.com/second-state/wasmedge-rustsdk-examples)
//!
//! ## Get Started
//!
//! Since this crate depends on the WasmEdge C API, it needs to be installed in your system first. Please refer to [WasmEdge Installation and Uninstallation](https://wasmedge.org/book/en/quick_start/install.html) to install the WasmEdge library. The versioning table below shows the version of the WasmEdge library required by each version of the `wasmedge-sdk` crate.
//!
//! | wasmedge-sdk  | WasmEdge lib  | wasmedge-sys  | wasmedge-types| wasmedge-macro| async-wasi|
//! | :-----------: | :-----------: | :-----------: | :-----------: | :-----------: | :-------: |
//! | 0.11.2        | 0.13.3        | 0.16.2        | 0.4.3         | 0.6.1         | 0.1.0     |
//! | 0.11.0        | 0.13.3        | 0.16.0        | 0.4.3         | 0.6.0         | 0.0.3     |
//! | 0.10.1        | 0.13.3        | 0.15.1        | 0.4.2         | 0.5.0         | 0.0.2     |
//! | 0.10.0        | 0.13.2        | 0.15.0        | 0.4.2         | 0.5.0         | 0.0.2     |
//! | 0.9.0         | 0.13.1        | 0.14.0        | 0.4.2         | 0.4.0         | 0.0.1     |
//! | 0.9.0         | 0.13.0        | 0.14.0        | 0.4.2         | 0.4.0         | 0.0.1     |
//! | 0.8.1         | 0.12.1        | 0.13.1        | 0.4.1         | 0.3.0         | -         |
//! | 0.8.0         | 0.12.0        | 0.13.0        | 0.4.1         | 0.3.0         | -         |
//! | 0.7.1         | 0.11.2        | 0.12.2        | 0.3.1         | 0.3.0         | -         |
//! | 0.7.0         | 0.11.2        | 0.12          | 0.3.1         | 0.3.0         | -         |
//! | 0.6.0         | 0.11.2        | 0.11          | 0.3.0         | 0.2.0         | -         |
//! | 0.5.0         | 0.11.1        | 0.10          | 0.3.0         | 0.1.0         | -         |
//! | 0.4.0         | 0.11.0        | 0.9           | 0.2.1         | -             | -         |
//! | 0.3.0         | 0.10.1        | 0.8           | 0.2           | -             | -         |
//! | 0.1.0         | 0.10.0        | 0.7           | 0.1           | -             | -         |
//!
//!
//! WasmEdge Rust SDK will automatically search for the WasmEdge library in your system. Alternatively you can set the `WASMEDGE_DIR` environment variable to the path of the WasmEdge library (or the `WASMEDGE_INCLUDE_DIR` and `WASMEDGE_LIB_DIR` variables for more fine-grained control). If you want to use a local `cmake` build of WasmEdge you can set the `WASMEDGE_BUILD_DIR` instead.
//!
//! WasmEdge Rust SDK will search for the WasmEdge library in the following paths in order:
//!
//! - `$WASMEDGE_[INCLUDE|LIB]_DIR`
//! - `$WASMEDGE_DIR`
//! - `$WASMEDGE_BUILD_DIR`
//! - `$HOME/.wasmedge`
//! - `/usr/local`
//! - `$HOME/.local`
//!
//! When the `standalone` feature is enabled the correct library will be downloaded during build time and the previous locations are ignored. You can specify a proxy for the download process using the `WASMEDGE_STANDALONE_PROXY`, `WASMEDGE_STANDALONE_PROXY_USER` and `WASMEDGE_STANDALONE_PROXY_PASS` environment variables. You can set the `WASMEDGE_STANDALONE_ARCHIVE` environment variable to use a local archive instead of downloading one.
//! The following architectures are supported for automatic downloads:
//!
//! | os    | libc    | architecture        | linking type    |
//! | :---: | :-----: | :-----------------: | :-------------: |
//! | macos | -       | `x86_64`, `aarch64` | dynamic         |
//! | linux | `glibc` | `x86_64`, `aarch64` | static, dynamic |
//! | linux | `musl`  | `x86_64`, `aarch64` | static          |
//!
//! This crate uses `rust-bindgen` during the build process. If you would like to use an external `rust-bindgen` you can set the `WASMEDGE_RUST_BINDGEN_PATH` environment variable to the `bindgen` executable path. This is particularly useful in systems like Alpine Linux (see [rust-lang/rust-bindgen#2360](https://github.com/rust-lang/rust-bindgen/issues/2360#issuecomment-1595869379), [rust-lang/rust-bindgen#2333](https://github.com/rust-lang/rust-bindgen/issues/2333)).
//!
//! **Notice:** The minimum supported Rust version is 1.68.
//!
//! ## Examples
//!
//! The [Examples of WasmEdge RustSDK](https://github.com/second-state/wasmedge-rustsdk-examples) repo contains a number of examples that demonstrate how to use the WasmEdge Rust SDK.
//!
//! ## Contributing
//!
//! Please read the [contribution guidelines](https://github.com/WasmEdge/wasmedge-rust-sdk/blob/main/CONTRIBUTING.md) on how to contribute code.
//!
//! ## License
//!
//! This project is licensed under the terms of the [Apache 2.0 license](https://github.com/tensorflow/rust/blob/HEAD/LICENSE).
//!

#[cfg(all(feature = "async", target_os = "linux"))]
#[cfg_attr(docsrs, doc(cfg(all(feature = "async", target_os = "linux"))))]
pub mod r#async;
#[doc(hidden)]
pub mod caller;
#[doc(hidden)]
#[cfg(feature = "aot")]
#[cfg_attr(docsrs, doc(cfg(feature = "aot")))]
mod compiler;
pub mod config;
pub mod dock;
mod executor;
mod externals;
mod import;
mod instance;
#[doc(hidden)]
pub mod io;
#[doc(hidden)]
pub mod log;
mod module;
pub mod plugin;
mod statistics;
mod store;
pub mod types;
pub mod utils;
#[doc(hidden)]
pub mod vm;
#[cfg(not(feature = "async"))]
pub mod wasi;

pub use caller::Caller;
#[doc(inline)]
#[cfg(feature = "aot")]
#[cfg_attr(docsrs, doc(cfg(feature = "aot")))]
pub use compiler::Compiler;
#[doc(inline)]
pub use executor::Executor;
#[doc(inline)]
pub use externals::{Func, FuncRef, FuncTypeBuilder, Global, Memory, Table};
#[doc(inline)]
pub use import::{ImportObject, ImportObjectBuilder};
pub use instance::{AsInstance, Instance};
#[doc(inline)]
pub use io::{WasmVal, WasmValType, WasmValTypeList};
#[doc(inline)]
pub use log::LogManager;
#[doc(inline)]
pub use module::{ExportType, ImportType, Module};
#[doc(inline)]
pub use statistics::Statistics;
#[doc(inline)]
pub use store::Store;
#[doc(inline)]
pub use vm::{Vm, VmBuilder};

pub use wasmedge_types::{
    error, wat2wasm, CompilerOptimizationLevel, CompilerOutputFormat, ExternalInstanceType,
    FuncType, GlobalType, HostRegistration, MemoryType, Mutability, RefType, TableType, ValType,
    WasmEdgeResult,
};

#[cfg(all(feature = "async", target_os = "linux"))]
#[cfg_attr(docsrs, doc(cfg(all(feature = "async", target_os = "linux"))))]
pub use wasmedge_macro::async_host_function;
pub use wasmedge_macro::host_function;

/// WebAssembly value type.
pub type WasmValue = wasmedge_sys::types::WasmValue;

/// This is a workaround solution to the [`never`](https://doc.rust-lang.org/std/primitive.never.html) type in Rust. It will be replaced by `!` once it is stable.
pub type NeverType = wasmedge_types::NeverType;

#[doc(hidden)]
pub type CallingFrame = wasmedge_sys::CallingFrame;

/// The object that is used to perform a [host function](crate::Func) is required to implement this trait.
pub trait Engine {
    /// Runs a host function instance and returns the results.
    ///
    /// # Arguments
    ///
    /// * `func` - The function instance to run.
    ///
    /// * `params` - The arguments to pass to the function.
    ///
    /// # Errors
    ///
    /// If fail to run the host function, then an error is returned.
    fn run_func(
        &self,
        func: &Func,
        params: impl IntoIterator<Item = WasmValue>,
    ) -> WasmEdgeResult<Vec<WasmValue>>;

    /// Runs a host function instance by calling its reference and returns the results.
    ///
    /// # Arguments
    ///
    /// * `func_ref` - A reference to the target host function instance.
    ///
    /// * `params` - The arguments to pass to the function.
    ///
    /// # Errors
    ///
    /// If fail to run the host function, then an error is returned.
    fn run_func_ref(
        &self,
        func_ref: &FuncRef,
        params: impl IntoIterator<Item = WasmValue>,
    ) -> WasmEdgeResult<Vec<WasmValue>>;
}
