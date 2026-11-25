//! The synchronous, deterministic frame driver: [`FramePipeline`].

use obscur_scene::scene::Scene;
use obscur_style::matching::{MediaEnvironment, Stylesheet};
use obscur_tree::reconciliation::Reconciler;
use obscur_tree::tree::RetainedTree;
use obscur_view::view::View;

use crate::orchestration::{
    LayoutCoordinator, SceneCoordinator, ShapedChMetrics, StyleCoordinator, StyleInputs,
    StyleMatchingCoordinator, root_font_size,
};
use crate::transaction::ReconcileEnactor;

use super::{FrameContext, FrameResult};

#[doc = include_str!("../../docs/api/frame/frame_pipeline.doc/FramePipeline.md")]
// Not `Clone`: it owns a `SceneCoordinator`, which holds a `Box<dyn TextShaper>`
// (a replaceable, non-cloneable shaping port) and so cannot be cloned.
#[derive(Debug, Default)]
pub struct FramePipeline {
    /// Turns the desired view tree into retained-tree structure.
    reconciler: Reconciler,
    /// Resolves computed styles over the retained tree.
    style: StyleCoordinator,
    /// Drives layout over the retained tree.
    layout: LayoutCoordinator,
    /// Composes the paint-ordered display list.
    scene: SceneCoordinator,
}

impl FramePipeline {
    /// Creates a pipeline with fresh coordinators and a fresh reconciler.
    #[must_use]
    pub fn new() -> Self {
        Self {
            reconciler: Reconciler::new(),
            style: StyleCoordinator::new(),
            layout: LayoutCoordinator::new(),
            scene: SceneCoordinator::new(),
        }
    }

    #[doc = include_str!("../../docs/api/frame/frame_pipeline.doc/run.md")]
    #[must_use]
    pub fn run(
        &self,
        tree: &mut RetainedTree,
        scene: &mut Scene,
        ctx: FrameContext,
        desired: &View,
        inputs: &StyleInputs,
    ) -> FrameResult {
        // 1. Reconcile: update the retained tree through the transaction commit
        //    boundary (spec §3.2) rather than mutating it directly.
        let reconciled = self.commit_reconcile(tree, desired);
        self.finish_frame(tree, scene, ctx, reconciled, inputs)
    }

    #[doc = include_str!("../../docs/api/frame/frame_pipeline.doc/run_matched.md")]
    #[must_use]
    pub fn run_matched(
        &self,
        tree: &mut RetainedTree,
        scene: &mut Scene,
        ctx: FrameContext,
        desired: &View,
        sheets: &[Stylesheet],
    ) -> FrameResult {
        // 1. Reconcile: update the retained tree through the transaction commit
        //    boundary (spec §3.2) rather than mutating it directly.
        let reconciled = self.commit_reconcile(tree, desired);

        // 2. Match: build cascade inputs from the sheets against the fresh tree,
        //    gating `@media` rules by this frame's viewport (§4.13).
        let environment = MediaEnvironment::viewport(ctx.viewport().width, ctx.viewport().height);
        let inputs = StyleMatchingCoordinator::new().build_inputs_env(tree, sheets, &environment);
        self.finish_frame(tree, scene, ctx, reconciled, &inputs)
    }

    #[doc = include_str!("../../docs/api/frame/frame_pipeline.doc/commit_reconcile.md")]
    fn commit_reconcile(&self, tree: &mut RetainedTree, desired: &View) -> bool {
        let (commit, _plan) = ReconcileEnactor::new()
            .commit_root(&self.reconciler, tree, desired)
            .expect("a root reconcile is a single, structurally-valid ReplaceRoot");
        // This pipeline drives no component instances, so the structural deltas
        // are not needed here — only whether anything changed.
        !commit.is_empty()
    }

