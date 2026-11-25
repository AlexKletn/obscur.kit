//! The [`FocusScope`] focus-containment scope for dialogs and menus
//! (spec §7.1 focus).

use obscur_identity::NodeId;

use super::FocusScopeId;

#[doc = include_str!("../../docs/api/focus/focus_scope.doc/FocusScope.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FocusScope {
    id: FocusScopeId,
    trap: bool,
    members: Vec<NodeId>,
}

impl FocusScope {
    /// Creates an empty, non-trapping scope with identifier `id`.
    #[must_use]
    pub fn new(id: FocusScopeId) -> Self {
        Self {
            id,
            trap: false,
            members: Vec::new(),
        }
    }

    #[doc = include_str!("../../docs/api/focus/focus_scope.doc/trapping.md")]
    #[must_use]
    pub fn trapping(mut self, trap: bool) -> Self {
        self.trap = trap;
        self
    }

    /// Appends `node` as the next member of this scope, returning the updated
    /// value.
    #[must_use]
    pub fn with_member(mut self, node: NodeId) -> Self {
        self.members.push(node);
        self
    }

    /// Appends `node` as the next member of this scope.
    pub fn add_member(&mut self, node: NodeId) {
        self.members.push(node);
    }

    /// Returns this scope's identifier.
    #[must_use]
    pub const fn id(&self) -> FocusScopeId {
        self.id
    }

    /// Returns whether this scope traps focus.
    #[must_use]
    pub const fn traps(&self) -> bool {
        self.trap
    }

    /// Returns this scope's ordered members.
    #[must_use]
    pub fn members(&self) -> &[NodeId] {
        &self.members
    }

    /// Returns whether `node` is a member of this scope.
    #[must_use]
    pub fn contains(&self, node: NodeId) -> bool {
        self.members.contains(&node)
    }

    /// Returns the number of members in this scope.
    #[must_use]
    pub fn len(&self) -> usize {
        self.members.len()
    }

    /// Returns whether this scope has no members.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.members.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(raw: u64) -> NodeId {
        NodeId::from_raw(raw)
    }

    #[test]
    fn new_scope_is_empty_and_non_trapping() {
        let s = FocusScope::new(FocusScopeId::from_raw(1));
        assert_eq!(s.id(), FocusScopeId::from_raw(1));
        assert!(!s.traps());
        assert!(s.is_empty());
        assert_eq!(s.len(), 0);
    }

    #[test]
    fn trapping_sets_the_flag() {
        let s = FocusScope::new(FocusScopeId::from_raw(1)).trapping(true);
        assert!(s.traps());
    }

    #[test]
    fn members_and_contains() {
        let mut s = FocusScope::new(FocusScopeId::from_raw(1)).with_member(node(1));
        s.add_member(node(2));
        assert_eq!(s.members(), &[node(1), node(2)]);
        assert_eq!(s.len(), 2);
        assert!(s.contains(node(1)));
        assert!(s.contains(node(2)));
        assert!(!s.contains(node(3)));
    }
}
