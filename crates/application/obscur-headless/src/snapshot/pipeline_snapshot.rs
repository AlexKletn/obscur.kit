//! An immutable capture of the frame pipeline stages: [`PipelineSnapshot`].

use obscur_accessibility::tree::SemanticSnapshot;
use obscur_geometry::Size;
use obscur_runtime::SemanticTreeBuilder;
use obscur_runtime::orchestration::{
    ComputedStyles, LayoutBoxes, LayoutCoordinator, SceneCoordinator, StyleCoordinator,
    StyleInputs, root_font_size,
};
use obscur_scene::command::DisplayList;
use obscur_scene::scene::{Scene, SceneSnapshot};
use obscur_tree::tree::{RetainedTree, TreeSnapshot};

use super::{ImageSnapshot, RenderGraphSummary};

/// An immutable, backend-independent capture of every frame-pipeline stage for
/// one frame, for golden testing (spec §7.4, §7.14).
///
/// A `PipelineSnapshot` bundles the eight stages §7.4 enumerates. Seven are
/// backend-independent and captured deterministically with no GPU:
///
/// 1. the retained [`TreeSnapshot`];
/// 2. the resolved [`ComputedStyles`];
/// 3. the arranged [`LayoutBoxes`];
/// 4. the composed [`SceneSnapshot`];
/// 5. the paint-command stream ([`DisplayList`]);
/// 6. the accessibility [`SemanticSnapshot`] (spec §7.1);
/// 7. the backend-neutral [`RenderGraphSummary`].
///
/// The eighth stage, the rendered [`ImageSnapshot`], is *backend-dependent*: a
/// raster comes from a concrete renderer, not the backend-independent pipeline,
/// so [`capture`](Self::capture) leaves it absent and a caller attaches one with
/// [`with_image`](Self::with_image) after running a backend such as
/// `obscur-render-cpu`.
///
/// It is the concrete artefact behind the Phase-1 slice proof that
/// `component → desired tree → retained tree → style → layout → scene` runs
/// deterministically headless (§5.13).
///
/// # Order-independent equality
///
/// The type derives [`PartialEq`], and two of its fields are
/// [`HashMap`](std::collections::HashMap)s whose iteration order is unstable.
/// `HashMap` equality is order-independent, so two captures of the *same* frame
/// state compare equal even though their maps iterate in different orders. For a
/// stable, byte-for-byte comparable form (e.g. to write to a golden file), run
/// it through [`SnapshotNormalizer`](super::SnapshotNormalizer).
#[derive(Debug, Clone, PartialEq)]
pub struct PipelineSnapshot {
    /// The reconciled retained tree at capture (spec §5.13).
    tree: TreeSnapshot,
    /// The per-node computed styles resolved from the tree (spec §4.14).
    computed: ComputedStyles,
    /// The per-node absolute layout boxes (spec §4.2).
    layout: LayoutBoxes,
    /// The composed, paint-ordered scene (spec §5.1).
    scene: SceneSnapshot,
    /// The flat paint-command stream a rasterizer consumes (spec §5.1, §7.4).
    paint: DisplayList,
    /// The semantic accessibility tree linked to the retained tree (spec §7.1).
    accessibility: SemanticSnapshot,
    /// The backend-neutral render-graph description (spec §7.4).
    render_graph: RenderGraphSummary,
    /// The rendered raster image, present only once a backend has produced it
    /// (spec §7.4). Backend-dependent, hence optional.
    image: Option<ImageSnapshot>,
}

impl PipelineSnapshot {
    /// Builds a snapshot directly from already-captured stage data.
    ///
    /// The rendered image is left absent; attach one with
    /// [`with_image`](Self::with_image). Prefer [`capture`](Self::capture),
    /// which derives every backend-independent stage from a retained tree in one
    /// call; this constructor exists for callers that already hold the
    /// individual stage artefacts.
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

    /// Returns this snapshot with `image` attached as its rendered-image stage
    /// (spec §7.4).
    ///
    /// This is how the one backend-dependent stage is supplied: a caller
    /// rasterizes the [`scene`](Self::scene) with a concrete renderer (e.g.
    /// `obscur-render-cpu`), builds an [`ImageSnapshot`] from the resulting
    /// pixels, and attaches it here.
    #[must_use]
    pub fn with_image(mut self, image: ImageSnapshot) -> Self {
        self.image = Some(image);
        self
    }

