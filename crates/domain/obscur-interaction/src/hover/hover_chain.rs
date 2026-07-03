//! The [`HoverChain`] — the ordered chain of nodes under the pointer, topmost
//! toward root (hover; parallels the hit chain in spec §4.7).

use std::slice::Iter;

use obscur_identity::NodeId;

/// The ordered chain of nodes currently under the pointer, topmost-first
/// (hover).
///
/// This parallels the [`HitChain`](crate::spatial::HitChain): index `0` is the
/// topmost node the pointer is over (the hover target); the last element is the
/// root. Comparing successive hover chains yields the enter/leave diff via
/// [`HoverTransition`](super::HoverTransition).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HoverChain {
    nodes: Vec<NodeId>,
}

impl HoverChain {
    /// Creates an empty hover chain (the pointer is over nothing).
    #[must_use]
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    /// Builds a hover chain from `nodes`, ordered topmost-first (index `0`)
    /// toward the root (last).
    #[must_use]
    pub fn from_nodes(nodes: Vec<NodeId>) -> Self {
        Self { nodes }
    }

    /// The topmost node — the hover target — or `None` if the chain is empty.
    #[must_use]
    pub fn topmost(&self) -> Option<NodeId> {
        self.nodes.first().copied()
    }

    /// The nodes along the chain, topmost-first through root.
    #[must_use]
    pub fn nodes(&self) -> &[NodeId] {
        &self.nodes
    }

    /// Returns whether `node` appears anywhere in the chain.
    #[must_use]
    pub fn contains(&self, node: NodeId) -> bool {
        self.nodes.contains(&node)
    }

    /// The number of nodes in the chain.
    #[must_use]
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Returns whether the chain is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Iterates over the nodes, topmost-first through root.
    pub fn iter(&self) -> Iter<'_, NodeId> {
        self.nodes.iter()
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
