//! Primitive module.
mod corner_radii;
mod insets;
mod point;
mod rect;
mod size;

pub use corner_radii::{CornerRadii, CornerRadius};
pub use insets::Insets;
pub use point::Point;
pub use rect::Rect;
pub use size::Size;

mod edges;
mod vector;
