//! `StagedRef`: a transaction-local reference to a node.

use obscur_identity::NodeId;

#[doc = include_str!("../../docs/api/transaction/staged_ref.doc/StagedRef.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StagedRef {
    /// A node that already exists in the live tree, addressed by its `NodeId`.
    Existing(NodeId),
    /// The *i*-th node staged for creation in this transaction (0-based, in
    /// stage order). Resolves to a concrete `NodeId` only at commit.
    New(u32),
}

impl StagedRef {
    /// Creates a reference to an already-live node.
    #[must_use]
    pub const fn existing(id: NodeId) -> Self {
        Self::Existing(id)
    }

    /// Creates a placeholder for the `index`-th node staged for creation.
    #[must_use]
    pub const fn new(index: u32) -> Self {
        Self::New(index)
    }

    /// Returns `true` if this refers to an already-live node.
    #[must_use]
    pub const fn is_existing(self) -> bool {
        matches!(self, Self::Existing(_))
    }

    /// Returns `true` if this is a placeholder for a not-yet-created node.
    #[must_use]
    pub const fn is_new(self) -> bool {
        matches!(self, Self::New(_))
    }

    /// Returns the underlying live `NodeId` if this is an
    /// [`Existing`](Self::Existing) reference.
    #[must_use]
    pub const fn existing_id(self) -> Option<NodeId> {
        match self {
            Self::Existing(id) => Some(id),
            Self::New(_) => None,
        }
    }

    /// Returns the creation index if this is a [`New`](Self::New) placeholder.
    #[must_use]
    pub const fn new_index(self) -> Option<u32> {
        match self {
            Self::New(i) => Some(i),
            Self::Existing(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn existing_roundtrips() {
        let id = NodeId::from_raw(7);
        let r = StagedRef::existing(id);
        assert!(r.is_existing());
        assert!(!r.is_new());
        assert_eq!(r.existing_id(), Some(id));
        assert_eq!(r.new_index(), None);
    }

    #[test]
    fn new_roundtrips() {
        let r = StagedRef::new(3);
        assert!(r.is_new());
        assert!(!r.is_existing());
        assert_eq!(r.new_index(), Some(3));
        assert_eq!(r.existing_id(), None);
    }

    #[test]
    fn is_copy_and_eq() {
        let r = StagedRef::New(1);
        let c = r; // Copy
        assert_eq!(r, c);
    }
}
