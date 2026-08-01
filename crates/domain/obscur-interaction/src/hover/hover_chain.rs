#![doc = include_str!("hover_chain.doc/_module.md")]

use std::slice::Iter;

use obscur_identity::NodeId;

#[doc = include_str!("hover_chain.doc/HoverChain.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HoverChain {
    nodes: Vec<NodeId>,
}

impl HoverChain {
    #[doc = include_str!("hover_chain.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    #[doc = include_str!("hover_chain.doc/from_nodes.md")]
    #[must_use]
    pub fn from_nodes(nodes: Vec<NodeId>) -> Self {
        Self { nodes }
    }

    #[doc = include_str!("hover_chain.doc/topmost.md")]
    #[must_use]
    pub fn topmost(&self) -> Option<NodeId> {
        self.nodes.first().copied()
    }

    #[doc = include_str!("hover_chain.doc/nodes.md")]
    #[must_use]
    pub fn nodes(&self) -> &[NodeId] {
        &self.nodes
    }

    #[doc = include_str!("hover_chain.doc/contains.md")]
    #[must_use]
    pub fn contains(&self, node: NodeId) -> bool {
        self.nodes.contains(&node)
    }

    #[doc = include_str!("hover_chain.doc/len.md")]
    #[must_use]
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    #[doc = include_str!("hover_chain.doc/is_empty.md")]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    #[doc = include_str!("hover_chain.doc/iter.md")]
    pub fn iter(&self) -> Iter<'_, NodeId> {
        self.nodes.iter()
    }
}

impl<'a> IntoIterator for &'a HoverChain {
    type Item = &'a NodeId;
    type IntoIter = Iter<'a, NodeId>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl Default for HoverChain {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(raw: u64) -> NodeId {
        NodeId::from_raw(raw)
    }

    #[test]
    fn empty_chain_has_no_topmost() {
        let c = HoverChain::new();
        assert!(c.is_empty());
        assert_eq!(c.len(), 0);
        assert_eq!(c.topmost(), None);
        assert_eq!(c, HoverChain::default());
    }

    #[test]
    fn topmost_is_index_zero() {
        let c = HoverChain::from_nodes(vec![node(1), node(2), node(3)]);
        assert_eq!(c.topmost(), Some(node(1)));
        assert_eq!(c.len(), 3);
        assert_eq!(c.nodes(), &[node(1), node(2), node(3)]);
    }

    #[test]
    fn contains_finds_membership() {
        let c = HoverChain::from_nodes(vec![node(1), node(2)]);
        assert!(c.contains(node(1)));
        assert!(c.contains(node(2)));
        assert!(!c.contains(node(9)));
    }

    #[test]
    fn iter_matches_nodes() {
        let c = HoverChain::from_nodes(vec![node(1), node(2)]);
        let via_iter: Vec<_> = c.iter().copied().collect();
        assert_eq!(via_iter.as_slice(), c.nodes());
    }
}
