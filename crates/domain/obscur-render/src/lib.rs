//! Narrow renderer port. No concrete GPU/backend implementation lives here.
#![forbid(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]

pub mod renderer;

pub use renderer::{HeadlessRenderer, Renderer, RendererCapabilities, RendererError, RendererId};

pub mod extension;
pub mod frame;
pub mod target;
