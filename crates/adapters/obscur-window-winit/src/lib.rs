//! Winit `WindowBackend` adapter.
#![forbid(unsafe_code)]

pub mod backend;
pub mod conversion;
pub mod error;
pub mod wake;

#[cfg(all(feature = "gamepad", target_os = "linux"))]
pub mod gamepad;
