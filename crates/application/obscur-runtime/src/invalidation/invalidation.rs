#![doc = include_str!("invalidation.doc/_module.md")]

use obscur_identity::NodeId;

use crate::invalidation::{InvalidationKind, InvalidationReason};

#[doc = include_str!("invalidation.doc/Invalidation.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Invalidation {
    node: Option<NodeId>,
    reason: InvalidationReason,
}

impl Invalidation {
    #[doc = include_str!("invalidation.doc/new.md")]
    #[must_use]
    pub fn new(node: Option<NodeId>, reason: InvalidationReason) -> Self {
        Self { node, reason }
    }

    #[doc = include_str!("invalidation.doc/global.md")]
    #[must_use]
    pub fn global(reason: InvalidationReason) -> Self {
        Self { node: None, reason }
    }

    #[doc = include_str!("invalidation.doc/node.md")]
    #[must_use]
    pub fn node(self) -> Option<NodeId> {
        self.node
    }

    #[doc = include_str!("invalidation.doc/reason.md")]
    #[must_use]
    pub fn reason(self) -> InvalidationReason {
        self.reason
    }

    #[doc = include_str!("invalidation.doc/kind.md")]
    #[must_use]
    pub fn kind(self) -> InvalidationKind {
        self.reason.kind()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(raw: u64) -> NodeId {
        NodeId::from_raw(raw)
    }

    #[test]
    fn new_records_node_and_reason() {
        let inv = Invalidation::new(Some(node(7)), InvalidationReason::StyleChanged);
        assert_eq!(inv.node(), Some(node(7)));
        assert_eq!(inv.reason(), InvalidationReason::StyleChanged);
        assert_eq!(inv.kind(), InvalidationKind::Style);
    }

    #[test]
    fn global_has_no_node() {
        let inv = Invalidation::global(InvalidationReason::EnvironmentChanged);
        assert_eq!(inv.node(), None);
        assert_eq!(inv.kind(), InvalidationKind::Environment);
    }
}
