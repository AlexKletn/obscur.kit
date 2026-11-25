//! An accumulator of pending relayouts, keyed by node and typed reason.

use crate::context::LayoutContext;
use crate::invalidation::{LayoutInvalidationReason, PropagationResult};
use obscur_identity::{FastMap, NodeId};
use std::collections::HashMap;

#[doc = include_str!("../../docs/api/invalidation/layout_invalidation.doc/LayoutInvalidation.md")]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LayoutInvalidation {
    pending: FastMap<NodeId, LayoutInvalidationReason>,
}

impl LayoutInvalidation {
    /// Creates an empty accumulator with no pending relayouts.
    #[must_use]
    pub fn new() -> Self {
        Self {
            pending: HashMap::default(),
        }
    }

    #[doc = include_str!("../../docs/api/invalidation/layout_invalidation.doc/invalidate.md")]
    pub fn invalidate(&mut self, node: NodeId, reason: LayoutInvalidationReason) {
        self.pending.entry(node).or_insert(reason);
    }

    /// Whether `node` is currently pending recomputation.
    #[must_use]
    pub fn is_invalidated(&self, node: NodeId) -> bool {
        self.pending.contains_key(&node)
    }

    /// The reason `node` was invalidated, if it is pending.
    #[must_use]
    pub fn reason(&self, node: NodeId) -> Option<LayoutInvalidationReason> {
        self.pending.get(&node).copied()
    }

    /// The number of distinct nodes pending recomputation.
    #[must_use]
    pub fn len(&self) -> usize {
        self.pending.len()
    }

    /// Whether nothing is pending.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }

    /// Iterates over the pending `(node, reason)` entries without consuming.
    pub fn iter(&self) -> impl Iterator<Item = (&NodeId, &LayoutInvalidationReason)> {
        self.pending.iter()
    }

    /// Removes and returns the pending map, leaving this accumulator empty.
    #[must_use]
    pub fn take(&mut self) -> FastMap<NodeId, LayoutInvalidationReason> {
        std::mem::take(&mut self.pending)
    }

    /// Alias for [`take`](Self::take): drains the pending map, leaving this
    /// accumulator empty.
    #[must_use]
    pub fn drain(&mut self) -> FastMap<NodeId, LayoutInvalidationReason> {
        self.take()
    }

    #[doc = include_str!("../../docs/api/invalidation/layout_invalidation.doc/note_constraints_changed.md")]
    pub fn note_constraints_changed(&mut self, ctx: &LayoutContext) -> bool {
        if ctx.should_remeasure(true, None) {
            self.invalidate(ctx.node(), LayoutInvalidationReason::ConstraintsChanged);
            true
        } else {
            false
        }
    }

    #[doc = include_str!("../../docs/api/invalidation/layout_invalidation.doc/note_child_changed.md")]
    pub fn note_child_changed(&mut self, ctx: &LayoutContext, child: NodeId) -> bool {
        if ctx.should_remeasure(false, Some(child)) {
            self.invalidate(ctx.node(), LayoutInvalidationReason::ChildSizeChanged);
            true
        } else {
            false
        }
    }

    #[doc = include_str!("../../docs/api/invalidation/layout_invalidation.doc/note_style_changed.md")]
    pub fn note_style_changed(&mut self, node: NodeId) {
        self.invalidate(node, LayoutInvalidationReason::StyleChanged);
    }

