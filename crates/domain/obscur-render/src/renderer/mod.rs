#![doc = include_str!("mod.doc/_module.md")]
mod renderer;
mod renderer_id;

pub use renderer::{HeadlessRenderer, Renderer};
pub use renderer_id::RendererId;

mod effect_report;
mod scene_effect;
pub use effect_report::{
    EFFECT_FALLBACK, EffectFallback, EffectFallbackReporter, UNSUPPORTED_EFFECT, scene_effects,
    text_run_font_handles, unsupported_effect_error, unsupported_effects,
};
pub use scene_effect::SceneEffect;

mod renderer_capabilities;
pub use renderer_capabilities::RendererCapabilities;
mod renderer_error;
pub use renderer_error::RendererError;
