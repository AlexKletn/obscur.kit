#![doc = include_str!("mod.doc/_module.md")]
mod handler_calls;
mod painted_images;

pub use handler_calls::counting_handler;
pub use painted_images::PaintedImages;
