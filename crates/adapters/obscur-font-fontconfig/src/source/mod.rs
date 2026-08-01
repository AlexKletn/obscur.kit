#![doc = include_str!("mod.doc/_module.md")]
mod attribute_scale;
mod fontconfig_source;
mod system_face;
pub use fontconfig_source::FontconfigSource;
pub use system_face::SystemFace;
