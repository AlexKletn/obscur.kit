#![doc = include_str!("../../docs/api/conversion/mod.doc/_module.md")]
mod blur_layer;
pub use blur_layer::BlurLayer;
mod brush_converter;
pub use brush_converter::BrushConverter;
mod clip_converter;
pub use clip_converter::ClipConverter;
mod color_converter;
pub use color_converter::ColorConverter;
mod image_blobs;
mod image_painter;
mod path_converter;
pub use path_converter::PathConverter;
mod scene_converter;
pub use scene_converter::SceneConverter;
pub(crate) use scene_converter::layer_blur_radius;
#[cfg(feature = "lyon")]
pub(crate) use scene_converter::{FAUX_ITALIC_SKEW, compose, mix};
mod scene_scope;
mod scene_segment;
pub use scene_segment::SceneSegment;
#[cfg(feature = "hybrid")]
pub(crate) mod hybrid_scene;
mod shadow_painter;
