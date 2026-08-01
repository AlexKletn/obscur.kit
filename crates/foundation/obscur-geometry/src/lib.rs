#![doc = include_str!("lib.doc/_module.md")]
#![forbid(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]

mod constraint;
mod primitive;
mod snapping;
mod transform;
mod unit;

pub use constraint::BoxConstraints;
pub use primitive::{CornerRadii, CornerRadius, Insets, Point, Rect, Size};
pub use snapping::PixelSnapMode;
pub use transform::AffineTransform;
pub use unit::{LogicalPx, PhysicalPx, ScaleFactor};

pub mod space;
