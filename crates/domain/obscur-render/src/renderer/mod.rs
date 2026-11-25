//! Renderer module.
mod renderer;
mod renderer_id;

pub use renderer::Renderer;
mod headless_renderer;
pub use headless_renderer::HeadlessRenderer;
pub use renderer_id::RendererId;

mod effect_fallback;
mod effect_report;
pub use effect_fallback::EffectFallback;
mod scene_effect;
pub use effect_report::{
    EFFECT_FALLBACK, EffectFallbackReporter, UNSUPPORTED_EFFECT, unsupported_effect_error,
};
pub use scene_effect::SceneEffect;

mod renderer_capabilities;
pub use renderer_capabilities::RendererCapabilities;
mod renderer_error;
pub use renderer_error::RendererError;

mod scene_requirements;
pub use scene_requirements::{scene_effects, text_run_font_handles, unsupported_effects};
