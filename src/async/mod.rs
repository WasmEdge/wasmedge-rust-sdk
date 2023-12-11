pub mod import;
pub mod vm;

pub use wasmedge_sys::r#async::module::AsyncInstance;
pub mod wasi {
    pub use async_wasi;
    pub use wasmedge_sys::r#async::AsyncWasiModule;
}
