#![doc = include_str!("mod.doc/_module.md")]
mod builtin_components;
pub use builtin_components::BuiltinComponents;
mod css_pipeline;
pub use css_pipeline::CssPipeline;
#[cfg(feature = "font-fontconfig")]
mod fontconfig_fallback;
#[cfg(feature = "font-fontconfig")]
pub use fontconfig_fallback::FontconfigFallback;
mod kit_bootstrap;
pub use kit_bootstrap::KitBootstrap;
mod text_stack;
pub use text_stack::{FontFaceBytes, FontSet, InstalledFonts};
