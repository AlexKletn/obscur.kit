#![doc = include_str!("lib.doc/_module.md")]
#![forbid(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]

mod canvas;
pub use canvas::Canvas;
mod container;
pub use container::Container;
mod foreign_surface;
pub use foreign_surface::ForeignSurface;
mod image;
pub use image::Image;
mod layer;
pub use layer::Layer;
mod portal;
pub use portal::Portal;
mod scroll;
pub use scroll::Scroll;
mod text;
pub use text::Text;