    #[doc = include_str!("../../docs/api/frame/frame_pipeline.doc/finish_frame.md")]
    fn finish_frame(
        &self,
        tree: &mut RetainedTree,
        scene: &mut Scene,
        ctx: FrameContext,
        reconciled: bool,
        inputs: &StyleInputs,
    ) -> FrameResult {
        // Style: resolve computed styles over the retained tree. The scene's
        // paragraph orchestrator comes along as the `ch` port, so `ch` folds to
        // pixels here with every other relative unit rather than being left for
        // layout to measure and paint to read as `0`. It is the same shaper the
        // measure and compose passes below use (`self.style` and `self.scene` are
        // disjoint fields, so this is a legal split borrow), so a `20ch` box, the
        // text in it, and the pixels for both agree on one font.
        let ch_metrics = ShapedChMetrics::new(self.scene.paragraph());
        let computed = self.style.resolve(
            tree,
            inputs,
            ctx.viewport(),
            ctx.environment(),
            Some(&ch_metrics),
        );
        let restyled = !computed.is_empty();

        // Layout: map styles into layout styles and arrange the tree. The root
        // `font-size` supplies the `rem` base; the viewport supplies `vw`/`vh`;
        // the scene's paragraph orchestrator supplies `ch` by shaping the `0`
        // glyph — for the rare node that writes `ch`, and no other. It is the
        // same shaper the measure pass below uses (`self.layout` and `self.scene`
        // are disjoint fields, so this is a legal split borrow), so a `20ch` box
        // and the text placed in it are measured against one font.
        let root_font = root_font_size(&computed, tree.root());
        let mut layout_styles = self.layout.layout_styles_from(
            &computed,
            ctx.viewport(),
            root_font,
            Some(self.scene.paragraph()),
        );
        // Named-area placement (`grid-area: <name>` against the container's
        // `grid-template-areas`) needs the tree to resolve, so it patches the
        // freshly built styles here; items with no named area are left untouched.
        self.layout
            .resolve_named_grid_areas(&mut layout_styles, tree, &computed);
        // Named grid *lines* (`grid-column: <name>` against the container's
        // `[name]` track-list lines) resolve the same way — after the styles table
        // is built and against the parent's template; items with no named line are
        // left untouched.
        self.layout
            .resolve_named_grid_lines(&mut layout_styles, tree, &computed);
        // Authored grid templates (`grid-template-columns`/`-rows`) become a
        // per-node side-table so a grid container dispatches the real track-sizing
        // algorithm; a grid node with no template falls back to a single column.
        let mut grid_templates = self.layout.grid_templates_from(
            &computed,
            ctx.viewport(),
            root_font,
            Some(self.scene.paragraph()),
        );
        crate::orchestration::defer_repeated_grid_lines(&mut grid_templates, tree, &computed);
        // Text leaves are shaped + wrapped to their available width inside the
        // layout pass through the scene's paragraph orchestrator (disjoint field,
        // so a legal split borrow), so `self.scene` later re-wraps and paints
        // identical lines.
        let boxes = self.layout.layout(
            tree,
            &layout_styles,
            &computed,
            &grid_templates,
            ctx.viewport(),
            // No live scroll system yet: the viewport is unscrolled, so every
            // `position: sticky` node lays out at its static position.
            (0.0, 0.0),
            self.scene.paragraph(),
        );
        let relaid_out = !boxes.is_empty();

        // Compose: build the display list, install it, and publish a snapshot.
        // The stateless pipeline owns no editing session, so it never paints a
        // caret or selection highlight (`None`).
        let display_list = self.scene.compose(tree, &computed, &boxes, None);
        scene.set_root(display_list);
        let snapshot = scene.snapshot();
        let composed = true;

        FrameResult::new(
            ctx.id(),
            snapshot,
            reconciled,
            restyled,
            relaid_out,
            composed,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::{FrameId, FrameReason};
    use obscur_geometry::Size;
    use obscur_identity::{ComponentId, NodeId};
    use obscur_scene::command::DisplayList;
    use obscur_style::cascade::ResolvedDeclarations;
    use obscur_style::property::{Length, PropertyId, PropertyValue, StyleColor};
    use obscur_view::view::{ViewFragment, ViewNode};

    /// A small view: two top-level element children. Reconciled into the
    /// retained tree, the synthetic root node gains these as its two children.
    fn sample_view() -> View {
        View::from_fragment(ViewFragment::from_nodes(vec![
            ViewNode::element(ComponentId::from_raw(10)),
            ViewNode::element(ComponentId::from_raw(11)),
        ]))
    }

    fn ctx(id: u64) -> FrameContext {
        FrameContext::new(
            FrameId::from_raw(id),
            FrameReason::UiCommit,
            Size::new(300.0, 300.0),
        )
    }

    #[test]
    fn runs_the_whole_slice_and_is_deterministic() {
        let pipeline = FramePipeline::new();
        let view = sample_view();
        let inputs = StyleInputs::new();

        let mut tree = RetainedTree::new();
        let mut scene = Scene::new();

        // First frame populates the retained tree from the desired tree.
        let first = pipeline.run(&mut tree, &mut scene, ctx(0), &view, &inputs);
        assert!(first.reconciled(), "first frame populates the tree");
        assert!(first.restyled(), "nodes resolve computed styles");
        assert!(first.composed());
        assert!(
            first.is_balanced(),
            "the published display list is balanced"
        );
        let first_commands: Vec<_> = first.snapshot().display_list().commands().to_vec();

        // Second frame with the same inputs is idempotent and deterministic:
        // reconcile preserves identity, so the display list is unchanged.
        let second = pipeline.run(&mut tree, &mut scene, ctx(1), &view, &inputs);
        assert!(second.is_balanced());
        let second_commands: Vec<_> = second.snapshot().display_list().commands().to_vec();
        assert_eq!(
            first_commands, second_commands,
            "same (tree, desired, inputs, viewport) yields the same display list"
        );
    }

    /// Collects the root id plus its direct children ids from the retained tree.
    fn root_and_children(tree: &RetainedTree) -> (NodeId, Vec<NodeId>) {
        let root = tree.root().expect("tree has a root after reconcile");
        let children = tree
            .get(root)
            .map(|node| node.child_nodes().to_vec())
            .unwrap_or_default();
        (root, children)
    }

    #[test]
    fn styled_background_flows_end_to_end() {
        let pipeline = FramePipeline::new();
        let view = sample_view();

        let mut tree = RetainedTree::new();
        let mut scene = Scene::new();

        // Frame 1: empty inputs, just to populate the retained tree and learn ids.
        let empty_inputs = StyleInputs::new();
        let _ = pipeline.run(&mut tree, &mut scene, ctx(0), &view, &empty_inputs);

        let (_root, children) = root_and_children(&tree);
        assert_eq!(children.len(), 2, "root reconciled two element children");
        let styled_child = children[0];

        // Give the chosen child a red background and an explicit 100x50 size.
        let mut decls = ResolvedDeclarations::new();
        decls.insert(
            PropertyId::BackgroundColor,
            PropertyValue::Color(StyleColor::rgba(1.0, 0.0, 0.0, 1.0)),
        );
        decls.insert(PropertyId::Width, PropertyValue::Length(Length::px(100.0)));
        decls.insert(PropertyId::Height, PropertyValue::Length(Length::px(50.0)));

        let mut inputs = StyleInputs::new();
        inputs.set_cascaded(styled_child, decls);

        // Frame 2: same view (reconcile is idempotent, ids are stable) + styles.
        let styled = pipeline.run(&mut tree, &mut scene, ctx(1), &view, &inputs);

        assert!(styled.restyled());
        assert!(styled.relaid_out());
        assert!(styled.composed());

        // The styled child produced a background fill: the list is non-empty and
        // balanced. This exercises View -> reconcile -> style -> layout -> scene
        // across five crates.
        let list: &DisplayList = styled.snapshot().display_list();
        assert!(!list.is_empty(), "styled child produced a background fill");
        assert!(styled.is_balanced());
    }

    /// A view whose single top-level element carries the `counter` class.
    fn counter_view() -> View {
        use obscur_view::view::{CLASS, ViewProperty, ViewValue};
        View::from_fragment(ViewFragment::from_nodes(vec![
            ViewNode::element(ComponentId::from_raw(10))
                .with_prop(ViewProperty::new(CLASS, ViewValue::Text("counter".into()))),
        ]))
    }

    /// A stylesheet: `.counter { background-color: red; width: 100px; height: 50px }`.
    fn counter_stylesheet() -> obscur_style::matching::Stylesheet {
        use obscur_style::cascade::CascadeOrigin;
        use obscur_style::matching::{BlockDeclaration, StyleRule, Stylesheet};
        use obscur_style::selector::{
            ComplexSelector, CompoundSelector, SelectorList, SimpleSelector,
        };

        let selectors = SelectorList::new(vec![ComplexSelector::new(
            CompoundSelector::new(vec![SimpleSelector::Class("counter".into())]),
            Vec::new(),
        )]);
        Stylesheet::new(
            CascadeOrigin::GlobalApp,
            vec![StyleRule::new(
                selectors,
                vec![
                    BlockDeclaration::normal(
                        PropertyId::BackgroundColor,
                        PropertyValue::Color(StyleColor::rgba(1.0, 0.0, 0.0, 1.0)),
                    ),
                    BlockDeclaration::normal(
                        PropertyId::Width,
                        PropertyValue::Length(Length::px(100.0)),
                    ),
                    BlockDeclaration::normal(
                        PropertyId::Height,
                        PropertyValue::Length(Length::px(50.0)),
                    ),
                ],
            )],
        )
    }

    #[test]
    fn run_matched_paints_matched_background() {
        // The whole point of the feature: authored CSS reaches paint. Matching a
        // `.counter { background-color … }` stylesheet against a `.counter`
        // element must yield a NON-EMPTY display list, whereas no stylesheet
        // yields an empty one.
        let pipeline = FramePipeline::new();
        let view = counter_view();
        let sheet = counter_stylesheet();

        // With the stylesheet: the matched element paints a background fill.
        let mut tree = RetainedTree::new();
        let mut scene = Scene::new();
        let styled = pipeline.run_matched(
            &mut tree,
            &mut scene,
            ctx(0),
            &view,
            std::slice::from_ref(&sheet),
        );
        assert!(styled.reconciled());
        assert!(styled.restyled(), "the matched node resolved styles");
        assert!(styled.relaid_out(), "the sized node arranged a box");
        let styled_list: &DisplayList = styled.snapshot().display_list();
        assert!(
            !styled_list.is_empty(),
            "matched background produced a non-empty display list"
        );
        assert!(styled.is_balanced());
        let fill_count = styled_list.commands().len();
        assert!(fill_count >= 1, "at least one paint command was emitted");

        // Without any stylesheet: nothing matches, the display list is empty.
        let mut bare_tree = RetainedTree::new();
        let mut bare_scene = Scene::new();
        let bare = pipeline.run_matched(&mut bare_tree, &mut bare_scene, ctx(0), &view, &[]);
        assert!(
            bare.snapshot().display_list().is_empty(),
            "no stylesheet means no fill"
        );
    }

    #[test]
    fn run_matched_matches_run_with_equivalent_inputs() {
        // run_matched with empty sheets is behaviourally identical to run with
        // empty inputs: same reconcile, same empty style, same composition.
        let pipeline = FramePipeline::new();
        let view = sample_view();

        let mut a_tree = RetainedTree::new();
        let mut a_scene = Scene::new();
        let via_run = pipeline.run(
            &mut a_tree,
            &mut a_scene,
            ctx(0),
            &view,
            &StyleInputs::new(),
        );

        let mut b_tree = RetainedTree::new();
        let mut b_scene = Scene::new();
        let via_matched = pipeline.run_matched(&mut b_tree, &mut b_scene, ctx(0), &view, &[]);

        let run_commands: Vec<_> = via_run.snapshot().display_list().commands().to_vec();
        let matched_commands: Vec<_> = via_matched.snapshot().display_list().commands().to_vec();
        assert_eq!(run_commands, matched_commands);
        assert_eq!(via_run.reconciled(), via_matched.reconciled());
    }
}
