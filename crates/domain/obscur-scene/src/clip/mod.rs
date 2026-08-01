#![doc = include_str!("mod.doc/_module.md")]
mod clip;
pub use clip::Clip;
mod clip_shape;
pub use clip_shape::ClipShape;
mod mask;
pub use mask::Mask;
mod mask_kind;
pub use mask_kind::MaskKind;
