use obscur_identity::{FastMap, NodeId};
use obscur_view::view::ViewNode;

use crate::tree::{RetainedTree, Subtree};

use super::{IdentityMatch, ReconciliationOp, ReconciliationPlan};

/// Retains a sibling group's progress while its current child's descendants are processed.
pub(super) struct SiblingReconciliation<'a> {
    pub(super) parent: NodeId,
    pub(super) desired: std::slice::Iter<'a, ViewNode>,
    pub(super) matches: std::vec::IntoIter<IdentityMatch>,
    pub(super) removed: Vec<NodeId>,
    pub(super) old_rank: FastMap<NodeId, usize>,
    pub(super) new_children: Vec<NodeId>,
    pub(super) reused_order: Vec<NodeId>,
}

impl SiblingReconciliation<'_> {
    /// Commits sibling order and removes unmatched subtrees after processing the desired children.
    pub(super) fn finish(self, tree: &mut RetainedTree, plan: &mut ReconciliationPlan) {
        // Record moves for reused nodes whose relative order changed. Nodes that
        // stay in increasing old-rank order are left in place (no-op); the rest
        // are repositioned to their final index.
        let mut last_rank: Option<usize> = None;
        for &id in &self.reused_order {
            let Some(&rank) = self.old_rank.get(&id) else {
                continue;
            };
            let in_order = last_rank.is_none_or(|last| rank > last);
            if in_order {
                last_rank = Some(rank);
            } else if let Some(index) = self.new_children.iter().position(|&c| c == id) {
                plan.push(ReconciliationOp::Move { id, index });
            }
        }

        // Commit the parent's child order to the desired order, then refresh the
        // element-only projection the component tree exposes.
        if let Some(parent_node) = tree.get_mut(self.parent) {
            parent_node.children = self.new_children;
        }
        tree.sync_element_children(self.parent);

        // Remove every unmatched existing child together with its subtree,
        // leaves before parents so a parent is never removed before its children.
        for &root in &self.removed {
            let mut ids = Subtree::new(root).descendants(tree);
            ids.reverse();
            for id in ids {
                // Captured before the removal: afterwards the node is gone and its
                // parent is unreachable, which is exactly what forced consumers to
                // widen to a full restyle.
                let parent = tree.get(id).and_then(|node| node.logical_parent().parent());
                if tree.remove(id).is_some() {
                    plan.push(ReconciliationOp::Remove { id, parent });
                }
            }
        }
    }
}
