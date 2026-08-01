#![doc = include_str!("mod.doc/_module.md")]
mod adaptive_quality;
pub use adaptive_quality::AdaptiveQuality;
mod quality_level;
pub use quality_level::QualityLevel;
mod quality_policy;
pub use quality_policy::QualityPolicy;
