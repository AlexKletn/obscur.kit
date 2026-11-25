//! An incremental change an adapter applies without a full re-push (§7.1).

use obscur_identity::NodeId;

#[doc = include_str!("../../docs/api/tree/semantic_update.doc/SemanticUpdate.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SemanticUpdate {
    /// A node was added to the tree.
    NodeAdded(NodeId),
    /// A node was removed from the tree.
    NodeRemoved(NodeId),
    /// A node's semantic data (name, state, value, …) changed in place.
    NodeChanged(NodeId),
    /// The structure below this node changed (children added/removed/reordered).
    SubtreeChanged(NodeId),
    /// The tree root changed, to the given node or to none.
    RootChanged(Option<NodeId>),
    /// Accessibility focus moved, to the given node or was cleared.
    FocusChanged(Option<NodeId>),
}

impl SemanticUpdate {
    /// Returns the primary node this update concerns, or `None` for a
    /// [`RootChanged(None)`](Self::RootChanged) or
    /// [`FocusChanged(None)`](Self::FocusChanged).
    #[must_use]
    pub const fn affected(&self) -> Option<NodeId> {
        match self {
            Self::NodeAdded(id)
            | Self::NodeRemoved(id)
            | Self::NodeChanged(id)
            | Self::SubtreeChanged(id) => Some(*id),
            Self::RootChanged(id) | Self::FocusChanged(id) => *id,
        }
    }

    #[doc = include_str!("../../docs/api/tree/semantic_update.doc/is_structural.md")]
    #[must_use]
    pub const fn is_structural(&self) -> bool {
        matches!(
            self,
            Self::NodeAdded(_)
                | Self::NodeRemoved(_)
                | Self::SubtreeChanged(_)
                | Self::RootChanged(_)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ID: NodeId = NodeId::from_raw(4);

    #[test]
    fn affected_returns_primary_node() {
        assert_eq!(SemanticUpdate::NodeAdded(ID).affected(), Some(ID));
        assert_eq!(SemanticUpdate::NodeRemoved(ID).affected(), Some(ID));
        assert_eq!(SemanticUpdate::NodeChanged(ID).affected(), Some(ID));
        assert_eq!(SemanticUpdate::SubtreeChanged(ID).affected(), Some(ID));
        assert_eq!(SemanticUpdate::RootChanged(Some(ID)).affected(), Some(ID));
        assert_eq!(SemanticUpdate::FocusChanged(Some(ID)).affected(), Some(ID));
    }

    #[test]
    fn affected_is_none_for_cleared_root_and_focus() {
        assert_eq!(SemanticUpdate::RootChanged(None).affected(), None);
        assert_eq!(SemanticUpdate::FocusChanged(None).affected(), None);
    }

    #[test]
    fn is_structural_across_variants() {
        assert!(SemanticUpdate::NodeAdded(ID).is_structural());
        assert!(SemanticUpdate::NodeRemoved(ID).is_structural());
        assert!(SemanticUpdate::SubtreeChanged(ID).is_structural());
        assert!(SemanticUpdate::RootChanged(Some(ID)).is_structural());
        assert!(SemanticUpdate::RootChanged(None).is_structural());
        assert!(!SemanticUpdate::NodeChanged(ID).is_structural());
        assert!(!SemanticUpdate::FocusChanged(Some(ID)).is_structural());
        assert!(!SemanticUpdate::FocusChanged(None).is_structural());
    }
}
