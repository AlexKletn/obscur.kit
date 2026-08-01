#![doc = include_str!("subtree.doc/_module.md")]

use std::collections::HashSet;

use obscur_identity::NodeId;

use super::RetainedTree;

#[doc = include_str!("subtree.doc/Subtree.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Subtree {
    root: NodeId,
}

impl Subtree {
    #[doc = include_str!("subtree.doc/new.md")]
    #[must_use]
    pub const fn new(root: NodeId) -> Self {
        Self { root }
    }

    #[doc = include_str!("subtree.doc/root.md")]
    #[must_use]
    pub const fn root(&self) -> NodeId {
        self.root
    }

    #[doc = include_str!("subtree.doc/descendants.md")]
    #[must_use]
    pub fn descendants(&self, tree: &RetainedTree) -> Vec<NodeId> {
        let mut order = Vec::new();
        let mut visited: HashSet<NodeId> = HashSet::new();
        let mut stack = vec![self.root];
        while let Some(id) = stack.pop() {
            if !visited.insert(id) {
                continue;
            }
            let Some(node) = tree.get(id) else {
                continue;
            };
            order.push(id);
            // Push children reversed so they pop in link order (pre-order DFS).
            for &child in node.child_nodes().iter().rev() {
                if !visited.contains(&child) {
                    stack.push(child);
                }
            }
        }
        order
    }

    #[doc = include_str!("subtree.doc/contains.md")]
    #[must_use]
    pub fn contains(&self, tree: &RetainedTree, id: NodeId) -> bool {
        self.descendants(tree).contains(&id)
    }

    #[doc = include_str!("subtree.doc/len.md")]
    #[must_use]
    pub fn len(&self, tree: &RetainedTree) -> usize {
        self.descendants(tree).len()
    }

    #[doc = include_str!("subtree.doc/is_empty.md")]
    #[must_use]
    pub fn is_empty(&self, tree: &RetainedTree) -> bool {
        tree.get(self.root).is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::NodeKind;

    /// Builds a small tree and wires parent -> child links, returning the ids.
    fn linear_chain(tree: &mut RetainedTree, n: usize) -> Vec<NodeId> {
        let ids: Vec<NodeId> = (0..n).map(|_| tree.alloc(NodeKind::Root)).collect();
        for pair in ids.windows(2) {
            if let [parent, child] = pair
                && let Some(node) = tree.get_mut(*parent)
            {
                node.push_child(*child);
            }
        }
        ids
    }

    #[test]
    fn descendants_are_preorder() {
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        let a = tree.alloc(NodeKind::Root);
        let b = tree.alloc(NodeKind::Root);
        let a1 = tree.alloc(NodeKind::Root);
        // root -> [a, b]; a -> [a1]
        if let Some(n) = tree.get_mut(root) {
            n.push_child(a);
            n.push_child(b);
        }
        if let Some(n) = tree.get_mut(a) {
            n.push_child(a1);
        }
        let sub = Subtree::new(root);
        assert_eq!(sub.descendants(&tree), vec![root, a, a1, b]);
        assert_eq!(sub.len(&tree), 4);
        assert!(sub.contains(&tree, a1));
        assert!(!sub.is_empty(&tree));
    }

    #[test]
    fn missing_root_yields_empty() {
        let mut tree = RetainedTree::new();
        let id = tree.alloc(NodeKind::Root);
        tree.remove(id);
        let sub = Subtree::new(id);
        assert!(sub.descendants(&tree).is_empty());
        assert_eq!(sub.len(&tree), 0);
        assert!(sub.is_empty(&tree));
        assert!(!sub.contains(&tree, id));
    }

    #[test]
    fn dangling_child_link_is_skipped() {
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        let ghost = NodeId::from_raw(u64::MAX);
        if let Some(n) = tree.get_mut(root) {
            n.push_child(ghost);
        }
        let sub = Subtree::new(root);
        assert_eq!(sub.descendants(&tree), vec![root]);
    }

    #[test]
    fn cyclic_links_terminate() {
        let mut tree = RetainedTree::new();
        let ids = linear_chain(&mut tree, 3);
        // Close the cycle: last node points back to the first.
        if let Some(n) = tree.get_mut(ids[2]) {
            n.push_child(ids[0]);
        }
        let sub = Subtree::new(ids[0]);
        let visited = sub.descendants(&tree);
        // Each node visited exactly once despite the cycle.
        assert_eq!(visited, ids);
        assert_eq!(sub.len(&tree), 3);
    }
}
