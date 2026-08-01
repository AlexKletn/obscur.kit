#![doc = include_str!("focus_scope.doc/_module.md")]

use obscur_identity::NodeId;

#[doc = include_str!("focus_scope.doc/FocusScopeId.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FocusScopeId(u64);

impl FocusScopeId {
    #[doc = include_str!("focus_scope.doc/from_raw.md")]
    #[must_use]
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    #[doc = include_str!("focus_scope.doc/raw.md")]
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }
}

#[doc = include_str!("focus_scope.doc/FocusScope.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FocusScope {
    id: FocusScopeId,
    trap: bool,
    members: Vec<NodeId>,
}

impl FocusScope {
    #[doc = include_str!("focus_scope.doc/new.md")]
    #[must_use]
    pub fn new(id: FocusScopeId) -> Self {
        Self {
            id,
            trap: false,
            members: Vec::new(),
        }
    }

    #[doc = include_str!("focus_scope.doc/trapping.md")]
    #[must_use]
    pub fn trapping(mut self, trap: bool) -> Self {
        self.trap = trap;
        self
    }

    #[doc = include_str!("focus_scope.doc/with_member.md")]
    #[must_use]
    pub fn with_member(mut self, node: NodeId) -> Self {
        self.members.push(node);
        self
    }

    #[doc = include_str!("focus_scope.doc/add_member.md")]
    pub fn add_member(&mut self, node: NodeId) {
        self.members.push(node);
    }

    #[doc = include_str!("focus_scope.doc/id.md")]
    #[must_use]
    pub const fn id(&self) -> FocusScopeId {
        self.id
    }

    #[doc = include_str!("focus_scope.doc/traps.md")]
    #[must_use]
    pub const fn traps(&self) -> bool {
        self.trap
    }

    #[doc = include_str!("focus_scope.doc/members.md")]
    #[must_use]
    pub fn members(&self) -> &[NodeId] {
        &self.members
    }

    #[doc = include_str!("focus_scope.doc/contains.md")]
    #[must_use]
    pub fn contains(&self, node: NodeId) -> bool {
        self.members.contains(&node)
    }

    #[doc = include_str!("focus_scope.doc/len.md")]
    #[must_use]
    pub fn len(&self) -> usize {
        self.members.len()
    }

    #[doc = include_str!("focus_scope.doc/is_empty.md")]
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
    fn scope_id_roundtrips() {
        let id = FocusScopeId::from_raw(7);
        assert_eq!(id.raw(), 7);
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
