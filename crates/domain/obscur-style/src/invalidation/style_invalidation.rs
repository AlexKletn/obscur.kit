//! The pending-restyle accumulator: [`StyleInvalidation`] (§4.3, §4.14).
//!
//! [`StyleInvalidation`] is a small map of `NodeId -> StyleInvalidationReason`
//! recording which nodes must be restyled and why. The runtime feeds it changes
//! and consults each node's [`StyleDependency`], so the invalidation engine is
//! decoupled from the tree: this type never walks the tree itself, it only
//! records and merges the resulting invalidations (§4.3). Confining recalculation
//! to the recorded nodes is how invalidation stays limited to affected consumers
//! (§4.14).

use crate::computed::StyleDiff;
use crate::invalidation::{StyleDependency, StyleInvalidationReason};
use crate::selector::PseudoState;
use obscur_identity::NodeId;
use std::collections::HashMap;
use std::collections::hash_map::Iter;

/// An accumulator of pending node restyles keyed by [`NodeId`] (§4.3).
///
/// Each node maps to the typed [`StyleInvalidationReason`] that first put it in
/// the set. When a node is invalidated more than once the first reason is kept
/// (documented precedence: earliest wins); merging never panics and never loses a
/// node.
///
/// # Who reads it
///
/// The runtime's style phase, on the interaction path: `RestyleScope` fills one of
/// these from the nodes whose `:hover`/`:focus` state changed,
/// `StyleMatchingCoordinator::rematch_into` iterates it to re-match exactly those
/// elements, and `StyleCoordinator::resolve_incremental` reads
/// [`is_invalidated`](Self::is_invalidated) to decide, per node, whether to
/// recompute or keep the retained style. A frame whose style inputs moved for any
/// *other* reason — a new stylesheet, a resize, a tree edit — builds no set and
/// restyles in full, so this is consulted on one path, not all of them.
///
/// [`take`](Self::take)/[`drain`](Self::drain) are **not** on that path: the set is
/// read, not consumed, because the same set gates both the re-match and the
/// re-resolve. They have no production caller.
///
/// The tree walk — finding the children of a changed parent, or the node whose
/// state toggled — is the runtime's job; this type only records and merges the
/// invalidations that walk produces.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StyleInvalidation {
    pending: HashMap<NodeId, StyleInvalidationReason>,
}

impl StyleInvalidation {
    /// Creates an empty invalidation set.
    #[must_use]
    pub fn new() -> Self {
        Self {
            pending: HashMap::new(),
        }
    }

    /// Records that `node` must be restyled for `reason`.
    ///
    /// If `node` was already invalidated the existing reason is kept (earliest
    /// reason wins), so the record is deterministic regardless of call order.
    pub fn invalidate(&mut self, node: NodeId, reason: StyleInvalidationReason) {
        self.pending.entry(node).or_insert(reason);
    }

    /// Returns whether `node` is in the pending set.
    #[must_use]
    pub fn is_invalidated(&self, node: NodeId) -> bool {
        self.pending.contains_key(&node)
    }

    /// Returns the recorded reason for `node`, if it is invalidated.
    #[must_use]
    pub fn reason(&self, node: NodeId) -> Option<StyleInvalidationReason> {
        self.pending.get(&node).copied()
    }

    /// Returns the number of invalidated nodes.
    #[must_use]
    pub fn len(&self) -> usize {
        self.pending.len()
    }

    /// Returns whether no nodes are invalidated.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }

    /// Iterates the pending `(NodeId, StyleInvalidationReason)` entries.
    #[must_use]
    pub fn iter(&self) -> Iter<'_, NodeId, StyleInvalidationReason> {
        self.pending.iter()
    }

    /// Removes and returns the entire pending set, leaving `self` empty.
    ///
    /// The runtime calls this once per style pass to consume the work.
    #[must_use]
    pub fn take(&mut self) -> HashMap<NodeId, StyleInvalidationReason> {
        std::mem::take(&mut self.pending)
    }

    /// Alias for [`take`](Self::take): drains the pending set, leaving `self`
    /// empty and returning what was pending.
    #[must_use]
    pub fn drain(&mut self) -> HashMap<NodeId, StyleInvalidationReason> {
        self.take()
    }

    /// Invalidates `child` if `dep` shows it consumed something the parent's
    /// change (`parent_diff`) altered (§4.14).
    ///
    /// Consults [`StyleDependency::affected_by_parent_change`] and records the
    /// returned reason; a node the parent change does not affect is left
    /// untouched. Returns whether `child` was invalidated by this call. The caller
    /// (the runtime) is responsible for having identified `child` as a child of
    /// the changed parent.
    pub fn note_parent_style_changed(
        &mut self,
        child: NodeId,
        dep: &StyleDependency,
        parent_diff: &StyleDiff,
    ) -> bool {
        if let Some(reason) = dep.affected_by_parent_change(parent_diff) {
            self.invalidate(child, reason);
            true
        } else {
            false
        }
    }

    /// Invalidates `node` if `dep` shows its matching depends on a pseudo-state
    /// that `changed` toggled (§4.14).
    ///
    /// Consults [`StyleDependency::affected_by_pseudo_change`] and records
    /// [`StyleInvalidationReason::PseudoStateChanged`] on a match. Returns whether
    /// `node` was invalidated by this call.
    pub fn note_pseudo_state_changed(
        &mut self,
        node: NodeId,
        dep: &StyleDependency,
        changed: PseudoState,
    ) -> bool {
        if dep.affected_by_pseudo_change(changed) {
            self.invalidate(node, StyleInvalidationReason::PseudoStateChanged);
            true
        } else {
            false
        }
    }
}

