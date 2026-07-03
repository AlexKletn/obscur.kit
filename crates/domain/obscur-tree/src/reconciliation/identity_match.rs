//! `IdentityMatch`: the reuse-or-create decision for one desired child.

use obscur_identity::NodeId;

/// The reconciliation decision for a single desired child.
///
/// For every desired child, reconciliation must decide whether to **reuse** an
/// existing retained node — preserving its stable [`NodeId`] and therefore all
/// state that hangs off that identity (layout boxes, focus, selection,
/// animation, caches; spec §2.2) — or to **create** a fresh node. This is the
/// atomic outcome of that decision; a whole parent's children reconcile to an
/// ordered list of `IdentityMatch`es aligned with the desired children.
///
/// The variant carries no kind information: compatibility (same variant, same
/// component) has already been checked before a [`IdentityMatch::Reuse`] is
/// produced, so a reuse always targets a node the desired child can inhabit
/// in place.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IdentityMatch {
    /// Reuse the existing retained node identified by this [`NodeId`],
    /// preserving its identity across the update (including across a reorder).
    Reuse(NodeId),
    /// No compatible existing node was available; a fresh node must be created.
    Create,
}

impl IdentityMatch {
    /// Returns the reused [`NodeId`] when this is a [`IdentityMatch::Reuse`],
    /// or `None` for a [`IdentityMatch::Create`].
    #[must_use]
    pub const fn reused(self) -> Option<NodeId> {
        match self {
            Self::Reuse(id) => Some(id),
            Self::Create => None,
        }
    }

    /// Returns `true` if this decision reuses an existing node.
    #[must_use]
    pub const fn is_reuse(self) -> bool {
        matches!(self, Self::Reuse(_))
    }

    /// Returns `true` if this decision creates a fresh node.
    #[must_use]
    pub const fn is_create(self) -> bool {
        matches!(self, Self::Create)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reuse_exposes_id() {
        let id = NodeId::from_raw(7);
        let m = IdentityMatch::Reuse(id);
        assert!(m.is_reuse());
        assert!(!m.is_create());
        assert_eq!(m.reused(), Some(id));
    }

    #[test]
    fn create_has_no_id() {
        let m = IdentityMatch::Create;
        assert!(m.is_create());
        assert!(!m.is_reuse());
        assert_eq!(m.reused(), None);
    }
}
