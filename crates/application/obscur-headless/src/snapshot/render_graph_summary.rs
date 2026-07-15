//! A backend-neutral render-graph description of a frame: [`RenderGraphSummary`].

use obscur_geometry::Size;
use obscur_scene::scene::SceneSnapshot;

use super::RenderPass;

/// A backend-neutral, GPU-free description of a frame's render graph (spec §7.4).
///
/// §7.4 lists the *render graph* as one of the eight headless outputs. A full
/// render graph — pass scheduling, transient targets, GPU resource aliasing — is
/// only meaningful with a concrete backend, so this type captures the
/// **backend-neutral structural description** that a headless frame determines:
/// the ordered [`passes`](Self::passes) and the [`target`](Self::target) they
/// render into. It invents no GPU data; every field is derived from the composed
/// scene and the viewport, so the same frame always yields the same summary.
///
/// For an ordinary headless frame the graph is a single `"scene"` pass (the
/// composited display list) into the viewport-sized target. Extension/custom
/// passes (spec §5.3) and GPU-specific passes are backend-gated and are not
/// modeled here; this is the deterministic skeleton every backend shares.
#[derive(Debug, Clone, PartialEq)]
pub struct RenderGraphSummary {
    /// The ordered render passes the frame comprises.
    passes: Vec<RenderPass>,
    /// The target area the passes render into, in logical pixels.
    target: Size,
}

impl RenderGraphSummary {
    /// Builds a summary from already-computed parts.
    ///
    /// Prefer [`from_scene`](Self::from_scene), which derives the single scene
    /// pass from a composed scene; this constructor exists for callers that hold
    /// the passes directly.
    #[must_use]
    pub fn new(passes: Vec<RenderPass>, target: Size) -> Self {
        Self { passes, target }
    }

    /// Derives the backend-neutral render graph for an ordinary headless frame:
    /// a single `"scene"` pass covering `scene`'s display list, rendering into a
    /// `target`-sized area.
    ///
    /// The pass counts come straight from the scene's
    /// [`SceneStatistics`](obscur_scene::scene::SceneStatistics), so capturing
    /// the same scene twice yields an equal summary (spec §7.14).
    #[must_use]
    pub fn from_scene(scene: &SceneSnapshot, target: Size) -> Self {
        let stats = scene.statistics();
        let passes = vec![RenderPass::new(
            "scene",
            stats.draw_count(),
            stats.command_count(),
        )];
        Self { passes, target }
    }

    /// The ordered render passes.
    #[must_use]
    pub fn passes(&self) -> &[RenderPass] {
        &self.passes
    }

    /// The number of render passes in the graph.
    #[must_use]
    pub fn pass_count(&self) -> usize {
        self.passes.len()
    }

    /// The target area the passes render into, in logical pixels.
    #[must_use]
    pub fn target(&self) -> Size {
        self.target
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_geometry::Rect;
    use obscur_scene::clip::Clip;
    use obscur_scene::color::Color;
    use obscur_scene::command::{DisplayList, PaintCommand};
    use obscur_scene::paint::Paint;
    use obscur_scene::path::Path;
    use obscur_scene::scene::SceneGeneration;

    fn draw() -> PaintCommand {
        PaintCommand::DrawPath {
            path: Path::rect(Rect::default()),
            paint: Paint::solid_fill(Color::WHITE),
        }
    }

    fn scene_of(commands: Vec<PaintCommand>) -> SceneSnapshot {
        SceneSnapshot::new(DisplayList::from_commands(commands), SceneGeneration::FIRST)
    }

    #[test]
    fn from_scene_emits_a_single_scene_pass() {
        let scene = scene_of(vec![
            PaintCommand::PushClip(Clip::rect(Rect::default())),
            draw(),
            draw(),
            PaintCommand::PopClip,
        ]);
        let summary = RenderGraphSummary::from_scene(&scene, Size::new(800.0, 600.0));

        assert_eq!(summary.pass_count(), 1);
        assert_eq!(summary.target(), Size::new(800.0, 600.0));
        let pass = &summary.passes()[0];
        assert_eq!(pass.label(), "scene");
        assert_eq!(pass.draw_count(), 2);
        assert_eq!(pass.command_count(), 4);
    }

    #[test]
    fn from_scene_is_deterministic() {
        let scene = scene_of(vec![draw()]);
        let target = Size::new(320.0, 240.0);
        assert_eq!(
            RenderGraphSummary::from_scene(&scene, target),
            RenderGraphSummary::from_scene(&scene, target),
        );
    }

    #[test]
    fn empty_scene_still_has_one_pass_with_zero_draws() {
        let summary = RenderGraphSummary::from_scene(&scene_of(vec![]), Size::new(1.0, 1.0));
        assert_eq!(summary.pass_count(), 1);
        assert_eq!(summary.passes()[0].draw_count(), 0);
        assert_eq!(summary.passes()[0].command_count(), 0);
    }
}
