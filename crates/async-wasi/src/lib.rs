#![doc(
    html_logo_url = "https://github.com/cncf/artwork/blob/master/projects/wasm-edge-runtime/icon/color/wasm-edge-runtime-icon-color.png?raw=true",
    html_favicon_url = "https://raw.githubusercontent.com/cncf/artwork/49169bdbc88a7ce3c4a722c641cc2d548bd5c340/projects/wasm-edge-runtime/icon/color/wasm-edge-runtime-icon-color.svg"
)]

//! The [async-wasi](https://crates.io/crates/async-wasi) crate implements WASI spec for the asynchronous scenarios.
//!
//! See also
//!
//! * [WasmEdge Runtime](https://wasmedge.org/)
//!

#[allow(clippy::too_many_arguments)]
pub mod snapshots;

pub use snapshots::WasiCtx;
