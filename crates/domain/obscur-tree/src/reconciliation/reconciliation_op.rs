//! `ReconciliationOp`: a normalized structural operation from a reconcile pass.

use obscur_identity::NodeId;

#[doc = include_str!("../../docs/api/reconciliation/reconciliation_op.doc/ReconciliationOp.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReconciliationOp {
    /// A newly created node placed under `parent` at child position `index`.
    Insert {
        /// The freshly minted node's id.
        id: NodeId,
        /// The parent the node was inserted under.
        parent: NodeId,
        /// The final child index of the node within `parent`.
        index: usize,
    },
    /// An existing node (and, transitively, its subtree) removed from the tree.
    Remove {
        /// The removed node's id.
        id: NodeId,
        #[doc = include_str!("../../docs/api/reconciliation/reconciliation_op.doc/parent_2.md")]
        parent: Option<NodeId>,
    },
    /// A reused node repositioned to child `index` under its parent (a reorder
    /// that preserves the node's identity).
    Move {
        /// The reused node's id (unchanged across the move).
        id: NodeId,
        /// The node's new child index within its parent.
        index: usize,
    },
    /// A reused node whose content changed in place (for example, updated text
    /// content), keeping its identity.
    Update {
        /// The reused node's id.
        id: NodeId,
    },
}

impl ReconciliationOp {
    /// Returns the primary node id this op acts on.
    #[must_use]
    pub const fn id(self) -> NodeId {
        match self {
            Self::Insert { id, .. }
            | Self::Remove { id, .. }
            | Self::Move { id, .. }
            | Self::Update { id } => id,
        }
    }

    /// Returns `true` if this op creates a node.
    #[must_use]
    pub const fn is_insert(self) -> bool {
        matches!(self, Self::Insert { .. })
    }

    /// Returns `true` if this op removes a node.
    #[must_use]
    pub const fn is_remove(self) -> bool {
        matches!(self, Self::Remove { .. })
    }

    /// Returns `true` if this op repositions a reused node.
    #[must_use]
    pub const fn is_move(self) -> bool {
        matches!(self, Self::Move { .. })
    }

    /// Returns `true` if this op updates a reused node in place.
    #[must_use]
    pub const fn is_update(self) -> bool {
        matches!(self, Self::Update { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn id_and_predicates() {
        let id = NodeId::from_raw(1);
        let parent = NodeId::from_raw(2);

        let insert = ReconciliationOp::Insert {
            id,
            parent,
            index: 0,
        };
        assert!(insert.is_insert());
        assert_eq!(insert.id(), id);

        let remove = ReconciliationOp::Remove { id, parent: None };
        assert!(remove.is_remove());
        assert_eq!(remove.id(), id);

        let moved = ReconciliationOp::Move { id, index: 3 };
        assert!(moved.is_move());
        assert_eq!(moved.id(), id);

        let update = ReconciliationOp::Update { id };
        assert!(update.is_update());
        assert_eq!(update.id(), id);
    }
}
