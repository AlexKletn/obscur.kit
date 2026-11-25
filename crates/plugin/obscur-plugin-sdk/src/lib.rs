//! Ergonomic Rust wrappers over the Plugin API/ABI.
#![forbid(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]

/// Bindings and export glue for the native plugin lifecycle.
pub mod native;
/// Registering a plugin's implementations.
pub mod registration;
/// Generated wrappers around plugin entry points.
pub mod wrapper;
