//! The [`FocusRequest`] describing a requested focus move (spec §7.1 focus).

use obscur_identity::NodeId;

#[doc = include_str!("../../docs/api/focus/focus_request.doc/FocusRequest.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FocusRequest {
    /// Focus the identified node.
    Node(NodeId),
    /// Move focus to the next node in tab order.
    Next,
    /// Move focus to the previous node in tab order.
    Previous,
    /// Move focus to the first node in tab order.
    First,
    /// Move focus to the last node in tab order.
    Last,
    /// Clear focus (blur), leaving nothing focused.
    Clear,
}

impl FocusRequest {
    /// Requests focus of the specific node `node`.
    #[must_use]
    pub const fn node(node: NodeId) -> Self {
        Self::Node(node)
    }

    /// Requests focus of the next node in tab order.
    #[must_use]
    pub const fn next() -> Self {
        Self::Next
    }

    /// Requests focus of the previous node in tab order.
    #[must_use]
    pub const fn previous() -> Self {
        Self::Previous
    }

    /// Requests focus of the first node in tab order.
    #[must_use]
    pub const fn first() -> Self {
        Self::First
    }

    /// Requests focus of the last node in tab order.
    #[must_use]
    pub const fn last() -> Self {
        Self::Last
    }

    /// Requests clearing focus (blur).
    #[must_use]
    pub const fn clear() -> Self {
        Self::Clear
    }

    /// Returns whether this is a directional tab step ([`Next`](Self::Next) or
    /// [`Previous`](Self::Previous)).
    #[must_use]
    pub const fn is_directional(self) -> bool {
        matches!(self, Self::Next | Self::Previous)
    }

    /// Returns the target node for a [`Node`](Self::Node) request, else `None`.
    #[must_use]
    pub const fn target_node(self) -> Option<NodeId> {
        match self {
            Self::Node(node) => Some(node),
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
    fn constructors_build_expected_variants() {
        assert_eq!(FocusRequest::node(node(1)), FocusRequest::Node(node(1)));
        assert_eq!(FocusRequest::next(), FocusRequest::Next);
        assert_eq!(FocusRequest::previous(), FocusRequest::Previous);
        assert_eq!(FocusRequest::first(), FocusRequest::First);
        assert_eq!(FocusRequest::last(), FocusRequest::Last);
        assert_eq!(FocusRequest::clear(), FocusRequest::Clear);
    }

    #[test]
    fn is_directional_only_for_next_and_previous() {
        assert!(FocusRequest::next().is_directional());
        assert!(FocusRequest::previous().is_directional());
        assert!(!FocusRequest::first().is_directional());
        assert!(!FocusRequest::last().is_directional());
        assert!(!FocusRequest::clear().is_directional());
        assert!(!FocusRequest::node(node(1)).is_directional());
    }

    #[test]
    fn target_node_only_for_node_variant() {
        assert_eq!(FocusRequest::node(node(9)).target_node(), Some(node(9)));
        assert_eq!(FocusRequest::next().target_node(), None);
        assert_eq!(FocusRequest::clear().target_node(), None);
    }
}
