//! Value module.
mod interpolatable;
pub use interpolatable::{Interpolatable, lerp_f32};
mod interpolation_result;
pub use interpolation_result::InterpolationResult;
mod motion_value;
pub use motion_value::MotionValue;
