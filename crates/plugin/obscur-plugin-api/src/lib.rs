//! Stable semantic Host/Plugin API boundary.
#![forbid(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]

pub mod executor;
pub mod handle;
pub mod host;
pub mod plugin;
pub mod service;
pub mod status;
