//! The object-safe renderer port and a reference software backend (spec §5.2).

use obscur_capability::SupportLevel;

use crate::frame::{FrameSubmission, PresentResult};
use crate::renderer::{RendererCapabilities, RendererError, RendererId, SceneEffect};
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
/// Contract (spec §5.2): "Unsupported effects and fallbacks are explicit
/// diagnostics" — and backends never panic. Concretely:
///
/// 1. [`supports_effect`](Self::supports_effect) answers, per [`SceneEffect`], at
///    what [`SupportLevel`] this backend reproduces it. The answer must describe
///    what the backend **does**, not what it wishes it did: an effect whose
///    defining parameter the backend ignores is [`SupportLevel::Unavailable`],
///    however much of the mark still reaches the frame.
/// 2. A backend that draws a scene demanding an effect it reports `Unavailable`
///    emits the explicit diagnostic —
///    [`EffectFallbackReporter`](crate::renderer::EffectFallbackReporter) over
///    [`unsupported_effects`](crate::renderer::unsupported_effects) — and then
///    draws its documented fallback. Silently dropping the effect is the one thing
///    §5.2 forbids.
/// 3. An effect for which the backend has no fallback at all yields
///    `Err(`[`RendererError::UnsupportedEffect`]`)`; device loss surfaces through
///    the typed outcomes rather than a crash.
///
/// Both in-tree backends have a documented fallback for every effect they report
/// `Unavailable` (the CPU rasterizer approximates by design), so neither *returns*
/// `UnsupportedEffect` from `render` today — they raise it as a diagnostic instead.
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
    /// reached the screen. An effect the backend cannot reproduce yields a
    /// documented fallback `Ok` — with the diagnostic of point 2 above raised, so
    /// the fallback is never silent — or, absent any fallback,
    /// `Err(`[`RendererError::UnsupportedEffect`]`)`. Never a panic. Device loss
    /// surfaces as `Err(`[`RendererError::DeviceLost`]`)` or
    /// [`PresentResult::DeviceLost`].
    ///
    /// # Errors
    ///
    /// Returns a [`RendererError`] when the submission cannot be presented — for
    /// example an unsupported effect or a lost device.
    fn render(
        &mut self,
        submission: &FrameSubmission,
        target: &RenderTarget,
    ) -> Result<PresentResult, RendererError>;

    /// The honest [`SupportLevel`] at which this backend reproduces `effect`, so a
    /// host can find out what it will actually get before it asks for it
    /// (spec §5.2).
    ///
    /// The vocabulary is closed ([`SceneEffect`]) precisely so this table can be
    /// audited: every arm is a claim about what the backend's own paint path does
    /// with that effect's parameters, and the compiler refuses a table with a gap.
    /// The levels mean:
    ///
    /// - [`Native`](SupportLevel::Native): reproduced by a first-class primitive of
    ///   the backend.
    /// - [`Emulated`](SupportLevel::Emulated): reproduced by other means; the
    ///   effect's parameters are all honored and only rasterization differs
    ///   (the §5.2 determinism boundary).
    /// - [`Limited`](SupportLevel::Limited): reproduced under a restriction the
    ///   backend documents on this method — some input of the effect is not
    ///   modeled.
    /// - [`Unavailable`](SupportLevel::Unavailable): **not** reproduced. Whatever
    ///   the backend draws instead is a fallback, not the effect. Ignoring a
    ///   parameter entirely is this level, not `Emulated`.
    #[must_use]
    fn supports_effect(&self, effect: SceneEffect) -> SupportLevel;
}

/// A deterministic, no-GPU reference [`Renderer`] — the software/test backend of
/// spec §5.2.
///
/// `HeadlessRenderer` presents every submitted frame by echoing its scene
/// generation, so a test can assert exactly which generation "reached the
/// screen" without a real device. It counts the frames it has presented and holds
/// no GPU or OS handle.
///
/// It rasterizes **nothing** — there is no pixel buffer here at all — so it
/// reports [`SupportLevel::Unavailable`] for every effect, the same honest answer
/// the other null objects in this workspace give. (It previously claimed
/// `Emulated` for everything on the grounds that "a software rasterizer emulates
/// all effects"; this one is not a rasterizer.) For a real software backend that
/// draws pixels, see `obscur-render-cpu`.
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
