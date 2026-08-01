#![doc = include_str!("focus_request.doc/_module.md")]

use obscur_identity::NodeId;

#[doc = include_str!("focus_request.doc/FocusReason.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FocusReason {
    #[doc = include_str!("focus_request.doc/Programmatic.md")]
    Programmatic,
    #[doc = include_str!("focus_request.doc/Pointer.md")]
    Pointer,
    #[doc = include_str!("focus_request.doc/Keyboard.md")]
    Keyboard,
    #[doc = include_str!("focus_request.doc/Restore.md")]
    Restore,
}

#[doc = include_str!("focus_request.doc/FocusRequest.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FocusRequest {
    #[doc = include_str!("focus_request.doc/Node.md")]
    Node(NodeId),
    #[doc = include_str!("focus_request.doc/Next.md")]
    Next,
    #[doc = include_str!("focus_request.doc/Previous.md")]
    Previous,
    #[doc = include_str!("focus_request.doc/First.md")]
    First,
    #[doc = include_str!("focus_request.doc/Last.md")]
    Last,
    #[doc = include_str!("focus_request.doc/Clear.md")]
    Clear,
}

impl FocusRequest {
    #[doc = include_str!("focus_request.doc/node.md")]
    #[must_use]
    pub const fn node(node: NodeId) -> Self {
        Self::Node(node)
    }

    #[doc = include_str!("focus_request.doc/next.md")]
    #[must_use]
    pub const fn next() -> Self {
        Self::Next
    }

    #[doc = include_str!("focus_request.doc/previous.md")]
    #[must_use]
    pub const fn previous() -> Self {
        Self::Previous
    }

    #[doc = include_str!("focus_request.doc/first.md")]
    #[must_use]
    pub const fn first() -> Self {
        Self::First
    }

    #[doc = include_str!("focus_request.doc/last.md")]
    #[must_use]
    pub const fn last() -> Self {
        Self::Last
    }

    #[doc = include_str!("focus_request.doc/clear.md")]
    #[must_use]
    pub const fn clear() -> Self {
        Self::Clear
    }

    #[doc = include_str!("focus_request.doc/is_directional.md")]
    #[must_use]
    pub const fn is_directional(self) -> bool {
        matches!(self, Self::Next | Self::Previous)
    }

    #[doc = include_str!("focus_request.doc/target_node.md")]
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

    #[test]
    fn focus_reason_variants_are_distinct() {
        assert_ne!(FocusReason::Programmatic, FocusReason::Keyboard);
        assert_ne!(FocusReason::Pointer, FocusReason::Restore);
    }
}
