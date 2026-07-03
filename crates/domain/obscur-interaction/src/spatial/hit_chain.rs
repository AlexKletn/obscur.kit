//! The [`HitChain`] mapping visual geometry back to the logical tree
//! (spec §4.7).

use std::slice::Iter;

use obscur_identity::NodeId;

use super::HitTest;

/// The ordered chain of hits from the topmost (target) node down to the root
/// (spec §4.7).
///
/// A hit chain is the result of a hit-test query: it maps a point in visual
/// geometry back to the logical event tree. Index `0` is the topmost node that
/// was struck (the event target); the last element is the root. Events
/// propagate along this chain (capture from root toward target, bubble back).
#[derive(Debug, Clone, PartialEq)]
pub struct HitChain {
    hits: Vec<HitTest>,
}

impl HitChain {
    /// Creates an empty hit chain.
    #[must_use]
    pub fn new() -> Self {
        Self { hits: Vec::new() }
    }

    /// Builds a hit chain from an existing sequence of hits, ordered
    /// topmost-first (index `0`) toward the root (last).
    #[must_use]
    pub fn from_hits(hits: Vec<HitTest>) -> Self {
        Self { hits }
    }

    /// Appends `hit` toward the root end of the chain.
    pub fn push(&mut self, hit: HitTest) {
        self.hits.push(hit);
    }

    /// The topmost hit — the event target — or `None` if the chain is empty.
    #[must_use]
    pub fn target(&self) -> Option<&HitTest> {
        self.hits.first()
    }

    /// The root hit — the deepest ancestor — or `None` if the chain is empty.
    #[must_use]
    pub fn root(&self) -> Option<&HitTest> {
        self.hits.last()
    }

    /// The nodes along the chain, topmost-first through root.
    #[must_use]
    pub fn nodes(&self) -> Vec<NodeId> {
        self.hits.iter().map(HitTest::node).collect()
    }

    /// Returns whether `node` appears anywhere in the chain.
    #[must_use]
    pub fn contains_node(&self, node: NodeId) -> bool {
        self.hits.iter().any(|hit| hit.node() == node)
    }

    /// The number of hits in the chain.
    #[must_use]
    pub fn len(&self) -> usize {
        self.hits.len()
    }

    /// Returns whether the chain is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.hits.is_empty()
    }

    /// The hits as a slice, topmost-first through root.
    #[must_use]
    pub fn hits(&self) -> &[HitTest] {
        &self.hits
    }

    /// Iterates over the hits, topmost-first through root.
    pub fn iter(&self) -> Iter<'_, HitTest> {
        self.hits.iter()
    }
}

impl Default for HitChain {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_geometry::Point;

    fn node(raw: u64) -> NodeId {
        NodeId::from_raw(raw)
    }

    fn hit(raw: u64, z: i32) -> HitTest {
        HitTest::new(node(raw), Point::new(0.0, 0.0), z)
    }

    #[test]
    fn empty_chain_has_no_target_or_root() {
        let c = HitChain::new();
        assert!(c.is_empty());
        assert_eq!(c.len(), 0);
        assert_eq!(c.target(), None);
        assert_eq!(c.root(), None);
        assert_eq!(c, HitChain::default());
    }

    #[test]
    fn target_is_topmost_root_is_last() {
        let mut c = HitChain::new();
        c.push(hit(1, 30)); // topmost / target
        c.push(hit(2, 20));
        c.push(hit(3, 10)); // root
        assert_eq!(c.target(), Some(&hit(1, 30)));
        assert_eq!(c.root(), Some(&hit(3, 10)));
        assert_eq!(c.len(), 3);
    }

    #[test]
    fn nodes_are_topmost_to_root() {
        let c = HitChain::from_hits(vec![hit(1, 30), hit(2, 20), hit(3, 10)]);
        assert_eq!(c.nodes(), vec![node(1), node(2), node(3)]);
    }

    #[test]
    fn contains_node_finds_membership() {
        let c = HitChain::from_hits(vec![hit(1, 30), hit(2, 20)]);
        assert!(c.contains_node(node(1)));
        assert!(c.contains_node(node(2)));
        assert!(!c.contains_node(node(99)));
    }

    #[test]
    fn iter_and_hits_agree() {
        let c = HitChain::from_hits(vec![hit(1, 30), hit(2, 20)]);
        let via_iter: Vec<_> = c.iter().copied().collect();
        assert_eq!(via_iter.as_slice(), c.hits());
    }
}
