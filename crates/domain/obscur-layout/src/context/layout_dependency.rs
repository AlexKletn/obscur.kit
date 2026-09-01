#![doc = include_str!("layout_dependency.doc/_module.md")]

use obscur_identity::{FastSet, NodeId};
use std::collections::HashSet;

#[doc = include_str!("layout_dependency.doc/LayoutDependency.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayoutDependency {
    #[doc = include_str!("layout_dependency.doc/constraints_affect_size.md")]
    constraints_affect_size: bool,
    #[doc = include_str!("layout_dependency.doc/children.md")]
    children: FastSet<NodeId>,
}

impl LayoutDependency {
    #[doc = include_str!("layout_dependency.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        Self {
            constraints_affect_size: false,
            children: HashSet::default(),
        }
    }

    #[doc = include_str!("layout_dependency.doc/with_constraints_affect_size.md")]
    #[must_use]
    pub fn with_constraints_affect_size(mut self, affects: bool) -> Self {
        self.constraints_affect_size = affects;
        self
    }

    #[doc = include_str!("layout_dependency.doc/add_child.md")]
    pub fn add_child(&mut self, child: NodeId) {
        self.children.insert(child);
    }

    #[doc = include_str!("layout_dependency.doc/constraints_affect_size_2.md")]
    #[must_use]
    pub const fn constraints_affect_size(&self) -> bool {
        self.constraints_affect_size
    }

    #[doc = include_str!("layout_dependency.doc/children_2.md")]
    #[must_use]
    pub const fn children(&self) -> &FastSet<NodeId> {
        &self.children
    }

    #[doc = include_str!("layout_dependency.doc/affected_by_constraint_change.md")]
    #[must_use]
    pub const fn affected_by_constraint_change(&self) -> bool {
        self.constraints_affect_size
    }

    #[doc = include_str!("layout_dependency.doc/affected_by_child.md")]
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
