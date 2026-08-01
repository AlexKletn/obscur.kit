#![doc = include_str!("mod.doc/_module.md")]
mod window_backend;
pub use window_backend::{NullWindowBackend, WindowBackend, WindowError, WindowFeature};
