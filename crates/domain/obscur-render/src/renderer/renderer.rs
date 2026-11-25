//! The object-safe renderer port (spec §5.2).

use obscur_capability::SupportLevel;

use crate::frame::{FrameSubmission, PresentResult};
use crate::renderer::{RendererCapabilities, RendererError, RendererId, SceneEffect};
use crate::target::RenderTarget;

#[doc = include_str!("../../docs/api/renderer/renderer.doc/Renderer.md")]
pub trait Renderer {
    /// The stable identity of this backend instance (spec §5.2).
    #[must_use]
    fn id(&self) -> RendererId;

    /// The capability profile this backend advertises (spec §5.2).
    #[must_use]
    fn capabilities(&self) -> RendererCapabilities;

    #[doc = include_str!("../../docs/api/renderer/renderer.doc/render.md")]
    fn render(
        &mut self,
        submission: &FrameSubmission,
        target: &RenderTarget,
    ) -> Result<PresentResult, RendererError>;

    #[doc = include_str!("../../docs/api/renderer/renderer.doc/supports_effect.md")]
    #[must_use]
    fn supports_effect(&self, effect: SceneEffect) -> SupportLevel;
}

#[cfg(test)]
mod tests {
    use super::super::HeadlessRenderer;
    use super::*;

    #[test]
    fn renderer_is_object_safe() {
        let _: Box<dyn Renderer> = Box::new(HeadlessRenderer::new(RendererId::from_raw(1)));
    }
}
