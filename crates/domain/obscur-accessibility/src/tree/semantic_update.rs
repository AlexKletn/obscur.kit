#![doc = include_str!("semantic_update.doc/_module.md")]

use obscur_identity::NodeId;

#[doc = include_str!("semantic_update.doc/SemanticUpdate.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SemanticUpdate {
    #[doc = include_str!("semantic_update.doc/NodeAdded.md")]
    NodeAdded(NodeId),
    #[doc = include_str!("semantic_update.doc/NodeRemoved.md")]
    NodeRemoved(NodeId),
    #[doc = include_str!("semantic_update.doc/NodeChanged.md")]
    NodeChanged(NodeId),
    #[doc = include_str!("semantic_update.doc/SubtreeChanged.md")]
    SubtreeChanged(NodeId),
    #[doc = include_str!("semantic_update.doc/RootChanged.md")]
    RootChanged(Option<NodeId>),
    #[doc = include_str!("semantic_update.doc/FocusChanged.md")]
    FocusChanged(Option<NodeId>),
}

impl SemanticUpdate {
    #[doc = include_str!("semantic_update.doc/affected.md")]
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

    #[doc = include_str!("semantic_update.doc/is_structural.md")]
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
