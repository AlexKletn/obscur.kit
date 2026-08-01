#![doc = include_str!("logical_parent.doc/_module.md")]

use obscur_identity::NodeId;

#[doc = include_str!("logical_parent.doc/LogicalParent.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LogicalParent(Option<NodeId>);

impl LogicalParent {
    #[doc = include_str!("logical_parent.doc/root.md")]
    #[must_use]
    pub const fn root() -> Self {
        Self(None)
    }

    #[doc = include_str!("logical_parent.doc/of.md")]
    #[must_use]
    pub const fn of(parent: NodeId) -> Self {
        Self(Some(parent))
    }

    #[doc = include_str!("logical_parent.doc/parent.md")]
    #[must_use]
    pub const fn parent(self) -> Option<NodeId> {
        self.0
    }

    #[doc = include_str!("logical_parent.doc/is_root.md")]
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
