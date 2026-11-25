//! Typed geometry: logical/physical units, primitives, transforms and constraints.
#![forbid(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]

mod constraint;
mod primitive;
mod snapping;
mod transform;
mod unit;

pub use constraint::BoxConstraints;
pub use primitive::{CornerRadii, CornerRadius, Insets, Point, Rect, Size};
pub use snapping::{PixelSnapMode, PixelSnapPolicy};
pub use transform::{AffineTransform, CoordinateMapper};
pub use unit::{LogicalPx, PhysicalPx, ScaleFactor};

pub mod space;

mod saturating_f32;
pub use saturating_f32::saturating_f32;
