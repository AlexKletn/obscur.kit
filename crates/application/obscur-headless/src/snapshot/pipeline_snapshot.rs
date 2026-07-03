//! An immutable capture of the frame pipeline stages: [`PipelineSnapshot`].

use obscur_geometry::Size;
use obscur_runtime::orchestration::{
    ComputedStyles, LayoutBoxes, LayoutCoordinator, SceneCoordinator, StyleCoordinator, StyleInputs,
};
use obscur_scene::scene::{Scene, SceneSnapshot};
use obscur_tree::tree::{RetainedTree, TreeSnapshot};

/// An immutable, backend-independent capture of every frame-pipeline stage for
/// one frame, for golden testing (spec §7.14).
///
/// A `PipelineSnapshot` bundles the four backend-independent stages the spec
/// prefers for golden diffs (§7.14): the retained [`TreeSnapshot`], the resolved
/// [`ComputedStyles`], the arranged [`LayoutBoxes`], and the composed
/// [`SceneSnapshot`]. It is the concrete artefact behind the Phase-1 slice proof
/// that `component → desired tree → retained tree → style → layout → scene` runs
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
}

impl PipelineSnapshot {
    /// Builds a snapshot directly from already-captured stage data.
    ///
    /// Prefer [`capture`](Self::capture), which derives every stage from a
    /// retained tree in one call; this constructor exists for callers that
    /// already hold the individual stage artefacts.
    #[must_use]
    pub fn new(
        tree: TreeSnapshot,
        computed: ComputedStyles,
        layout: LayoutBoxes,
        scene: SceneSnapshot,
    ) -> Self {
        Self {
            tree,
            computed,
            layout,
            scene,
        }
    }

    /// Captures every pipeline stage from the current state of `tree`,
    /// deterministically (spec §5.13, §7.14).
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
    ///    in a fresh [`Scene`], and snapshots it.
    ///
    /// Because the coordinators are stateless and deterministic and the tree is
    /// unchanged between calls, capturing the same frame twice yields equal stage
    /// data. The call never panics.
    #[must_use]
    pub fn capture(tree: &RetainedTree, inputs: &StyleInputs, viewport: Size) -> Self {
        let tree_snapshot = tree.snapshot();

        let style = StyleCoordinator::new();
        let computed = style.resolve(tree, inputs);

        let layout = LayoutCoordinator::new();
        let layout_styles = layout.layout_styles_from(&computed);
        let boxes = layout.layout(tree, &layout_styles, viewport);

        let scene_coordinator = SceneCoordinator::new();
        let display_list = scene_coordinator.compose(tree, &computed, &boxes);
        let mut scene = Scene::new();
        scene.set_root(display_list);
        let scene_snapshot = scene.snapshot();

        Self::new(tree_snapshot, computed, boxes, scene_snapshot)
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
}
