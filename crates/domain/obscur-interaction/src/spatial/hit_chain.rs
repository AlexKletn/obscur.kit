#![doc = include_str!("hit_chain.doc/_module.md")]

use std::slice::Iter;

use obscur_identity::NodeId;

use super::HitTest;

#[doc = include_str!("hit_chain.doc/HitChain.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct HitChain {
    hits: Vec<HitTest>,
}

impl HitChain {
    #[doc = include_str!("hit_chain.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        Self { hits: Vec::new() }
    }

    #[doc = include_str!("hit_chain.doc/from_hits.md")]
    #[must_use]
    pub fn from_hits(hits: Vec<HitTest>) -> Self {
        Self { hits }
    }

    #[doc = include_str!("hit_chain.doc/push.md")]
    pub fn push(&mut self, hit: HitTest) {
        self.hits.push(hit);
    }

    #[doc = include_str!("hit_chain.doc/target.md")]
    #[must_use]
    pub fn target(&self) -> Option<&HitTest> {
        self.hits.first()
    }

    #[doc = include_str!("hit_chain.doc/root.md")]
    #[must_use]
    pub fn root(&self) -> Option<&HitTest> {
        self.hits.last()
    }

    #[doc = include_str!("hit_chain.doc/nodes.md")]
    #[must_use]
    pub fn nodes(&self) -> Vec<NodeId> {
        self.hits.iter().map(HitTest::node).collect()
    }

    #[doc = include_str!("hit_chain.doc/contains_node.md")]
    #[must_use]
    pub fn contains_node(&self, node: NodeId) -> bool {
        self.hits.iter().any(|hit| hit.node() == node)
    }

    #[doc = include_str!("hit_chain.doc/len.md")]
    #[must_use]
    pub fn len(&self) -> usize {
        self.hits.len()
    }

    #[doc = include_str!("hit_chain.doc/is_empty.md")]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.hits.is_empty()
    }

    #[doc = include_str!("hit_chain.doc/hits.md")]
    #[must_use]
    pub fn hits(&self) -> &[HitTest] {
        &self.hits
    }

    #[doc = include_str!("hit_chain.doc/iter.md")]
    pub fn iter(&self) -> Iter<'_, HitTest> {
        self.hits.iter()
    }
}

impl<'a> IntoIterator for &'a HitChain {
    type Item = &'a HitTest;
    type IntoIter = Iter<'a, HitTest>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
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
