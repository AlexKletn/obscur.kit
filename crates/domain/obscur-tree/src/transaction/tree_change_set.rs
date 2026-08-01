#![doc = include_str!("tree_change_set.doc/_module.md")]

use super::TreeChange;

#[doc = include_str!("tree_change_set.doc/TreeChangeSet.md")]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TreeChangeSet {
    #[doc = include_str!("tree_change_set.doc/changes.md")]
    changes: Vec<TreeChange>,
    #[doc = include_str!("tree_change_set.doc/creations.md")]
    creations: u32,
}

impl TreeChangeSet {
    #[doc = include_str!("tree_change_set.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[doc = include_str!("tree_change_set.doc/push.md")]
    pub(crate) fn push(&mut self, change: TreeChange) {
        if matches!(change, TreeChange::CreateNode { .. }) {
            self.creations = self.creations.saturating_add(1);
        }
        self.changes.push(change);
    }

    #[doc = include_str!("tree_change_set.doc/changes_2.md")]
    #[must_use]
    pub fn changes(&self) -> &[TreeChange] {
        &self.changes
    }

    #[doc = include_str!("tree_change_set.doc/iter.md")]
    pub fn iter(&self) -> impl Iterator<Item = &TreeChange> {
        self.changes.iter()
    }

    #[doc = include_str!("tree_change_set.doc/len.md")]
    #[must_use]
    pub fn len(&self) -> usize {
        self.changes.len()
    }

    #[doc = include_str!("tree_change_set.doc/is_empty.md")]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }

    #[doc = include_str!("tree_change_set.doc/creation_count.md")]
    #[must_use]
    pub fn creation_count(&self) -> u32 {
        self.creations
    }

    #[doc = include_str!("tree_change_set.doc/into_changes.md")]
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
