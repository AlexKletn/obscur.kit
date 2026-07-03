//! The [`FocusScope`] focus-containment scope for dialogs and menus
//! (spec §7.1 focus).

use obscur_identity::NodeId;

/// Opaque identifier for a [`FocusScope`] (spec §7.1).
///
/// A local newtype minted by the runtime to name focus scopes; the numeric
/// value carries no meaning outside the subsystem that assigns it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FocusScopeId(u64);

impl FocusScopeId {
    /// Wraps a raw handle value into a `FocusScopeId`.
    #[must_use]
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    /// Returns the raw handle value backing this identifier.
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }
}

/// A focus-containment scope (spec §7.1).
///
/// A scope groups an ordered set of focusable members. Dialogs and menus open a
/// scope and set `trap` so that tab navigation wraps within the scope's members
/// instead of escaping to the surrounding UI — a *focus trap*. The scope itself
/// is a pure description; the runtime applies trapping using this data together
/// with the tab order and focus-restore stack.
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

    /// Sets whether this scope traps focus, returning the updated value.
    ///
    /// When trapping, tab navigation wraps within [`members`](Self::members)
    /// rather than leaving the scope.
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
