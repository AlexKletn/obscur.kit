//! An accumulator of pending relayouts, keyed by node and typed reason.

use crate::context::LayoutContext;
use crate::invalidation::{LayoutInvalidationReason, PropagationResult};
use obscur_identity::NodeId;
use std::collections::HashMap;

/// Accumulates the set of layout contexts that must be recomputed, each tagged
/// with the typed [`LayoutInvalidationReason`] that first invalidated it
/// (spec §4.3).
///
/// # Status: the layout phase consumes this, for measure only
///
/// `UiRuntime::layout_phase` populates one of these from real per-node style
/// deltas and hands it to `LayoutCoordinator::layout_invalidated`, which drops
/// exactly the `MeasureCache` entries it can have invalidated and serves every
/// other node's size without re-measuring. So a hover that changes one node's
/// width re-measures that node and its ancestor chain, not the tree.
///
/// What that does **not** yet cover, stated plainly:
///
/// * **Arrange is not incremental.** The walk still visits every node and re-runs
///   every container's algorithm; only `measure` is gated on this set. That is
///   deliberate — it is what makes the reuse safe, since all of §4.3's
///   propagation is still performed — but the phase remains O(nodes) per frame.
/// * **The `note_*` gates below are not what the runtime calls.** It records
///   through [`invalidate`](Self::invalidate) /
///   [`note_style_changed`](Self::note_style_changed) only. The downward gate is
///   instead exact-by-construction in the measure cache's key (a child
///   re-measures precisely when its constraints differ), and the upward gate is a
///   whole-ancestor-chain closure rather than a
///   [`PropagationResult`]-driven early stop, because nothing populates a
///   [`LayoutContext`]'s [`LayoutDependency`](crate::context::LayoutDependency) in
///   production. Until something does,
///   [`note_constraints_changed`](Self::note_constraints_changed),
///   [`note_child_changed`](Self::note_child_changed) and
///   [`note_upward`](Self::note_upward) are offered to a consumer that does not
///   exist yet — read them as a contract, not as shipped behaviour.
///
/// It is **decoupled from the tree**: it neither stores nor walks parent/child
/// links. The runtime performs the actual tree walk — finding parents, children,
/// and their [`LayoutContext`]s — and feeds the observed changes here. This type
/// only records and merges pending work and, through its `note_*` methods, offers
/// the §4.3 gate decisions:
///
/// - **propagate down only on constraint change** — [`note_constraints_changed`](Self::note_constraints_changed),
/// - **propagate up only on geometry change** (and stop at containment) —
///   [`note_upward`](Self::note_upward),
/// - re-measure a context when one of its depended-on children changed —
///   [`note_child_changed`](Self::note_child_changed).
///
/// Merging is deterministic: on a repeat invalidation of the same node the
/// **first** recorded reason is kept, and no operation ever panics.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LayoutInvalidation {
    pending: HashMap<NodeId, LayoutInvalidationReason>,
}

impl LayoutInvalidation {
    /// Creates an empty accumulator with no pending relayouts.
    #[must_use]
    pub fn new() -> Self {
        Self {
            pending: HashMap::new(),
        }
    }

    /// Records that `node` must be recomputed for the given `reason`.
    ///
    /// If `node` is already pending, the existing (first-recorded) reason is
    /// kept; the new reason is discarded. This keeps merging deterministic and
    /// order-stable, and never panics.
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
    pub fn take(&mut self) -> HashMap<NodeId, LayoutInvalidationReason> {
        std::mem::take(&mut self.pending)
    }

    /// Alias for [`take`](Self::take): drains the pending map, leaving this
    /// accumulator empty.
    #[must_use]
    pub fn drain(&mut self) -> HashMap<NodeId, LayoutInvalidationReason> {
        self.take()
    }

    /// **Downward gate.** Invalidates `ctx`'s node with
    /// [`ConstraintsChanged`](LayoutInvalidationReason::ConstraintsChanged)
    /// **iff** the context's size depends on the incoming constraints
    /// (`ctx.should_remeasure(true, None)`), and returns whether it did.
    ///
    /// This encodes §4.3's "propagate down only when constraints change": a
    /// context whose measured size is independent of its constraints is not
    /// invalidated by a constraint change.
    pub fn note_constraints_changed(&mut self, ctx: &LayoutContext) -> bool {
        if ctx.should_remeasure(true, None) {
            self.invalidate(ctx.node(), LayoutInvalidationReason::ConstraintsChanged);
            true
        } else {
            false
        }
    }

    /// Invalidates `ctx`'s node with
    /// [`ChildSizeChanged`](LayoutInvalidationReason::ChildSizeChanged) **iff**
    /// `child`'s size feeds this context's measure
    /// (`ctx.should_remeasure(false, Some(child))`), and returns whether it did.
    ///
    /// A context that does not depend on `child` is left untouched (§4.3:
    /// recompute only what the dependency graph says is affected).
    pub fn note_child_changed(&mut self, ctx: &LayoutContext, child: NodeId) -> bool {
        if ctx.should_remeasure(false, Some(child)) {
            self.invalidate(ctx.node(), LayoutInvalidationReason::ChildSizeChanged);
            true
        } else {
            false
        }
    }

    /// Invalidates `node` with
    /// [`StyleChanged`](LayoutInvalidationReason::StyleChanged).
    ///
    /// A change to a node's own layout-affecting style always requires
    /// re-laying-out that node, so this is unconditional.
    pub fn note_style_changed(&mut self, node: NodeId) {
        self.invalidate(node, LayoutInvalidationReason::StyleChanged);
    }

    /// **Upward gate.** When a child's `child_result` continues upward
    /// ([`PropagationResult::continues_upward`]), invalidates `parent_ctx`'s
    /// node with
    /// [`ChildSizeChanged`](LayoutInvalidationReason::ChildSizeChanged) and
    /// returns `true` so the runtime knows to keep walking up. If the child's
    /// geometry was stable or contained, nothing is recorded and `false` is
    /// returned.
    ///
    /// This encodes §4.3's "propagate up only when output geometry changes" and
    /// the containment stop, delegating the decision to the child's
    /// [`PropagationResult`].
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
