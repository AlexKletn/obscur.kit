#![doc = include_str!("mod.doc/_module.md")]
mod font_handle;
pub use font_handle::FontHandle;
mod image_handle;
pub use image_handle::ImageHandle;
mod resource_handle;
pub use resource_handle::{ResourceHandle, ResourceKind};
mod shader_handle;
pub use shader_handle::ShaderHandle;
mod stylesheet_handle;
pub use stylesheet_handle::StylesheetHandle;
