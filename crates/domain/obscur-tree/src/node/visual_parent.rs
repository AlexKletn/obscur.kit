#![doc = include_str!("visual_parent.doc/_module.md")]

use obscur_identity::NodeId;

#[doc = include_str!("visual_parent.doc/VisualParent.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VisualParent(Option<NodeId>);

impl VisualParent {
    #[doc = include_str!("visual_parent.doc/root.md")]
    #[must_use]
    pub const fn root() -> Self {
        Self(None)
    }

    #[doc = include_str!("visual_parent.doc/of.md")]
    #[must_use]
    pub const fn of(parent: NodeId) -> Self {
        Self(Some(parent))
    }

    #[doc = include_str!("visual_parent.doc/parent.md")]
    #[must_use]
    pub const fn parent(self) -> Option<NodeId> {
        self.0
    }

    #[doc = include_str!("visual_parent.doc/is_root.md")]
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
