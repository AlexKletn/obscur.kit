#![doc = include_str!("pipeline_snapshot.doc/_module.md")]

use obscur_accessibility::tree::SemanticSnapshot;
use obscur_geometry::Size;
use obscur_runtime::SemanticTreeBuilder;
use obscur_runtime::orchestration::{
    ComputedStyles, LayoutBoxes, LayoutCoordinator, SceneCoordinator, ShapedChMetrics,
    StyleCoordinator, StyleInputs, root_font_size,
};
use obscur_scene::command::DisplayList;
use obscur_scene::scene::{Scene, SceneSnapshot};
use obscur_style::property::EnvironmentValues;
use obscur_tree::tree::{RetainedTree, TreeSnapshot};

use super::{ImageSnapshot, RenderGraphSummary};

#[doc = include_str!("pipeline_snapshot.doc/PipelineSnapshot.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct PipelineSnapshot {
    #[doc = include_str!("pipeline_snapshot.doc/tree.md")]
    tree: TreeSnapshot,
    #[doc = include_str!("pipeline_snapshot.doc/computed.md")]
    computed: ComputedStyles,
    #[doc = include_str!("pipeline_snapshot.doc/layout.md")]
    layout: LayoutBoxes,
    #[doc = include_str!("pipeline_snapshot.doc/scene.md")]
    scene: SceneSnapshot,
    #[doc = include_str!("pipeline_snapshot.doc/paint.md")]
    paint: DisplayList,
    #[doc = include_str!("pipeline_snapshot.doc/accessibility.md")]
    accessibility: SemanticSnapshot,
    #[doc = include_str!("pipeline_snapshot.doc/render_graph.md")]
    render_graph: RenderGraphSummary,
    #[doc = include_str!("pipeline_snapshot.doc/image.md")]
    image: Option<ImageSnapshot>,
}

impl PipelineSnapshot {
    #[doc = include_str!("pipeline_snapshot.doc/new.md")]
    #[must_use]
    pub fn new(
        tree: TreeSnapshot,
        computed: ComputedStyles,
        layout: LayoutBoxes,
        scene: SceneSnapshot,
        paint: DisplayList,
        accessibility: SemanticSnapshot,
        render_graph: RenderGraphSummary,
    ) -> Self {
        Self {
            tree,
            computed,
            layout,
            scene,
            paint,
            accessibility,
            render_graph,
            image: None,
        }
    }

    #[doc = include_str!("pipeline_snapshot.doc/with_image.md")]
    #[must_use]
    pub fn with_image(mut self, image: ImageSnapshot) -> Self {
        self.image = Some(image);
        self
    }

    #[doc = include_str!("pipeline_snapshot.doc/capture.md")]
    #[must_use]
    pub fn capture(tree: &RetainedTree, inputs: &StyleInputs, viewport: Size) -> Self {
        let tree_snapshot = tree.snapshot();

        // The scene coordinator owns the paragraph orchestrator; build it first so
        // every later stage shapes through the same orchestrator the scene finally
        // re-wraps and paints with (so box height and painted line count agree,
        // deterministically). It is built before the style stage, not just before
        // layout, because the style stage needs it too: it is the `ch` port.
        let scene_coordinator = SceneCoordinator::new();

        // `ch` is the `0` glyph's advance, so it must be shaped rather than folded
        // from numbers. Supplying the port here folds it to pixels at
        // computed-value time like every other relative unit — which is what makes
        // this snapshot's *paint* stage agree with its *layout* stage about a `ch`
        // box, rather than reserving space and painting nothing.
        let ch_metrics = ShapedChMetrics::new(scene_coordinator.paragraph());
        let style = StyleCoordinator::new();
        // No environment: this snapshot is a pure function of the tree, the sheets
        // and the viewport, so a host inset that varied between runs would make it
        // non-deterministic — which is the one thing it exists to guarantee.
        let computed = style.resolve(
            tree,
            inputs,
            viewport,
            EnvironmentValues::none(),
            Some(&ch_metrics),
        );

        let layout = LayoutCoordinator::new();
        let root_font = root_font_size(&computed, tree.root());
        // The same orchestrator also supplies `ch`: the unit is the `0` glyph's
        // advance, so it must be shaped rather than folded from numbers. Measured
        // only for a node that writes `ch` — almost none do.
        let mut layout_styles = layout.layout_styles_from(
            &computed,
            viewport,
            root_font,
            Some(scene_coordinator.paragraph()),
        );
        // Named-area placement (`grid-area: <name>` against the container's
        // `grid-template-areas`) needs the tree to resolve, so it patches the
        // freshly built styles here; items with no named area are left untouched.
        layout.resolve_named_grid_areas(&mut layout_styles, tree, &computed);
        // Authored grid templates become a per-node side-table so a grid container
        // dispatches the real track-sizing algorithm (spec §4.1).
        let grid_templates = layout.grid_templates_from(
            &computed,
            viewport,
            root_font,
            Some(scene_coordinator.paragraph()),
        );
        let boxes = layout.layout(
            tree,
            &layout_styles,
            &computed,
            &grid_templates,
            viewport,
            // Headless snapshots have no live scroll: the viewport is unscrolled,
            // so `position: sticky` nodes resolve to their static positions.
            (0.0, 0.0),
            scene_coordinator.paragraph(),
        );

        // The headless snapshot host owns no editing session, so it paints no
        // caret or selection highlight (`None`).
        let display_list = scene_coordinator.compose(tree, &computed, &boxes, None);
        let mut scene = Scene::new();
        scene.set_root(display_list);
        let scene_snapshot = scene.snapshot();

        // The paint stage is the flat paint-command stream a rasterizer
        // consumes, distinct from the scene stage (which also carries the scene
        // generation and statistics).
        let paint = scene_snapshot.display_list().clone();

        // The accessibility stage projects the retained tree into a semantic
        // tree (spec §7.1). A headless host tracks no focus, so no node is
        // marked focused.
        let accessibility = SemanticTreeBuilder::snapshot(tree, None);

        // The render-graph stage is the backend-neutral pass description.
        let render_graph = RenderGraphSummary::from_scene(&scene_snapshot, viewport);

        Self::new(
            tree_snapshot,
            computed,
            boxes,
            scene_snapshot,
            paint,
            accessibility,
            render_graph,
        )
    }

