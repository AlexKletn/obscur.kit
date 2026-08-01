#![doc = include_str!("focus_result.doc/_module.md")]

use obscur_identity::NodeId;

#[doc = include_str!("focus_result.doc/FocusResult.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FocusResult {
    #[doc = include_str!("focus_result.doc/Focused.md")]
    Focused(NodeId),
    #[doc = include_str!("focus_result.doc/Cleared.md")]
    Cleared,
    #[doc = include_str!("focus_result.doc/Rejected.md")]
    Rejected(NodeId),
    #[doc = include_str!("focus_result.doc/NoTarget.md")]
    NoTarget,
    #[doc = include_str!("focus_result.doc/Unchanged.md")]
    Unchanged(NodeId),
}

impl FocusResult {
    #[doc = include_str!("focus_result.doc/is_success.md")]
    #[must_use]
    pub const fn is_success(self) -> bool {
        matches!(self, Self::Focused(_) | Self::Cleared)
    }

    #[doc = include_str!("focus_result.doc/focused_node.md")]
    #[must_use]
    pub const fn focused_node(self) -> Option<NodeId> {
        match self {
            Self::Focused(node) | Self::Unchanged(node) => Some(node),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(raw: u64) -> NodeId {
        NodeId::from_raw(raw)
    }

    #[test]
    fn is_success_for_focused_and_cleared() {
        assert!(FocusResult::Focused(node(1)).is_success());
        assert!(FocusResult::Cleared.is_success());
        assert!(!FocusResult::Rejected(node(1)).is_success());
        assert!(!FocusResult::NoTarget.is_success());
        assert!(!FocusResult::Unchanged(node(1)).is_success());
    }

    #[test]
    fn focused_node_for_focused_and_unchanged() {
        assert_eq!(FocusResult::Focused(node(2)).focused_node(), Some(node(2)));
        assert_eq!(
            FocusResult::Unchanged(node(3)).focused_node(),
            Some(node(3))
        );
        assert_eq!(FocusResult::Cleared.focused_node(), None);
        assert_eq!(FocusResult::Rejected(node(4)).focused_node(), None);
        assert_eq!(FocusResult::NoTarget.focused_node(), None);
    }
}
