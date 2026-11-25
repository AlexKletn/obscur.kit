//! The logical ancestor chain of a node: [`AncestorChain`].

use obscur_identity::NodeId;
use obscur_tree::tree::RetainedTree;

#[doc = include_str!("../../docs/api/runtime/ancestor_chain.doc/AncestorChain.md")]
pub(crate) struct AncestorChain(Vec<NodeId>);

impl AncestorChain {
    #[doc = include_str!("../../docs/api/runtime/ancestor_chain.doc/of.md")]
    pub(crate) fn of(tree: &RetainedTree, target: Option<NodeId>) -> Self {
        let Some(target) = target else {
            return Self(Vec::new());
        };
        let mut chain = vec![target];
        let mut current = target;
        let cap = tree.len();
        for _ in 0..cap {
            match tree.get(current).and_then(|n| n.logical_parent().parent()) {
                Some(parent) => {
                    chain.push(parent);
                    current = parent;
                }
                None => break,
            }
        }
        Self(chain)
    }

    /// The chain as an owned vector, node first.
    pub(crate) fn into_vec(self) -> Vec<NodeId> {
        self.0
    }
}

impl IntoIterator for AncestorChain {
    type Item = NodeId;
    type IntoIter = std::vec::IntoIter<NodeId>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nothing_hovered_yields_an_empty_chain() {
        let tree = RetainedTree::new();
        assert!(AncestorChain::of(&tree, None).into_vec().is_empty());
    }

    #[test]
    fn a_node_that_is_not_in_the_tree_is_still_the_head_of_its_own_chain() {
        // The chain always starts at the target, even when the tree cannot resolve
        // it: a caller asking "is this disabled" about a stale id gets a one-element
        // answer rather than an empty one, and every predicate over the chain then
        // reads the same "unknown node" as "no ancestors".
        let tree = RetainedTree::new();
        let stale = NodeId::from_raw(7);
        assert_eq!(
            AncestorChain::of(&tree, Some(stale)).into_vec(),
            vec![stale]
        );
    }
}
