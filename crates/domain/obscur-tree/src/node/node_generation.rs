#![doc = include_str!("node_generation.doc/_module.md")]

use obscur_identity::Generation;

#[doc = include_str!("node_generation.doc/NodeGeneration.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeGeneration(Generation);

impl NodeGeneration {
    #[doc = include_str!("node_generation.doc/FIRST.md")]
    pub const FIRST: Self = Self(Generation::FIRST);

    #[doc = include_str!("node_generation.doc/new.md")]
    #[must_use]
    pub const fn new() -> Self {
        Self::FIRST
    }

    #[doc = include_str!("node_generation.doc/next.md")]
    #[must_use]
    pub const fn next(self) -> Self {
        Self(self.0.next())
    }

    #[doc = include_str!("node_generation.doc/raw.md")]
    #[must_use]
    pub const fn raw(self) -> u32 {
        self.0.raw()
    }
}

impl Default for NodeGeneration {
    fn default() -> Self {
        Self::FIRST
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_at_first() {
        assert_eq!(NodeGeneration::new(), NodeGeneration::FIRST);
        assert_eq!(NodeGeneration::default(), NodeGeneration::FIRST);
    }

    #[test]
    fn next_advances_and_orders() {
        let g0 = NodeGeneration::FIRST;
        let g1 = g0.next();
        assert_ne!(g0, g1);
        assert!(g1 > g0);
        assert_eq!(g1.raw(), g0.raw() + 1);
    }
}
