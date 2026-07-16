//! What a layout context's output depends on, for incremental recomputation.

use obscur_identity::NodeId;
use std::collections::HashSet;

/// Records what a [`LayoutContext`](super::LayoutContext)'s layout output
/// depends on, so incremental layout can decide precisely what to recompute
/// (spec §4.3).
///
/// A dependency captures the two inputs to a context's measure:
///
/// - whether its measured size depends on the incoming constraints (so a
///   **constraint change** forces a re-measure), and
/// - which children's sizes feed its measure (so a **child size change** in one
///   of those children forces a re-measure).
///
/// # Status: nothing populates this in production — read it as a contract
///
/// This type used to describe itself as "the structured replacement for
/// scattered ad-hoc dirty flags, which §4.3 explicitly rejects as the
/// architectural model". That was an over-claim about an unbuilt architecture:
/// every call that sets a dependency ([`with_constraints_affect_size`], [`add_child`])
/// is in a `#[cfg(test)]` module, so in production every [`LayoutContext`] carries
/// the empty [`new`](Self::new) dependency and
/// [`should_remeasure`](super::LayoutContext::should_remeasure) — along with the
/// [`LayoutInvalidation`](crate::invalidation::LayoutInvalidation) `note_*` gates
/// built on it — answers for a graph nothing filled in.
///
/// What *is* real is a different mechanism. Layout's incremental half
/// (`LayoutCoordinator::layout_invalidated`) is driven by a per-node
/// [`LayoutInvalidation`](crate::invalidation::LayoutInvalidation) and a measure
/// cache, and it derives its two §4.3 gates without consulting this type at all:
/// the **downward** gate is exact in the cache key (a child re-measures precisely
/// when its constraints differ, because a different constraint is a different key),
/// and the **upward** gate is a whole-ancestor-chain eviction rather than a
/// dependency-driven early stop. This type is what would make that upward gate
/// *precise* instead of conservative — the day something populates it.
///
/// So: a contract for a consumer that does not exist yet, not shipped behaviour.
///
/// [`with_constraints_affect_size`]: Self::with_constraints_affect_size
/// [`add_child`]: Self::add_child
/// [`LayoutContext`]: super::LayoutContext
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayoutDependency {
    /// Whether the node's measured size depends on the incoming constraints.
    constraints_affect_size: bool,
    /// The children whose sizes feed this context's measure.
    children: HashSet<NodeId>,
}

impl LayoutDependency {
    /// Creates an empty dependency: size does not depend on constraints and no
    /// children feed the measure.
    #[must_use]
    pub fn new() -> Self {
        Self {
            constraints_affect_size: false,
            children: HashSet::new(),
        }
    }

    /// Returns a copy with `constraints_affect_size` set to the given value.
    #[must_use]
    pub fn with_constraints_affect_size(mut self, affects: bool) -> Self {
        self.constraints_affect_size = affects;
        self
    }

    /// Records that `child`'s size feeds this context's measure.
    pub fn add_child(&mut self, child: NodeId) {
        self.children.insert(child);
    }

    /// Whether the node's measured size depends on the incoming constraints.
    #[must_use]
    pub const fn constraints_affect_size(&self) -> bool {
        self.constraints_affect_size
    }

    /// The set of children whose sizes feed this context's measure.
    #[must_use]
    pub const fn children(&self) -> &HashSet<NodeId> {
        &self.children
    }

    /// Whether a **constraint change** requires re-measuring this context.
    ///
    /// Equal to [`constraints_affect_size`](Self::constraints_affect_size).
    #[must_use]
    pub const fn affected_by_constraint_change(&self) -> bool {
        self.constraints_affect_size
    }

    /// Whether a size change in `child` requires re-measuring this context.
    ///
    /// True iff `child` is one of the children this context's measure depends on.
    #[must_use]
    pub fn affected_by_child(&self, child: NodeId) -> bool {
        self.children.contains(&child)
    }
}

impl Default for LayoutDependency {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::LayoutDependency;
    use obscur_identity::NodeId;

    #[test]
    fn new_is_empty() {
        let dep = LayoutDependency::new();
        assert!(!dep.affected_by_constraint_change());
        assert!(dep.children().is_empty());
    }

    #[test]
    fn constraints_gate_tracks_flag() {
        let dep = LayoutDependency::new().with_constraints_affect_size(true);
        assert!(dep.affected_by_constraint_change());
        assert!(dep.constraints_affect_size());
    }

    #[test]
    fn child_gate_tracks_membership() {
        let mut dep = LayoutDependency::new();
        let a = NodeId::from_raw(1);
        let b = NodeId::from_raw(2);
        dep.add_child(a);
        assert!(dep.affected_by_child(a));
        assert!(!dep.affected_by_child(b));
    }

    #[test]
    fn default_matches_new() {
        assert_eq!(LayoutDependency::default(), LayoutDependency::new());
    }
}