    #[doc = include_str!("pipeline_snapshot.doc/tree_2.md")]
    #[must_use]
    pub fn tree(&self) -> &TreeSnapshot {
        &self.tree
    }

    #[doc = include_str!("pipeline_snapshot.doc/computed_2.md")]
    #[must_use]
    pub fn computed(&self) -> &ComputedStyles {
        &self.computed
    }

    #[doc = include_str!("pipeline_snapshot.doc/layout_2.md")]
    #[must_use]
    pub fn layout(&self) -> &LayoutBoxes {
        &self.layout
    }

    #[doc = include_str!("pipeline_snapshot.doc/scene_2.md")]
    #[must_use]
    pub fn scene(&self) -> &SceneSnapshot {
        &self.scene
    }

    #[doc = include_str!("pipeline_snapshot.doc/paint_2.md")]
    #[must_use]
    pub fn paint(&self) -> &DisplayList {
        &self.paint
    }

    #[doc = include_str!("pipeline_snapshot.doc/accessibility_2.md")]
    #[must_use]
    pub fn accessibility(&self) -> &SemanticSnapshot {
        &self.accessibility
    }

    #[doc = include_str!("pipeline_snapshot.doc/render_graph_2.md")]
    #[must_use]
    pub fn render_graph(&self) -> &RenderGraphSummary {
        &self.render_graph
    }

    #[doc = include_str!("pipeline_snapshot.doc/image_2.md")]
    #[must_use]
    pub fn image(&self) -> Option<&ImageSnapshot> {
        self.image.as_ref()
    }

    #[doc = include_str!("pipeline_snapshot.doc/node_count.md")]
    #[must_use]
    pub fn node_count(&self) -> usize {
        self.tree.len()
    }

    #[doc = include_str!("pipeline_snapshot.doc/fill_count.md")]
    #[must_use]
    pub fn fill_count(&self) -> usize {
        self.scene.display_list().len()
    }

