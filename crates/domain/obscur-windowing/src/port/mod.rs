#![doc = include_str!("mod.doc/_module.md")]
mod layer_backend;
pub use layer_backend::{LayerBackend, LayerError, LayerFeature, NullLayerBackend};
mod loop_waker;
pub use loop_waker::LoopWaker;
mod window_backend;
pub use window_backend::{NullWindowBackend, WindowBackend, WindowError, WindowFeature};