impl<'a> IntoIterator for &'a StyleInvalidation {
    type Item = (&'a NodeId, &'a StyleInvalidationReason);
    type IntoIter = Iter<'a, NodeId, StyleInvalidationReason>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::computed::{ComputedStyle, ComputedValue};
    use crate::property::PropertyId;

    fn node(raw: u64) -> NodeId {
        NodeId::from_raw(raw)
    }

    #[test]
    fn records_and_reports() {
        let mut inv = StyleInvalidation::new();
        assert!(inv.is_empty());
        inv.invalidate(node(1), StyleInvalidationReason::StylesheetChanged);
        assert!(inv.is_invalidated(node(1)));
        assert_eq!(
            inv.reason(node(1)),
            Some(StyleInvalidationReason::StylesheetChanged)
        );
        assert_eq!(inv.len(), 1);
        assert!(!inv.is_invalidated(node(2)));
        assert_eq!(inv.reason(node(2)), None);
    }

    #[test]
    fn merge_keeps_first_reason() {
        let mut inv = StyleInvalidation::new();
        inv.invalidate(node(1), StyleInvalidationReason::StylesheetChanged);
        inv.invalidate(node(1), StyleInvalidationReason::PseudoStateChanged);
        // Earliest reason wins, and the node is still recorded exactly once.
        assert_eq!(inv.len(), 1);
        assert_eq!(
            inv.reason(node(1)),
            Some(StyleInvalidationReason::StylesheetChanged)
        );
    }

    #[test]
    fn drain_empties() {
        let mut inv = StyleInvalidation::new();
        inv.invalidate(node(1), StyleInvalidationReason::DeclarationsChanged);
        inv.invalidate(node(2), StyleInvalidationReason::InlineStyleChanged);
        let drained = inv.drain();
        assert_eq!(drained.len(), 2);
        assert!(inv.is_empty());
        assert_eq!(inv.len(), 0);
    }

    #[test]
    fn note_parent_style_changed_gates_on_dependency() {
        // Parent changed the inherited `color` property.
        let mut old = ComputedStyle::new();
        old.set(PropertyId::Color, ComputedValue::number(0.0));
        let mut new = ComputedStyle::new();
        new.set(PropertyId::Color, ComputedValue::number(1.0));
        let diff = StyleDiff::between(&old, &new);

        let inheriting = StyleDependency::new().with_inherits_from_parent(true);
        let non_inheriting = StyleDependency::new().with_inherits_from_parent(false);

        let mut inv = StyleInvalidation::new();
        assert!(inv.note_parent_style_changed(node(1), &inheriting, &diff));
        assert!(!inv.note_parent_style_changed(node(2), &non_inheriting, &diff));

        assert_eq!(
            inv.reason(node(1)),
            Some(StyleInvalidationReason::ParentStyleChanged)
        );
        assert!(!inv.is_invalidated(node(2)));
    }

    #[test]
    fn note_pseudo_state_changed_gates_on_dependency() {
        let dep = StyleDependency::new().with_pseudo_states(PseudoState::HOVER);
        let mut inv = StyleInvalidation::new();
        assert!(inv.note_pseudo_state_changed(node(1), &dep, PseudoState::HOVER));
        assert!(!inv.note_pseudo_state_changed(node(2), &dep, PseudoState::ACTIVE));
        assert_eq!(
            inv.reason(node(1)),
            Some(StyleInvalidationReason::PseudoStateChanged)
        );
        assert!(!inv.is_invalidated(node(2)));
    }
}
