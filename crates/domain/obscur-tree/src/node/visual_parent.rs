//! The visual parent of a retained node.

use obscur_identity::NodeId;

/// A node's **visual** parent: where it is actually painted, i.e. its place in
/// the visual / stacking tree.
///
/// For an ordinary node the visual parent coincides with its
/// [`super::LogicalParent`]. For a portalled node they differ: the portal moves
/// the node's paint location (visual parent) while its logical parent — and thus
/// its style/theme scope — is preserved (spec §4.14). A newtype over
/// `Option<NodeId>`: `None` denotes a visual root.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VisualParent(Option<NodeId>);

impl VisualParent {
    /// A visual parent denoting the visual root (no parent).
    #[must_use]
    pub const fn root() -> Self {
        Self(None)
    }

    /// A visual parent pointing at `parent`.
    #[must_use]
    pub const fn of(parent: NodeId) -> Self {
        Self(Some(parent))
    }

    /// Returns the parent `NodeId`, or `None` at the visual root.
    #[must_use]
    pub const fn parent(self) -> Option<NodeId> {
        self.0
    }

    /// Returns `true` if this node has no visual parent (it is a visual root).
    #[must_use]
    pub const fn is_root(self) -> bool {
        self.0.is_none()
    }
}

impl Default for VisualParent {
    fn default() -> Self {
        Self::root()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn root_has_no_parent() {
        let p = VisualParent::root();
        assert!(p.is_root());
        assert_eq!(p.parent(), None);
        assert_eq!(VisualParent::default(), p);
    }

    #[test]
    fn of_carries_parent() {
        let id = NodeId::from_raw(9);
        let p = VisualParent::of(id);
        assert!(!p.is_root());
        assert_eq!(p.parent(), Some(id));
    }
}
