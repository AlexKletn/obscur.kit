//! Extension module.
mod custom_pass;
pub use custom_pass::CustomPass;
mod pass_stage;
pub use pass_stage::PassStage;
mod render_extension;
pub use render_extension::RenderExtension;
mod trusted_render_context;
pub use trusted_render_context::TrustedRenderContext;