    /// Captures every backend-independent pipeline stage from the current state
    /// of `tree`, deterministically (spec §5.13, §7.4, §7.14).
    ///
    /// This re-runs the read-only style → layout → scene stages purely to
    /// observe them; it takes `tree` by shared reference and **does not mutate
    /// it**. Concretely it:
    ///
    /// 1. captures the retained tree via [`RetainedTree::snapshot`];
    /// 2. resolves [`ComputedStyles`] with a fresh [`StyleCoordinator`] against
    ///    `inputs`;
    /// 3. derives [`LayoutBoxes`] with a fresh [`LayoutCoordinator`]
    ///    ([`layout_styles_from`](LayoutCoordinator::layout_styles_from) then
    ///    [`layout`](LayoutCoordinator::layout) under `viewport`);
    /// 4. composes a display list with a fresh [`SceneCoordinator`], installs it
    ///    in a fresh [`Scene`], and snapshots it (the **scene** stage);
    /// 5. clones the composed display list as the **paint** stage;
    /// 6. projects the retained tree into a [`SemanticSnapshot`] via
    ///    [`SemanticTreeBuilder`] (the **accessibility** stage, spec §7.1);
    /// 7. derives the backend-neutral [`RenderGraphSummary`] from the scene and
    ///    `viewport` (the **render-graph** stage).
    ///
    /// The rendered-**image** stage is backend-dependent and left absent (see
    /// [`with_image`](Self::with_image)).
    ///
    /// Because the coordinators are stateless and deterministic and the tree is
    /// unchanged between calls, capturing the same frame twice yields equal stage
    /// data. The call never panics.
    #[must_use]
    pub fn capture(tree: &RetainedTree, inputs: &StyleInputs, viewport: Size) -> Self {
        let tree_snapshot = tree.snapshot();

        let style = StyleCoordinator::new();
        let computed = style.resolve(tree, inputs, viewport);

        // The scene coordinator owns the paragraph orchestrator; build it first so
        // the layout pass shapes and wraps text leaves through the same
        // orchestrator the scene later re-wraps and paints with (so box height and
        // painted line count agree, deterministically).
        let scene_coordinator = SceneCoordinator::new();

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

    /// Returns the captured retained-tree snapshot.
    #[must_use]
    pub fn tree(&self) -> &TreeSnapshot {
        &self.tree
    }

    /// Returns the captured per-node computed styles.
    #[must_use]
    pub fn computed(&self) -> &ComputedStyles {
        &self.computed
    }

    /// Returns the captured per-node absolute layout boxes.
    #[must_use]
    pub fn layout(&self) -> &LayoutBoxes {
        &self.layout
    }

    /// Returns the captured scene snapshot.
    #[must_use]
    pub fn scene(&self) -> &SceneSnapshot {
        &self.scene
    }

    /// Returns the captured paint-command stream (spec §7.4).
    #[must_use]
    pub fn paint(&self) -> &DisplayList {
        &self.paint
    }

    /// Returns the captured accessibility (semantic) snapshot (spec §7.1, §7.4).
    #[must_use]
    pub fn accessibility(&self) -> &SemanticSnapshot {
        &self.accessibility
    }

    /// Returns the captured backend-neutral render-graph description (spec §7.4).
    #[must_use]
    pub fn render_graph(&self) -> &RenderGraphSummary {
        &self.render_graph
    }

    /// Returns the captured rendered image, if a backend has attached one
    /// (spec §7.4).
    #[must_use]
    pub fn image(&self) -> Option<&ImageSnapshot> {
        self.image.as_ref()
    }

    /// Returns the number of nodes captured in the retained-tree snapshot.
    #[must_use]
    pub fn node_count(&self) -> usize {
        self.tree.len()
    }

    /// Returns the number of paint commands (fills) in the composed scene's
    /// display list.
    #[must_use]
    pub fn fill_count(&self) -> usize {
        self.scene.display_list().len()
    }

    /// Returns whether the composed scene's display list is balanced, i.e. a
    /// well-formed stream a renderer can consume.
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
        host.render(&view, &StyleInputs::new());
        let child = {
            let tree = host.tree();
            let root = tree.root().expect("root");
            tree.get(root).unwrap().children().first().copied()
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
        host.render(&view, &inputs);

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
        host.render(&view, &StyleInputs::new());

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
        let with = snapshot.with_image(image);
        assert_eq!(with.image(), Some(&image));
    }
}
