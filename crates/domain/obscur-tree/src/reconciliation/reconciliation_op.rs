#![doc = include_str!("reconciliation_op.doc/_module.md")]

use obscur_identity::NodeId;

#[doc = include_str!("reconciliation_op.doc/ReconciliationOp.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReconciliationOp {
    #[doc = include_str!("reconciliation_op.doc/Insert.md")]
    Insert {
        #[doc = include_str!("reconciliation_op.doc/id.md")]
        id: NodeId,
        #[doc = include_str!("reconciliation_op.doc/parent.md")]
        parent: NodeId,
        #[doc = include_str!("reconciliation_op.doc/index.md")]
        index: usize,
    },
    #[doc = include_str!("reconciliation_op.doc/Remove.md")]
    Remove {
        #[doc = include_str!("reconciliation_op.doc/id_2.md")]
        id: NodeId,
        #[doc = include_str!("reconciliation_op.doc/parent_2.md")]
        parent: Option<NodeId>,
    },
    #[doc = include_str!("reconciliation_op.doc/Move.md")]
    Move {
        #[doc = include_str!("reconciliation_op.doc/id_3.md")]
        id: NodeId,
        #[doc = include_str!("reconciliation_op.doc/index_2.md")]
        index: usize,
    },
    #[doc = include_str!("reconciliation_op.doc/Update.md")]
    Update {
        #[doc = include_str!("reconciliation_op.doc/id_4.md")]
        id: NodeId,
    },
}

impl ReconciliationOp {
    #[doc = include_str!("reconciliation_op.doc/id_5.md")]
    #[must_use]
    pub const fn id(self) -> NodeId {
        match self {
            Self::Insert { id, .. }
            | Self::Remove { id, .. }
            | Self::Move { id, .. }
            | Self::Update { id } => id,
        }
    }

    #[doc = include_str!("reconciliation_op.doc/is_insert.md")]
    #[must_use]
    pub const fn is_insert(self) -> bool {
        matches!(self, Self::Insert { .. })
    }

    #[doc = include_str!("reconciliation_op.doc/is_remove.md")]
    #[must_use]
    pub const fn is_remove(self) -> bool {
        matches!(self, Self::Remove { .. })
    }

    #[doc = include_str!("reconciliation_op.doc/is_move.md")]
    #[must_use]
    pub const fn is_move(self) -> bool {
        matches!(self, Self::Move { .. })
    }

    #[doc = include_str!("reconciliation_op.doc/is_update.md")]
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
