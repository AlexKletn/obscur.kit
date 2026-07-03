//! The synchronous, deterministic frame driver: [`FramePipeline`].

use obscur_scene::scene::Scene;
use obscur_tree::reconciliation::Reconciler;
use obscur_tree::tree::RetainedTree;
use obscur_view::view::View;

use crate::orchestration::{LayoutCoordinator, SceneCoordinator, StyleCoordinator, StyleInputs};

use super::{FrameContext, FrameResult};

/// The synchronous, deterministic driver that runs one frame's core pipeline
/// (spec §3.4).
///
/// The pipeline is **orchestration glue only**: it owns a [`Reconciler`] and the
/// three stateless coordinators and calls them in a fixed order. It reimplements
/// no domain logic — reconciliation, the cascade, layout, and scene composition
/// each live entirely in their own crates.
///
/// Cross-frame state is *not* held here. The retained [`RetainedTree`] and the
/// mutable [`Scene`] are owned by the caller (the runtime) and passed into
/// [`run`](Self::run) by mutable reference, so the same pipeline value drives
/// every frame while the runtime keeps the evolving state.
///
/// # Phase-1 scope
///
/// [`run`](Self::run) executes the deterministic **core**: reconcile the desired
/// tree → resolve styles → layout → compose the scene → publish a snapshot. The
/// remaining §3.4 stages — input dispatch, motion sampling (pre- and
/// post-layout), render-graph construction, submission, and post-frame
/// notifications — are wired in later phases.
#[derive(Debug, Clone, Default)]
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

    /// Runs one synchronous frame over caller-owned state and returns its
    /// [`FrameResult`].
    ///
    /// The frame is short and **deterministic** (spec §3.4): for a given
    /// `(tree state, desired, inputs, viewport)` it always produces the same
    /// snapshot and the same flags. It never allocates cross-frame state of its
    /// own and never panics.
    ///
    /// The phases, in order:
    ///
    /// 1. **Reconcile** — the desired [`View`] is reconciled into `tree`,
    ///    updating retained structure, identity, and text while preserving node
    ///    ids across reorders. `reconciled` is set when the returned plan is
    ///    non-empty (the tree changed).
    /// 2. **Style** — computed styles are resolved for every node reachable from
    ///    the root, inheriting parent-first. `restyled` is set when any style was
    ///    resolved.
    /// 3. **Layout** — the computed styles are mapped to layout styles and the
    ///    tree is arranged under [`FrameContext::viewport`], producing absolute
    ///    boxes. `relaid_out` is set when any box was produced.
    /// 4. **Compose** — a paint-ordered display list is composed from the styled,
    ///    laid-out tree, installed as the scene root (advancing the scene
    ///    generation), and published as an immutable snapshot. `composed` is
    ///    always set: a scene is composed every frame, even if empty.
    ///
    /// Input dispatch, motion sampling, and render submission are not part of
    /// this core and are wired in later phases.
    #[must_use]
    pub fn run(
        &self,
        tree: &mut RetainedTree,
        scene: &mut Scene,
        ctx: FrameContext,
        desired: &View,
        inputs: &StyleInputs,
    ) -> FrameResult {
        // 1. Reconcile: update the retained tree from the desired tree.
        let plan = self.reconciler.reconcile_root(tree, desired);
        let reconciled = !plan.is_empty();

        // 2. Style: resolve computed styles over the retained tree.
        let computed = self.style.resolve(tree, inputs);
        let restyled = !computed.is_empty();

        // 3. Layout: map styles into layout styles and arrange the tree.
        let layout_styles = self.layout.layout_styles_from(&computed);
        let boxes = self.layout.layout(tree, &layout_styles, ctx.viewport());
        let relaid_out = !boxes.is_empty();

        // 4. Compose: build the display list, install it, and publish a snapshot.
        let display_list = self.scene.compose(tree, &computed, &boxes);
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
        assert!(first.is_balanced(), "the published display list is balanced");
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
            .map(|node| node.children().to_vec())
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
}
