#![doc(
    html_logo_url = "https://github.com/cncf/artwork/blob/master/projects/wasm-edge-runtime/icon/color/wasm-edge-runtime-icon-color.png?raw=true",
    html_favicon_url = "https://raw.githubusercontent.com/cncf/artwork/49169bdbc88a7ce3c4a722c641cc2d548bd5c340/projects/wasm-edge-runtime/icon/color/wasm-edge-runtime-icon-color.svg"
)]

//! # Overview
//! The [wasmedge-sys](https://crates.io/crates/wasmedge-sys) crate defines a group of low-level Rust APIs for WasmEdge, a light-weight, high-performance, and extensible WebAssembly runtime for cloud-native, edge, and decentralized applications.
//!
//! For developers, it is strongly recommended that the APIs in `wasmedge-sys` are used to construct high-level libraries, while `wasmedge-sdk` is for building up business applications.
//!
//! * Notice that [wasmedge-sys](https://crates.io/crates/wasmedge-sys) requires **Rust v1.69 or above** in the **stable** channel.
//!

//! ## Build
//!
//! To use or build the `wasmedge-sys` crate, the `WasmEdge` library is required. Please refer to [WasmEdge Installation and Uninstallation](https://wasmedge.org/book/en/quick_start/install.html) to install the `WasmEdge` library.
//!
//! * The following table provides the versioning information about each crate of WasmEdge Rust bindings.
//!
//!   | wasmedge-sdk  | WasmEdge lib  | wasmedge-sys  | wasmedge-types| wasmedge-macro| async-wasi|
//!   | :-----------: | :-----------: | :-----------: | :-----------: | :-----------: | :-------: |
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
//!
//!

//! ## See also
//!
//! * [WasmEdge Runtime Official Website](https://wasmedge.org/)
//! * [WasmEdge Docs](https://wasmedge.org/book/en/)
//! * [WasmEdge C API Documentation](https://github.com/WasmEdge/WasmEdge/blob/master/docs/c_api.md)

#![deny(rust_2018_idioms, unreachable_pub)]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[macro_use]
extern crate lazy_static;

use parking_lot::{Mutex, RwLock};
use std::{collections::HashMap, sync::Arc};

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
#[cfg(not(feature = "async"))]
pub use instance::module::WasiModule;
#[doc(inline)]
pub use instance::{
    function::{FuncRef, FuncType, Function},
    global::{Global, GlobalType},
    memory::{MemType, Memory},
    module::{AsImport, AsInstance, ImportModule, Instance, WasiInstance},
    table::{Table, TableType},
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
use wasmedge_types::{error::HostFuncError, WasmEdgeResult};

/// Type of wasi context that is used to configure the wasi environment.
#[cfg(all(feature = "async", target_os = "linux"))]
#[cfg_attr(docsrs, doc(cfg(all(feature = "async", target_os = "linux"))))]
pub type WasiCtx = ::async_wasi::snapshots::WasiCtx;

pub(crate) type BoxedFn = Box<
    dyn Fn(
            CallingFrame,
            Vec<WasmValue>,
            *mut std::os::raw::c_void,
        ) -> Result<Vec<WasmValue>, HostFuncError>
        + Send
        + Sync,
>;

lazy_static! {
    pub(crate) static ref HOST_FUNCS: RwLock<HashMap<usize, Arc<Mutex<BoxedFn>>>> =
        RwLock::new(HashMap::new());
}

/// Type alias for a boxed native function. This type is used in thread-safe cases.
pub(crate) type BoxedAsyncFn = Box<
    dyn Fn(
            CallingFrame,
            Vec<WasmValue>,
            *mut std::os::raw::c_void,
        )
            -> Box<dyn std::future::Future<Output = Result<Vec<WasmValue>, HostFuncError>> + Send>
        + Send
        + Sync,
>;

lazy_static! {
    pub(crate) static ref ASYNC_HOST_FUNCS: RwLock<HashMap<usize, Arc<Mutex<BoxedAsyncFn>>>> =
        RwLock::new(HashMap::new());
}

// Stores the mapping from the address of each host function pointer to the key of the `HOST_FUNCS`.
lazy_static! {
    pub(crate) static ref HOST_FUNC_FOOTPRINTS: Mutex<HashMap<usize, usize>> =
        Mutex::new(HashMap::new());
}

/// The object that is used to perform a [host function](crate::Function) is required to implement this trait.
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
        func: &Function,
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
