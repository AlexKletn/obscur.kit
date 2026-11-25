//! Color module.
mod color;

pub use color::Color;

mod alpha_mode;
pub use alpha_mode::AlphaMode;
mod color_space;
pub use color_space::ColorSpace;

pub mod convert;
