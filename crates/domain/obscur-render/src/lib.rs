#![doc = include_str!("lib.doc/_module.md")]
#![forbid(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]

pub mod renderer;

pub use renderer::{
    EffectFallback, EffectFallbackReporter, HeadlessRenderer, Renderer, RendererCapabilities,
    RendererError, RendererId, SceneEffect, text_run_font_handles, unsupported_effects,
};

pub mod extension;
pub mod frame;
pub mod target;
