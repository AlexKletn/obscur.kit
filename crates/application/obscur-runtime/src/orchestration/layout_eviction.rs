#![doc = include_str!("../../docs/api/orchestration/layout_eviction.doc/_module.md")]

use std::collections::HashSet;

use obscur_identity::{FastSet, NodeId};
use obscur_layout::invalidation::LayoutInvalidation;
use obscur_tree::tree::RetainedTree;

use super::layout_topology::LayoutTopology;

#[doc = include_str!("../../docs/api/orchestration/layout_eviction.doc/LayoutEviction.md")]
pub(super) struct LayoutEviction {
    /// The nodes whose memoized `measure` can differ (see
    /// [`invalidated_layout`]'s "What `measure(N, c)` actually reads").
    pub(super) measures: FastSet<NodeId>,
    #[doc = include_str!("../../docs/api/orchestration/layout_eviction.doc/arranges.md")]
    pub(super) arranges: FastSet<NodeId>,
}

#[doc = include_str!("../../docs/api/orchestration/layout_eviction.doc/invalidated_layout.md")]
pub(super) fn invalidated_layout(
    tree: &RetainedTree,
    dirty: &LayoutInvalidation,
) -> Option<LayoutEviction> {
    if dirty.is_empty() {
        return Some(LayoutEviction {
            measures: HashSet::default(),
            arranges: HashSet::default(),
        });
    }
    // No root: nothing was arranged, so nothing retained can be trusted.
    let root = tree.root()?;
    // The root's `font-size` is the `rem` base of every node's lengths and text
    // metrics, so a change to it can move any box in the tree. Unboundable.
    if dirty.is_invalidated(root) {
        return None;
    }

    LayoutTopology::new(tree, root).evictions(dirty)
}
