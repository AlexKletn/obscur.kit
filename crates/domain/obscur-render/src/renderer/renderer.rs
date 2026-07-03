//! The object-safe renderer port and a reference software backend (spec §5.2).

use obscur_capability::SupportLevel;

use crate::frame::{FrameSubmission, PresentResult};
use crate::renderer::{RendererCapabilities, RendererError, RendererId};
use crate::target::RenderTarget;

/// The narrow renderer port: a replaceable 2D backend (spec §5.2).
///
/// The base 2D backend (Vello) is hidden behind this trait, and alternates —
/// software/headless, platform, WASM/WebGPU, and test renderers — implement it.
/// Every backend must reproduce the determinism boundary of spec §5.2 (layout,
/// hit-test, paint order, clipping, and the [`SceneSnapshot`] it consumes)
/// identically; only rasterization, anti-aliasing, and precision may differ.
///
/// The trait is deliberately **object-safe** — no generic or `Self`-returning
/// methods — so a runtime can hold the chosen backend as `Box<dyn Renderer>`.
///
/// Contract (spec §5.2): backends report unsupported effects and fallbacks as
/// explicit diagnostics and never panic. An effect the backend cannot reproduce
/// must surface as [`RendererError::UnsupportedEffect`] or a documented fallback
/// [`Ok`], and device loss must surface through the typed outcomes rather than a
/// crash.
pub trait Renderer {
    /// The stable identity of this backend instance (spec §5.2).
    #[must_use]
    fn id(&self) -> RendererId;

    /// The capability profile this backend advertises (spec §5.2).
    #[must_use]
    fn capabilities(&self) -> RendererCapabilities;

    /// Renders and presents `submission` to `target` (spec §5.2).
    ///
    /// On success returns a [`PresentResult`] echoing which scene generation
    /// reached the screen. An effect the backend cannot reproduce must yield
    /// `Err(`[`RendererError::UnsupportedEffect`]`)` (the explicit fallback
    /// diagnostic) or a documented fallback `Ok` — **never a panic**. Device
    /// loss surfaces as `Err(`[`RendererError::DeviceLost`]`)` or
    /// [`PresentResult::DeviceLost`].
    fn render(
        &mut self,
        submission: &FrameSubmission,
        target: &RenderTarget,
    ) -> Result<PresentResult, RendererError>;

    /// The honest [`SupportLevel`] at which this backend offers a named `effect`
    /// (blur, backdrop, and the like), so callers can choose fallbacks
    /// (spec §5.2).
    #[must_use]
    fn supports_effect(&self, effect: &str) -> SupportLevel;
}

/// A deterministic, no-GPU reference [`Renderer`] — the software/test backend of
/// spec §5.2.
///
/// `HeadlessRenderer` presents every submitted frame by echoing its scene
/// generation, so a test can assert exactly which generation "reached the
/// screen" without a real device. It counts the frames it has presented and
/// reports [`SupportLevel::Emulated`] for every effect: a software rasterizer
/// emulates all effects, honoring the determinism boundary of spec §5.2 even
/// where a GPU backend would accelerate them. It holds no GPU or OS handle.
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

    fn supports_effect(&self, _effect: &str) -> SupportLevel {
        // A software renderer emulates every effect (spec §5.2).
        SupportLevel::Emulated
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
    fn renderer_is_object_safe() {
        let _: Box<dyn Renderer> = Box::new(HeadlessRenderer::new(RendererId::from_raw(1)));
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
    fn supports_every_effect_via_emulation() {
        let renderer = HeadlessRenderer::default();
        assert_eq!(renderer.supports_effect("blur"), SupportLevel::Emulated);
        assert_eq!(renderer.supports_effect("backdrop"), SupportLevel::Emulated);
        assert_eq!(renderer.supports_effect("anything"), SupportLevel::Emulated);
    }

    #[test]
    fn id_and_capabilities_reflect_construction() {
        let renderer = HeadlessRenderer::new(RendererId::from_raw(5));
        assert_eq!(renderer.id(), RendererId::from_raw(5));
        assert_eq!(renderer.capabilities(), RendererCapabilities::new());
    }
}
