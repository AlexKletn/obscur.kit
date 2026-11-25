//! `TreeCommit`: the recorded outcome of a successful transaction commit.

use obscur_identity::{NodeId, VersionStamp};

#[doc = include_str!("../../docs/api/transaction/tree_commit.doc/TreeCommit.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeCommit {
    /// The tree version captured at the start of commit, before any change was
    /// applied.
    base_version: VersionStamp,
    /// The tree version after the whole batch was applied.
    committed_version: VersionStamp,
    /// Nodes created, in creation order; index `i` is the concrete id for the
    /// `i`-th [`StagedRef::New`](super::StagedRef::New).
    created: Vec<NodeId>,
    /// Nodes removed (subtree removals), in the order they were removed.
    removed: Vec<NodeId>,
    /// Existing children that were repositioned.
    moved: Vec<NodeId>,
    /// Text nodes whose content was replaced.
    updated: Vec<NodeId>,
    /// The node designated as the new root, if the batch set one.
    new_root: Option<NodeId>,
}

impl TreeCommit {
    /// Builds a commit record. Constructed by
    /// [`TreeTransaction::commit`](super::TreeTransaction::commit) after a
    /// successful apply.
    #[must_use]
    pub(crate) fn new(
        base_version: VersionStamp,
        committed_version: VersionStamp,
        created: Vec<NodeId>,
        removed: Vec<NodeId>,
        moved: Vec<NodeId>,
        updated: Vec<NodeId>,
        new_root: Option<NodeId>,
    ) -> Self {
        Self {
            base_version,
            committed_version,
            created,
            removed,
            moved,
            updated,
            new_root,
        }
    }

    /// The tree version before the batch was applied.
    #[must_use]
    pub fn base_version(&self) -> VersionStamp {
        self.base_version
    }

    /// The tree version after the batch was applied.
    #[must_use]
    pub fn committed_version(&self) -> VersionStamp {
        self.committed_version
    }

    /// The nodes created by this commit, in creation order.
    #[must_use]
    pub fn created(&self) -> &[NodeId] {
        &self.created
    }

    /// The nodes removed by this commit.
    #[must_use]
    pub fn removed(&self) -> &[NodeId] {
        &self.removed
    }

    /// The existing children repositioned by this commit.
    #[must_use]
    pub fn moved(&self) -> &[NodeId] {
        &self.moved
    }

    /// The text nodes whose content was replaced by this commit.
    #[must_use]
    pub fn updated(&self) -> &[NodeId] {
        &self.updated
    }

    /// The node designated as the new root by this commit, if any.
    #[must_use]
    pub fn new_root(&self) -> Option<NodeId> {
        self.new_root
    }

    /// Resolves the `i`-th staged [`StagedRef::New`](super::StagedRef::New) to
    /// its concrete [`NodeId`], or `None` if no such creation was in the batch.
    #[must_use]
    pub fn resolve_new(&self, index: u32) -> Option<NodeId> {
        self.created.get(index as usize).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accessors_return_recorded_effects() {
        let base = VersionStamp::default();
        let after = base.bump();
        let created = vec![NodeId::from_raw(10), NodeId::from_raw(11)];
        let commit = TreeCommit::new(
            base,
            after,
            created.clone(),
            vec![NodeId::from_raw(20)],
            vec![NodeId::from_raw(30)],
            vec![NodeId::from_raw(40)],
            Some(NodeId::from_raw(10)),
        );
        assert_eq!(commit.base_version(), base);
        assert_eq!(commit.committed_version(), after);
        assert_eq!(commit.created(), created.as_slice());
        assert_eq!(commit.removed(), &[NodeId::from_raw(20)]);
        assert_eq!(commit.moved(), &[NodeId::from_raw(30)]);
        assert_eq!(commit.updated(), &[NodeId::from_raw(40)]);
        assert_eq!(commit.new_root(), Some(NodeId::from_raw(10)));
        assert_eq!(commit.resolve_new(0), Some(NodeId::from_raw(10)));
        assert_eq!(commit.resolve_new(1), Some(NodeId::from_raw(11)));
        assert_eq!(commit.resolve_new(2), None);
    }
}
