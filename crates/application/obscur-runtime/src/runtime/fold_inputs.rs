use obscur_tree::tree::RetainedTree;

use crate::orchestration::{LayoutBoxes, ScrollContainer};

#[doc = include_str!("../../docs/api/runtime/scroll_offsets.doc/FoldInputs.md")]
#[derive(Clone, Copy)]
pub(crate) struct FoldInputs<'a> {
    /// The retained tree, walked parent-ward to sum a node's scroll chain.
    pub(crate) tree: &'a RetainedTree,
    /// The absolute layout boxes of the last produced frame, in content space.
    pub(crate) boxes: &'a LayoutBoxes,
    /// Every currently-overflowing scroll container, as
    /// [`scroll_containers`](crate::orchestration::scroll_containers) resolved
    /// them for this same tree/styles/boxes.
    pub(crate) containers: &'a [ScrollContainer],
}
