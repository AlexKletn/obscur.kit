//! Ergonomic Rust wrappers over the Plugin API/ABI.
#![forbid(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]

/// Registering a plugin's implementations.
pub mod registration;
/// Generated wrappers around plugin entry points.
pub mod wrapper;
