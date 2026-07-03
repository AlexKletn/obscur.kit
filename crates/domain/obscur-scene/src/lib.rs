//! Backend-independent Scene/Paint model (skeleton).
#![forbid(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]

mod color;

pub use color::Color;

pub mod clip;
pub mod command;
pub mod layer;
pub mod paint;
pub mod path;
pub mod scene;
pub mod surface;
