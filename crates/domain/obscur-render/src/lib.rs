//! Narrow renderer port. No concrete GPU/backend implementation lives here.
#![forbid(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]

mod renderer;

pub use renderer::{Renderer, RendererId};

pub mod extension;
pub mod frame;
pub mod target;
