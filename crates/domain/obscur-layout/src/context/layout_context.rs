#![doc = include_str!("layout_context.doc/_module.md")]

use crate::context::{LayoutContainment, LayoutContextId, LayoutDependency};
use obscur_identity::NodeId;

#[doc = include_str!("layout_context.doc/LayoutContext.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayoutContext {
    #[doc = include_str!("layout_context.doc/id.md")]
    id: LayoutContextId,
    #[doc = include_str!("layout_context.doc/node.md")]
    node: NodeId,
    #[doc = include_str!("layout_context.doc/containment.md")]
    containment: LayoutContainment,
    #[doc = include_str!("layout_context.doc/dependency.md")]
    dependency: LayoutDependency,
}

impl LayoutContext {
    #[doc = include_str!("layout_context.doc/new.md")]
    #[must_use]
    pub fn new(id: LayoutContextId, node: NodeId) -> Self {
        Self {
            id,
            node,
            containment: LayoutContainment::none(),
            dependency: LayoutDependency::new(),
        }
    }

    #[doc = include_str!("layout_context.doc/id_2.md")]
    #[must_use]
    pub const fn id(&self) -> LayoutContextId {
        self.id
    }

    #[doc = include_str!("layout_context.doc/node_2.md")]
    #[must_use]
    pub const fn node(&self) -> NodeId {
        self.node
    }

    #[doc = include_str!("layout_context.doc/containment_2.md")]
    #[must_use]
    pub const fn containment(&self) -> LayoutContainment {
        self.containment
    }

    #[doc = include_str!("layout_context.doc/dependency_2.md")]
    #[must_use]
    pub const fn dependency(&self) -> &LayoutDependency {
        &self.dependency
    }

    #[doc = include_str!("layout_context.doc/dependency_mut.md")]
    pub fn dependency_mut(&mut self) -> &mut LayoutDependency {
        &mut self.dependency
    }

    #[doc = include_str!("layout_context.doc/set_containment.md")]
    pub fn set_containment(&mut self, containment: LayoutContainment) {
        self.containment = containment;
    }

    #[doc = include_str!("layout_context.doc/is_containment_boundary.md")]
    #[must_use]
    pub const fn is_containment_boundary(&self) -> bool {
        self.containment.is_boundary()
    }

    #[doc = include_str!("layout_context.doc/should_remeasure.md")]
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
        *dependent.dependency_mut() = LayoutDependency::new().with_constraints_affect_size(true);
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
