//! The logical parent of a retained node.

use obscur_identity::NodeId;

/// A node's **logical** parent: its place in the component / structural tree,
/// which also defines its style and theme scope.
///
/// The logical parent is preserved across portals: a portal changes only where
/// a node is *painted* (its [`super::VisualParent`]), while its logical parent —
/// and therefore its style/theme scope — stays put (spec §4.14). A newtype over
/// `Option<NodeId>`: `None` denotes the tree root.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LogicalParent(Option<NodeId>);

impl LogicalParent {
    /// A logical parent denoting the tree root (no parent).
    #[must_use]
    pub const fn root() -> Self {
        Self(None)
    }

    /// A logical parent pointing at `parent`.
    #[must_use]
    pub const fn of(parent: NodeId) -> Self {
        Self(Some(parent))
    }

    /// Returns the parent `NodeId`, or `None` at the root.
    #[must_use]
    pub const fn parent(self) -> Option<NodeId> {
        self.0
    }

    /// Returns `true` if this node has no logical parent (it is a root).
    #[must_use]
    pub const fn is_root(self) -> bool {
        self.0.is_none()
    }
}

impl Default for LogicalParent {
    fn default() -> Self {
        Self::root()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn root_has_no_parent() {
        let p = LogicalParent::root();
        assert!(p.is_root());
        assert_eq!(p.parent(), None);
        assert_eq!(LogicalParent::default(), p);
    }

    #[test]
    fn of_carries_parent() {
        let id = NodeId::from_raw(42);
        let p = LogicalParent::of(id);
        assert!(!p.is_root());
        assert_eq!(p.parent(), Some(id));
    }
}
