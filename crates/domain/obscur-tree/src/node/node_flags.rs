#![doc = include_str!("node_flags.doc/_module.md")]

#[doc = include_str!("node_flags.doc/NodeFlags.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeFlags(u32);

impl NodeFlags {
    #[doc = include_str!("node_flags.doc/HIDDEN.md")]
    pub const HIDDEN: Self = Self(1 << 0);
    #[doc = include_str!("node_flags.doc/PORTAL_HOST.md")]
    pub const PORTAL_HOST: Self = Self(1 << 1);
    #[doc = include_str!("node_flags.doc/LAYER_ROOT.md")]
    pub const LAYER_ROOT: Self = Self(1 << 2);
    #[doc = include_str!("node_flags.doc/SYNTHETIC.md")]
    pub const SYNTHETIC: Self = Self(1 << 3);
    #[doc = include_str!("node_flags.doc/COMPONENT_HOST.md")]
    pub const COMPONENT_HOST: Self = Self(1 << 4);
    #[doc = include_str!("node_flags.doc/OPEN_STYLE_SCOPE.md")]
    pub const OPEN_STYLE_SCOPE: Self = Self(1 << 5);

    #[doc = include_str!("node_flags.doc/empty.md")]
    #[must_use]
    pub const fn empty() -> Self {
        Self(0)
    }

    #[doc = include_str!("node_flags.doc/bits.md")]
    #[must_use]
    pub const fn bits(self) -> u32 {
        self.0
    }

    #[doc = include_str!("node_flags.doc/is_empty.md")]
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    #[doc = include_str!("node_flags.doc/contains.md")]
    #[must_use]
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    #[doc = include_str!("node_flags.doc/union.md")]
    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    #[doc = include_str!("node_flags.doc/insert.md")]
    pub fn insert(&mut self, other: Self) {
        self.0 |= other.0;
    }

    #[doc = include_str!("node_flags.doc/remove.md")]
    pub fn remove(&mut self, other: Self) {
        self.0 &= !other.0;
    }
}

impl Default for NodeFlags {
    fn default() -> Self {
        Self::empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_contains_nothing() {
        let f = NodeFlags::empty();
        assert!(f.is_empty());
        assert!(!f.contains(NodeFlags::HIDDEN));
        assert!(f.contains(NodeFlags::empty()));
    }

    #[test]
    fn insert_remove_contains() {
        let mut f = NodeFlags::empty();
        f.insert(NodeFlags::HIDDEN);
        f.insert(NodeFlags::LAYER_ROOT);
        assert!(f.contains(NodeFlags::HIDDEN));
        assert!(f.contains(NodeFlags::LAYER_ROOT));
        assert!(!f.contains(NodeFlags::PORTAL_HOST));
        assert!(f.contains(NodeFlags::HIDDEN.union(NodeFlags::LAYER_ROOT)));

        f.remove(NodeFlags::HIDDEN);
        assert!(!f.contains(NodeFlags::HIDDEN));
        assert!(f.contains(NodeFlags::LAYER_ROOT));
    }

    #[test]
    fn union_is_nondestructive() {
        let a = NodeFlags::HIDDEN;
        let b = NodeFlags::SYNTHETIC;
        let u = a.union(b);
        assert!(u.contains(a));
        assert!(u.contains(b));
        assert!(!a.contains(b));
    }

    #[test]
    fn flags_are_distinct_bits() {
        let all = [
            NodeFlags::HIDDEN,
            NodeFlags::PORTAL_HOST,
            NodeFlags::LAYER_ROOT,
            NodeFlags::SYNTHETIC,
        ];
        let mut combined = 0u32;
        for f in all {
            assert_ne!(f.bits(), 0);
            assert_eq!(combined & f.bits(), 0, "flags overlap");
            combined |= f.bits();
        }
    }
}
