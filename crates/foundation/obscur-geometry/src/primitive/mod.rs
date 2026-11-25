//! Primitive module.
mod corner_radii;
mod insets;
mod point;
mod rect;
mod size;

pub use corner_radii::CornerRadii;
mod corner_radius;
pub use corner_radius::CornerRadius;
pub use insets::Insets;
pub use point::Point;
pub use rect::Rect;
pub use size::Size;

mod edges;
mod vector;