    #[doc = include_str!("pipeline_snapshot.doc/is_balanced.md")]
    #[must_use]
    pub fn is_balanced(&self) -> bool {
        self.scene.is_balanced()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_geometry::Size;

    #[test]
    fn capture_from_empty_tree_is_empty_and_balanced() {
        let tree = RetainedTree::new();
        let inputs = StyleInputs::new();
        let snapshot = PipelineSnapshot::capture(&tree, &inputs, Size::new(800.0, 600.0));

        assert_eq!(snapshot.node_count(), 0);
        assert_eq!(snapshot.fill_count(), 0);
        assert!(snapshot.is_balanced());
        assert!(snapshot.computed().is_empty());
        assert!(snapshot.layout().is_empty());
        // The new stages are present and empty/neutral.
        assert!(snapshot.paint().is_empty());
        assert!(snapshot.accessibility().is_empty());
        assert_eq!(snapshot.render_graph().pass_count(), 1);
        assert!(snapshot.image().is_none());
    }

    #[test]
    fn capture_does_not_mutate_tree() {
        let tree = RetainedTree::new();
        let version_before = tree.version();
        let inputs = StyleInputs::new();
        let _ = PipelineSnapshot::capture(&tree, &inputs, Size::new(800.0, 600.0));
        assert_eq!(tree.version(), version_before);
    }

    #[test]
    fn capturing_twice_yields_equal_snapshots() {
        let tree = RetainedTree::new();
        let inputs = StyleInputs::new();
        let viewport = Size::new(640.0, 480.0);

        let first = PipelineSnapshot::capture(&tree, &inputs, viewport);
        let second = PipelineSnapshot::capture(&tree, &inputs, viewport);
        assert_eq!(first, second);
    }

    #[test]
    fn paint_stage_mirrors_the_scene_display_list() {
        use obscur_identity::ComponentId;
        use obscur_style::cascade::ResolvedDeclarations;
        use obscur_style::property::{Length, PropertyId, PropertyValue, StyleColor};
        use obscur_view::view::{View, ViewFragment, ViewNode};

        use crate::host::HeadlessHost;
        use obscur_identity::RuntimeId;

        // Render a view whose child is styled into a solid box, so the scene has
        // real paint commands.
        let view = View::from_fragment(ViewFragment::from_nodes(vec![ViewNode::element(
            ComponentId::from_raw(10),
        )]));
        let mut host = HeadlessHost::new(RuntimeId::from_raw(1));
        host.render_styled(&view, &[]);
        let child = {
            let tree = host.tree();
            let root = tree.root().expect("root");
            tree.get(root).unwrap().child_nodes().first().copied()
        };

        let mut inputs = StyleInputs::new();
        if let Some(child) = child {
            let mut decls = ResolvedDeclarations::new();
            decls.insert(
                PropertyId::BackgroundColor,
                PropertyValue::Color(StyleColor::rgba(1.0, 0.0, 0.0, 1.0)),
            );
            decls.insert(PropertyId::Width, PropertyValue::Length(Length::px(100.0)));
            decls.insert(PropertyId::Height, PropertyValue::Length(Length::px(50.0)));
            inputs.set_cascaded(child, decls);
        }
        host.render_styled(&view, &[]);

        let snapshot = PipelineSnapshot::capture(host.tree(), &inputs, host.config().viewport());

        // The paint stage is exactly the scene stage's display-list command
        // stream (the source a rasterizer consumes).
        assert_eq!(snapshot.paint(), snapshot.scene().display_list());
        assert!(!snapshot.paint().is_empty(), "the styled child emits paint");
        assert_eq!(snapshot.paint().len(), snapshot.fill_count());
    }

    #[test]
    fn accessibility_stage_reflects_the_rendered_tree() {
        use obscur_accessibility::semantic::SemanticRole;
        use obscur_identity::{ComponentId, RuntimeId};
        use obscur_view::view::{TYPE, View, ViewFragment, ViewNode, ViewProperty, ViewValue};

        use crate::host::HeadlessHost;

        // A single `button` element.
        let view = View::from_fragment(ViewFragment::from_nodes(vec![
            ViewNode::element(ComponentId::from_raw(10))
                .with_prop(ViewProperty::new(TYPE, ViewValue::Text("button".into()))),
        ]));
        let mut host = HeadlessHost::new(RuntimeId::from_raw(1));
        host.render_styled(&view, &[]);

        let snapshot =
            PipelineSnapshot::capture(host.tree(), &StyleInputs::new(), host.config().viewport());

        let a11y = snapshot.accessibility();
        // One semantic node per retained node, keyed by the same ids.
        assert_eq!(a11y.len(), snapshot.node_count());
        assert!(!a11y.is_empty());
        assert_eq!(a11y.root(), snapshot.tree().root());
        // The button element surfaces a Button role.
        let has_button = a11y.iter().any(|(_, n)| n.role() == SemanticRole::Button);
        assert!(
            has_button,
            "the button element becomes a Button semantic node"
        );
    }

    #[test]
    fn render_graph_stage_describes_the_scene_pass() {
        let tree = RetainedTree::new();
        let inputs = StyleInputs::new();
        let viewport = Size::new(320.0, 240.0);
        let snapshot = PipelineSnapshot::capture(&tree, &inputs, viewport);

        let graph = snapshot.render_graph();
        assert_eq!(graph.pass_count(), 1);
        assert_eq!(graph.target(), viewport);
        assert_eq!(graph.passes()[0].label(), "scene");
    }

    #[test]
    fn with_image_attaches_the_backend_dependent_stage() {
        let tree = RetainedTree::new();
        let inputs = StyleInputs::new();
        let snapshot = PipelineSnapshot::capture(&tree, &inputs, Size::new(4.0, 4.0));
        assert!(snapshot.image().is_none());

        let image = ImageSnapshot::from_rgba(4, 4, &[0u8; 64]);
        let with = snapshot.with_image(image.clone());
        assert_eq!(with.image(), Some(&image));
    }
}
