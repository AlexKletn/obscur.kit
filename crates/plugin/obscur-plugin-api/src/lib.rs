//! Stable semantic Host/Plugin API boundary.
#![forbid(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]

/// Running plugin code.
pub mod executor;
/// Opaque references to host-owned objects.
pub mod handle;
/// The host side of the API, as a plugin sees it.
pub mod host;
/// A plugin as the API models it.
pub mod plugin;
/// Services exchanged across the boundary.
pub mod service;
/// The status a call or a plugin reports.
pub mod status;
