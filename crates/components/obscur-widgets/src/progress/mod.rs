#![doc = include_str!("mod.doc/_module.md")]

mod progress;
pub use progress::{Progress, ProgressShape};
mod progress_painter;
pub use progress_painter::paint_progress;
