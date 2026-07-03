//! The [`FocusResult`] reporting the outcome of a focus request
//! (spec §7.1 focus).

use obscur_identity::NodeId;

/// The outcome of resolving a [`FocusRequest`](super::FocusRequest) (spec §7.1).
///
/// The focus state machine returns exactly one of these for every request,
/// distinguishing a successful move from a rejection (target exists but is not
/// focusable), no available target, or a request that left focus unchanged.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FocusResult {
    /// Focus moved to this node.
    Focused(NodeId),
    /// Focus was removed; nothing is focused.
    Cleared,
    /// The target exists but is not focusable, so focus did not move.
    Rejected(NodeId),
    /// There was nothing to focus (no matching or eligible target).
    NoTarget,
    /// The requested target was already focused; focus did not change.
    Unchanged(NodeId),
}

impl FocusResult {
    /// Returns whether the request succeeded in changing focus state:
    /// [`Focused`](Self::Focused) or [`Cleared`](Self::Cleared).
    #[must_use]
    pub const fn is_success(self) -> bool {
        matches!(self, Self::Focused(_) | Self::Cleared)
    }

    /// Returns the node that now holds focus after this result, if any:
    /// [`Focused`](Self::Focused) or [`Unchanged`](Self::Unchanged).
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
        assert_eq!(FocusResult::Unchanged(node(3)).focused_node(), Some(node(3)));
        assert_eq!(FocusResult::Cleared.focused_node(), None);
        assert_eq!(FocusResult::Rejected(node(4)).focused_node(), None);
        assert_eq!(FocusResult::NoTarget.focused_node(), None);
    }
}
