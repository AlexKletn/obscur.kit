//! A single invalidation record (Obscur Kit 2 specs §4.3).

use obscur_identity::NodeId;

use crate::invalidation::{InvalidationKind, InvalidationReason};

#[doc = include_str!("../../docs/api/invalidation/invalidation.doc/Invalidation.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Invalidation {
    node: Option<NodeId>,
    reason: InvalidationReason,
}

impl Invalidation {
    /// Creates an invalidation record for `node` (or a global change when
    /// `node` is `None`) with the given `reason`.
    #[must_use]
    pub fn new(node: Option<NodeId>, reason: InvalidationReason) -> Self {
        Self { node, reason }
    }

    /// Creates a global invalidation record (`node = None`) with the given
    /// `reason`, for changes not scoped to a single node.
    #[must_use]
    pub fn global(reason: InvalidationReason) -> Self {
        Self { node: None, reason }
    }

    /// Returns the node this invalidation applies to, or `None` for a global
    /// change.
    #[must_use]
    pub fn node(self) -> Option<NodeId> {
        self.node
    }

    /// Returns the runtime-level cause of this invalidation.
    #[must_use]
    pub fn reason(self) -> InvalidationReason {
        self.reason
    }

    /// Returns the classified [`InvalidationKind`], delegating to
    /// [`self.reason().kind()`](InvalidationReason::kind).
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
