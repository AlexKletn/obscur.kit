//! A backend-neutral render-graph description of a frame: [`RenderGraphSummary`].

use obscur_geometry::Size;
use obscur_scene::scene::SceneSnapshot;

use super::RenderPass;

#[doc = include_str!("../../docs/api/snapshot/render_graph_summary.doc/RenderGraphSummary.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct RenderGraphSummary {
    /// The ordered render passes the frame comprises.
    passes: Vec<RenderPass>,
    /// The target area the passes render into, in logical pixels.
    target: Size,
}

impl RenderGraphSummary {
    #[doc = include_str!("../../docs/api/snapshot/render_graph_summary.doc/new.md")]
    #[must_use]
    pub fn new(passes: Vec<RenderPass>, target: Size) -> Self {
        Self { passes, target }
    }

    #[doc = include_str!("../../docs/api/snapshot/render_graph_summary.doc/from_scene.md")]
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
