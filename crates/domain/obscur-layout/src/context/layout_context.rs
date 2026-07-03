//! A layout context: an invalidation boundary anchored at a node.

use crate::context::{LayoutContainment, LayoutContextId, LayoutDependency};
use obscur_identity::NodeId;

/// An **invalidation boundary** anchored at a node (spec §4.2, §4.3).
///
/// A layout context owns the child constraints and dependencies for its
/// subtree, forms an invalidation boundary, and can stop propagation when
/// geometry remains unchanged. It is *not* a bag of dirty flags: what it can be
/// invalidated by is recorded structurally in its [`LayoutDependency`], and
/// whether inner changes escape is recorded in its [`LayoutContainment`]
/// (§4.3).
///
/// Incremental recalculation starts at the nearest affected context;
/// propagation is upward only when output geometry changes and downward only
/// when constraints change (§4.3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayoutContext {
    /// Identifier of this context.
    id: LayoutContextId,
    /// The node at which this boundary is anchored.
    node: NodeId,
    /// Whether layout/size effects are contained at this boundary.
    containment: LayoutContainment,
    /// What this context's layout output depends on.
    dependency: LayoutDependency,
}

impl LayoutContext {
    /// Creates a context anchored at `node` with empty containment and an empty
    /// dependency record.
    #[must_use]
    pub fn new(id: LayoutContextId, node: NodeId) -> Self {
        Self {
            id,
            node,
            containment: LayoutContainment::none(),
            dependency: LayoutDependency::new(),
        }
    }

    /// The identifier of this context.
    #[must_use]
    pub const fn id(&self) -> LayoutContextId {
        self.id
    }

    /// The node at which this boundary is anchored.
    #[must_use]
    pub const fn node(&self) -> NodeId {
        self.node
    }

    /// The containment declaration for this boundary.
    #[must_use]
    pub const fn containment(&self) -> LayoutContainment {
        self.containment
    }

    /// The dependency record for this context.
    #[must_use]
    pub const fn dependency(&self) -> &LayoutDependency {
        &self.dependency
    }

    /// Mutable access to the dependency record, to register constraint/child
    /// dependencies as the subtree is built.
    pub fn dependency_mut(&mut self) -> &mut LayoutDependency {
        &mut self.dependency
    }

    /// Replaces the containment declaration for this boundary.
    pub fn set_containment(&mut self, containment: LayoutContainment) {
        self.containment = containment;
    }

    /// Whether this context forms a containment boundary that stops upward
    /// invalidation propagation (delegates to
    /// [`LayoutContainment::is_boundary`]).
    #[must_use]
    pub const fn is_containment_boundary(&self) -> bool {
        self.containment.is_boundary()
    }

    /// Whether this context must be re-measured given a possible constraint
    /// change and/or a changed child (spec §4.3).
    ///
    /// Returns `true` when either:
    /// - `constraints_changed` is set and this context's size depends on the
    ///   incoming constraints ("propagate down only on constraint change"), or
    /// - `changed_child` is a child whose size feeds this context's measure.
    ///
    /// This encodes that recomputation starts at the nearest affected context
    /// and is driven by structural dependencies rather than dirty flags.
    #[must_use]
    pub fn should_remeasure(
        &self,
        constraints_changed: bool,
        changed_child: Option<NodeId>,
    ) -> bool {
        if constraints_changed && self.dependency.affected_by_constraint_change() {
            return true;
        }
        match changed_child {
            Some(child) => self.dependency.affected_by_child(child),
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::LayoutContext;
    use crate::context::{LayoutContainment, LayoutContextId, LayoutDependency};
    use obscur_identity::NodeId;

    fn ctx() -> LayoutContext {
        LayoutContext::new(LayoutContextId::from_raw(1), NodeId::from_raw(10))
    }

    #[test]
    fn new_is_empty_and_not_a_boundary() {
        let c = ctx();
        assert_eq!(c.id(), LayoutContextId::from_raw(1));
        assert_eq!(c.node(), NodeId::from_raw(10));
        assert!(!c.is_containment_boundary());
        assert!(c.dependency().children().is_empty());
    }

    #[test]
    fn set_containment_updates_boundary() {
        let mut c = ctx();
        c.set_containment(LayoutContainment::layout());
        assert!(c.is_containment_boundary());
        assert_eq!(c.containment(), LayoutContainment::layout());
    }

    #[test]
    fn remeasure_on_constraint_change_only_when_dependent() {
        let independent = ctx();
        assert!(!independent.should_remeasure(true, None));

        let mut dependent = ctx();
        *dependent.dependency_mut() =
            LayoutDependency::new().with_constraints_affect_size(true);
        assert!(dependent.should_remeasure(true, None));
        assert!(!dependent.should_remeasure(false, None));
    }

    #[test]
    fn remeasure_on_changed_child_only_when_dependent() {
        let mut c = ctx();
        let child = NodeId::from_raw(20);
        let other = NodeId::from_raw(21);
        c.dependency_mut().add_child(child);
        assert!(c.should_remeasure(false, Some(child)));
        assert!(!c.should_remeasure(false, Some(other)));
        assert!(!c.should_remeasure(false, None));
    }
}
