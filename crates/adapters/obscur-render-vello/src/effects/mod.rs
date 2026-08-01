#![doc = include_str!("mod.doc/_module.md")]
mod color_matrix;
mod gaussian_blur;
pub use color_matrix::ColorMatrix;
pub use gaussian_blur::GaussianBlur;
