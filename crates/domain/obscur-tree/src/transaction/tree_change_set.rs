//! `TreeChangeSet`: the ordered batch of staged tree changes.

use super::TreeChange;

#[doc = include_str!("../../docs/api/transaction/tree_change_set.doc/TreeChangeSet.md")]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TreeChangeSet {
    /// The staged operations, in application order.
    changes: Vec<TreeChange>,
    /// Number of [`CreateNode`](TreeChange::CreateNode) ops staged so far.
    creations: u32,
}

impl TreeChangeSet {
    /// Creates an empty change set.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[doc = include_str!("../../docs/api/transaction/tree_change_set.doc/push.md")]
    pub(crate) fn push(&mut self, change: TreeChange) {
        if matches!(change, TreeChange::CreateNode { .. }) {
            self.creations = self.creations.saturating_add(1);
        }
        self.changes.push(change);
    }

    /// Returns the staged changes in application order.
    #[must_use]
    pub fn changes(&self) -> &[TreeChange] {
        &self.changes
    }

    /// Returns an iterator over the staged changes in application order.
    pub fn iter(&self) -> impl Iterator<Item = &TreeChange> {
        self.changes.iter()
    }

    /// Returns the number of staged changes.
    #[must_use]
    pub fn len(&self) -> usize {
        self.changes.len()
    }

    /// Returns `true` if no changes have been staged.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }

    /// Returns the number of [`CreateNode`](TreeChange::CreateNode) operations
    /// staged, i.e. the number of distinct
    /// [`StagedRef::New`](super::StagedRef::New) indices in use.
    #[must_use]
    pub fn creation_count(&self) -> u32 {
        self.creations
    }

    /// Consumes the set, yielding the owned change list for application.
    pub(crate) fn into_changes(self) -> Vec<TreeChange> {
        self.changes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::NodeKind;
    use crate::transaction::StagedRef;
    use obscur_identity::NodeId;

    #[test]
    fn empty_by_default() {
        let set = TreeChangeSet::new();
        assert!(set.is_empty());
        assert_eq!(set.len(), 0);
        assert_eq!(set.creation_count(), 0);
    }

    #[test]
    fn push_tracks_creations_only() {
        let mut set = TreeChangeSet::new();
        set.push(TreeChange::CreateNode {
            kind: NodeKind::Root,
            parent: StagedRef::Existing(NodeId::from_raw(1)),
            index: 0,
        });
        set.push(TreeChange::RemoveSubtree {
            target: NodeId::from_raw(2),
        });
        assert_eq!(set.len(), 2);
        assert_eq!(set.creation_count(), 1);
        assert_eq!(set.iter().count(), 2);
    }
}