    #[doc = include_str!("../../docs/api/invalidation/layout_invalidation.doc/note_upward.md")]
    pub fn note_upward(
        &mut self,
        parent_ctx: &LayoutContext,
        child_result: &PropagationResult,
    ) -> bool {
        if child_result.continues_upward() {
            self.invalidate(
                parent_ctx.node(),
                LayoutInvalidationReason::ChildSizeChanged,
            );
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::LayoutInvalidation;
    use crate::context::{LayoutContext, LayoutContextId, LayoutDependency};
    use crate::invalidation::{LayoutInvalidationReason, PropagationResult};
    use obscur_identity::NodeId;

    fn ctx(node: u64) -> LayoutContext {
        LayoutContext::new(LayoutContextId::from_raw(1), NodeId::from_raw(node))
    }

    #[test]
    fn new_is_empty() {
        let inv = LayoutInvalidation::new();
        assert!(inv.is_empty());
        assert_eq!(inv.len(), 0);
    }

    #[test]
    fn invalidate_records_and_reports() {
        let mut inv = LayoutInvalidation::new();
        let n = NodeId::from_raw(5);
        inv.invalidate(n, LayoutInvalidationReason::StyleChanged);
        assert!(inv.is_invalidated(n));
        assert_eq!(inv.reason(n), Some(LayoutInvalidationReason::StyleChanged));
        assert_eq!(inv.len(), 1);
    }

    #[test]
    fn merge_keeps_first_reason() {
        let mut inv = LayoutInvalidation::new();
        let n = NodeId::from_raw(5);
        inv.invalidate(n, LayoutInvalidationReason::StyleChanged);
        inv.invalidate(n, LayoutInvalidationReason::ConstraintsChanged);
        assert_eq!(inv.reason(n), Some(LayoutInvalidationReason::StyleChanged));
        assert_eq!(inv.len(), 1);
    }

    #[test]
    fn drain_empties() {
        let mut inv = LayoutInvalidation::new();
        inv.invalidate(NodeId::from_raw(1), LayoutInvalidationReason::TreeChanged);
        let drained = inv.drain();
        assert_eq!(drained.len(), 1);
        assert!(inv.is_empty());
    }

    #[test]
    fn take_empties() {
        let mut inv = LayoutInvalidation::new();
        inv.invalidate(NodeId::from_raw(1), LayoutInvalidationReason::TreeChanged);
        let taken = inv.take();
        assert_eq!(taken.len(), 1);
        assert!(inv.is_empty());
    }

    #[test]
    fn note_constraints_changed_only_when_dependent() {
        let mut inv = LayoutInvalidation::new();

        let independent = ctx(10);
        assert!(!inv.note_constraints_changed(&independent));
        assert!(!inv.is_invalidated(NodeId::from_raw(10)));

        let mut dependent = ctx(11);
        *dependent.dependency_mut() = LayoutDependency::new().with_constraints_affect_size(true);
        assert!(inv.note_constraints_changed(&dependent));
        assert_eq!(
            inv.reason(NodeId::from_raw(11)),
            Some(LayoutInvalidationReason::ConstraintsChanged)
        );
    }

    #[test]
    fn note_child_changed_only_for_depended_child() {
        let mut inv = LayoutInvalidation::new();
        let mut parent = ctx(10);
        let child = NodeId::from_raw(20);
        let other = NodeId::from_raw(21);
        parent.dependency_mut().add_child(child);

        assert!(!inv.note_child_changed(&parent, other));
        assert!(!inv.is_invalidated(NodeId::from_raw(10)));

        assert!(inv.note_child_changed(&parent, child));
        assert_eq!(
            inv.reason(NodeId::from_raw(10)),
            Some(LayoutInvalidationReason::ChildSizeChanged)
        );
    }

    #[test]
    fn note_style_changed_is_unconditional() {
        let mut inv = LayoutInvalidation::new();
        let n = NodeId::from_raw(7);
        inv.note_style_changed(n);
        assert_eq!(inv.reason(n), Some(LayoutInvalidationReason::StyleChanged));
    }

    #[test]
    fn note_upward_only_when_child_continues() {
        let mut inv = LayoutInvalidation::new();
        let parent = ctx(10);

        // Stable geometry: does not continue upward.
        assert!(!inv.note_upward(&parent, &PropagationResult::new(false, false)));
        assert!(!inv.is_invalidated(NodeId::from_raw(10)));

        // Contained: does not continue upward.
        assert!(!inv.note_upward(&parent, &PropagationResult::new(true, true)));
        assert!(!inv.is_invalidated(NodeId::from_raw(10)));

        // Geometry changed and not contained: continues upward.
        assert!(inv.note_upward(&parent, &PropagationResult::new(true, false)));
        assert_eq!(
            inv.reason(NodeId::from_raw(10)),
            Some(LayoutInvalidationReason::ChildSizeChanged)
        );
    }

    #[test]
    fn iter_visits_all() {
        let mut inv = LayoutInvalidation::new();
        inv.invalidate(NodeId::from_raw(1), LayoutInvalidationReason::StyleChanged);
        inv.invalidate(NodeId::from_raw(2), LayoutInvalidationReason::TreeChanged);
        assert_eq!(inv.iter().count(), 2);
    }
}
