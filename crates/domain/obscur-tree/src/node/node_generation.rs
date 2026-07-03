//! Per-node generation used to detect stale `NodeId` references.

use obscur_identity::Generation;

/// A per-node generation counter.
///
/// The retained tree is an arena: when a slot's `NodeId` index is reused for a
/// new node, this generation is advanced so that references minted against the
/// previous occupant can be recognised as stale (spec §1.5). It wraps the shared
/// [`obscur_identity::Generation`] primitive so the whole workspace agrees on a
/// single generation representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeGeneration(Generation);

impl NodeGeneration {
    /// The first live generation for a freshly created node.
    pub const FIRST: Self = Self(Generation::FIRST);

    /// Creates a node generation at [`NodeGeneration::FIRST`].
    #[must_use]
    pub const fn new() -> Self {
        Self::FIRST
    }

    /// Returns the next generation, used when a slot is reused.
    #[must_use]
    pub const fn next(self) -> Self {
        Self(self.0.next())
    }

    /// Returns the raw generation value.
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
