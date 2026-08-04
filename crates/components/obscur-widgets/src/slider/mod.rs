#![doc = include_str!("mod.doc/_module.md")]

mod slider;
pub use slider::Slider;
mod slider_painter;
pub use slider_painter::paint_slider;
