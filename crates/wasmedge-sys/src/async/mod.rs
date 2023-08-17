//! Defines `async` related types.

pub mod fiber;
pub(crate) mod function;
pub mod module;

pub use module::AsyncWasiModule;
