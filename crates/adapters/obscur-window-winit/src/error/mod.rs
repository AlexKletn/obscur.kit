//! The winit backend's error type (spec §6.1 windowing; §2.7). Maps adapter-side
//! winit failures into the domain's `WindowError`.

mod winit_backend_error;
pub use winit_backend_error::WinitBackendError;
