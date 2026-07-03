//! `StagedRef`: a transaction-local reference to a node.

use obscur_identity::NodeId;

/// A reference to a node *within a staged transaction*.
///
/// While a [`TreeTransaction`](super::TreeTransaction) is being built its
/// changes have not touched the live tree yet, so a node that the transaction
/// itself will create does not have a [`NodeId`] until commit time. A
/// `StagedRef` bridges that gap: it is either an [`Existing`](Self::Existing)
/// live node, or a [`New`](Self::New) placeholder naming the *i*-th node staged
/// for creation in this transaction (0-based, in stage order).
///
/// At [`commit`](super::TreeTransaction::commit) each `New(i)` is resolved to
/// the concrete `NodeId` returned by the *i*-th allocation, so a node created
/// earlier in the batch can be referenced (e.g. as a parent) by later changes.
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
