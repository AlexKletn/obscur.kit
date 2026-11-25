//! §5.3's canvas, put to its first shipped use: a bar that reports how far along something is.

mod progress;
pub use progress::Progress;
mod progress_shape;
pub use progress_shape::ProgressShape;
mod progress_painter;
pub use progress_painter::paint_progress;
