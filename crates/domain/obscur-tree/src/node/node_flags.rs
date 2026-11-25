//! Structural / behavioural flags for a retained node.

#[doc = include_str!("../../docs/api/node/node_flags.doc/NodeFlags.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeFlags(u32);

impl NodeFlags {
    /// Present in the tree but not visible / not painted.
    pub const HIDDEN: Self = Self(1 << 0);
    /// Hosts a portal: descendants may be painted under a different visual
    /// parent (spec §4.14).
    pub const PORTAL_HOST: Self = Self(1 << 1);
    /// Establishes a stacking context / compositing layer.
    pub const LAYER_ROOT: Self = Self(1 << 2);
    /// System-generated node, hidden from the component tree (spec §4.16).
    pub const SYNTHETIC: Self = Self(1 << 3);
    #[doc = include_str!("../../docs/api/node/node_flags.doc/COMPONENT_HOST.md")]
    pub const COMPONENT_HOST: Self = Self(1 << 4);
    #[doc = include_str!("../../docs/api/node/node_flags.doc/OPEN_STYLE_SCOPE.md")]
    pub const OPEN_STYLE_SCOPE: Self = Self(1 << 5);

    /// Returns an empty flag set.
    #[must_use]
    pub const fn empty() -> Self {
        Self(0)
    }

    /// Returns the raw bit pattern.
    #[must_use]
    pub const fn bits(self) -> u32 {
        self.0
    }

    /// Returns `true` if no flags are set.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Returns `true` if every flag in `other` is set in `self`.
    #[must_use]
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    /// Returns the union of `self` and `other`.
    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// Adds every flag in `other` to `self`.
    pub fn insert(&mut self, other: Self) {
        self.0 |= other.0;
    }

    /// Removes every flag in `other` from `self`.
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
