#![doc = include_str!("../docs/api/lib.doc/_module.md")]
#![deny(unsafe_op_in_unsafe_fn)]

mod face_metrics;
mod registry;
mod shaper;

pub use face_metrics::FaceMetrics;
pub use registry::FontRegistry;
pub use shaper::HarfrustShaper;
