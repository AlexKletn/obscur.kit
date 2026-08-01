#![doc = include_str!("tree_commit.doc/_module.md")]

use obscur_identity::{NodeId, VersionStamp};

#[doc = include_str!("tree_commit.doc/TreeCommit.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeCommit {
    #[doc = include_str!("tree_commit.doc/base_version.md")]
    base_version: VersionStamp,
    #[doc = include_str!("tree_commit.doc/committed_version.md")]
    committed_version: VersionStamp,
    #[doc = include_str!("tree_commit.doc/created.md")]
    created: Vec<NodeId>,
    #[doc = include_str!("tree_commit.doc/removed.md")]
    removed: Vec<NodeId>,
    #[doc = include_str!("tree_commit.doc/moved.md")]
    moved: Vec<NodeId>,
    #[doc = include_str!("tree_commit.doc/updated.md")]
    updated: Vec<NodeId>,
    #[doc = include_str!("tree_commit.doc/new_root.md")]
    new_root: Option<NodeId>,
}

impl TreeCommit {
    #[doc = include_str!("tree_commit.doc/new.md")]
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

    #[doc = include_str!("tree_commit.doc/base_version_2.md")]
    #[must_use]
    pub fn base_version(&self) -> VersionStamp {
        self.base_version
    }

    #[doc = include_str!("tree_commit.doc/committed_version_2.md")]
    #[must_use]
    pub fn committed_version(&self) -> VersionStamp {
        self.committed_version
    }

    #[doc = include_str!("tree_commit.doc/created_2.md")]
    #[must_use]
    pub fn created(&self) -> &[NodeId] {
        &self.created
    }

    #[doc = include_str!("tree_commit.doc/removed_2.md")]
    #[must_use]
    pub fn removed(&self) -> &[NodeId] {
        &self.removed
    }

    #[doc = include_str!("tree_commit.doc/moved_2.md")]
    #[must_use]
    pub fn moved(&self) -> &[NodeId] {
        &self.moved
    }

    #[doc = include_str!("tree_commit.doc/updated_2.md")]
    #[must_use]
    pub fn updated(&self) -> &[NodeId] {
        &self.updated
    }

    #[doc = include_str!("tree_commit.doc/new_root_2.md")]
    #[must_use]
    pub fn new_root(&self) -> Option<NodeId> {
        self.new_root
    }

    #[doc = include_str!("tree_commit.doc/resolve_new.md")]
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
