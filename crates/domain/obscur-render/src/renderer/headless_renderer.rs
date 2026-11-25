use obscur_capability::SupportLevel;

use crate::frame::{FrameSubmission, PresentResult};
use crate::renderer::{Renderer, RendererCapabilities, RendererError, RendererId, SceneEffect};
use crate::target::RenderTarget;

#[doc = include_str!("../../docs/api/renderer/headless_renderer.doc/HeadlessRenderer.md")]
#[derive(Debug, Clone)]
pub struct HeadlessRenderer {
    id: RendererId,
    capabilities: RendererCapabilities,
    frames_presented: u64,
}

impl HeadlessRenderer {
    /// Creates a headless renderer with the given `id` and the conservative
    /// baseline [`RendererCapabilities`], having presented no frames.
    #[must_use]
    pub fn new(id: RendererId) -> Self {
        Self {
            id,
            capabilities: RendererCapabilities::new(),
            frames_presented: 0,
        }
    }

    /// The number of frames this renderer has presented so far.
    #[must_use]
    pub const fn frames_presented(&self) -> u64 {
        self.frames_presented
    }
}

impl Default for HeadlessRenderer {
    fn default() -> Self {
        Self::new(RendererId::from_raw(0))
    }
}

impl Renderer for HeadlessRenderer {
    fn id(&self) -> RendererId {
        self.id
    }

    fn capabilities(&self) -> RendererCapabilities {
        self.capabilities
    }

    fn render(
        &mut self,
        submission: &FrameSubmission,
        _target: &RenderTarget,
    ) -> Result<PresentResult, RendererError> {
        // Deterministic: present the frame by echoing the generation that
        // reached the "screen", and record that a frame was presented.
        self.frames_presented += 1;
        Ok(PresentResult::Presented {
            generation: submission.generation(),
        })
    }

    fn supports_effect(&self, _effect: SceneEffect) -> SupportLevel {
        // This backend draws no pixels, so it reproduces no effect (spec §5.2).
        SupportLevel::Unavailable
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::RenderFrame;
    use crate::target::{RenderTarget, WindowTarget};
    use obscur_geometry::Size;
    use obscur_scene::command::DisplayList;
    use obscur_scene::scene::{SceneGeneration, SceneSnapshot};

    fn submission(generation: u64) -> FrameSubmission {
        FrameSubmission::new(RenderFrame::new(
            SceneSnapshot::new(DisplayList::new(), SceneGeneration::from_raw(generation)),
            Size::new(320.0, 240.0),
        ))
    }

    fn a_target() -> RenderTarget {
        RenderTarget::window(WindowTarget::new(Size::new(320.0, 240.0)))
    }

    #[test]
    fn render_presents_and_echoes_generation() {
        let mut renderer = HeadlessRenderer::new(RendererId::from_raw(2));
        let result = renderer.render(&submission(9), &a_target());
        assert_eq!(
            result,
            Ok(PresentResult::Presented {
                generation: SceneGeneration::from_raw(9),
            })
        );
    }

    #[test]
    fn frames_presented_increments_per_render() {
        let mut renderer = HeadlessRenderer::default();
        assert_eq!(renderer.frames_presented(), 0);
        let _ = renderer.render(&submission(1), &a_target());
        let _ = renderer.render(&submission(2), &a_target());
        assert_eq!(renderer.frames_presented(), 2);
    }

    #[test]
    fn reproduces_no_effect_because_it_rasterizes_nothing() {
        let renderer = HeadlessRenderer::default();
        for effect in SceneEffect::ALL {
            assert_eq!(
                renderer.supports_effect(effect),
                SupportLevel::Unavailable,
                "{effect} must not be claimed by a backend that draws no pixels"
            );
        }
    }

    #[test]
    fn id_and_capabilities_reflect_construction() {
        let renderer = HeadlessRenderer::new(RendererId::from_raw(5));
        assert_eq!(renderer.id(), RendererId::from_raw(5));
        assert_eq!(renderer.capabilities(), RendererCapabilities::new());
    }
}
