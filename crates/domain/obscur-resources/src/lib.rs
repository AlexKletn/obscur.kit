//! Resource Hub, typed handles and loader ports (skeleton).
#![forbid(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]

pub mod handle;

pub use handle::{
    FontHandle, ImageHandle, ResourceHandle, ResourceKind, ShaderHandle, StylesheetHandle,
};

pub mod cache;
pub mod hub;
pub mod port;
pub mod source;
pub mod state;
