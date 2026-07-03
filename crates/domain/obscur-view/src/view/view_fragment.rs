//! `ViewFragment`: an ordered collection of desired view nodes.

use super::ViewNode;

/// An ordered sequence of [`ViewNode`]s.
///
/// A fragment is the output shape of a component render and of collection or
/// conditional constructs (`For`/`If`, spec §2.11), which produce nodes only
/// when their rendered output requires them. Recursion in the desired tree
/// arises because an element node's `children` is itself a `ViewFragment`.
///
/// Because nodes carry [`super::ViewValue`] (which contains an `f64`), this
/// type derives only `Debug`, `Clone`, and `PartialEq`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ViewFragment {
    nodes: Vec<ViewNode>,
}

impl ViewFragment {
    /// Creates an empty fragment.
    #[must_use]
    pub const fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    /// Creates a fragment from an existing vector of nodes.
    #[must_use]
    pub fn from_nodes(nodes: Vec<ViewNode>) -> Self {
        Self { nodes }
    }

    /// Appends a node to the end of the fragment.
    pub fn push(&mut self, node: ViewNode) {
        self.nodes.push(node);
    }

    /// Returns an iterator over the fragment's nodes in order.
    pub fn iter(&self) -> core::slice::Iter<'_, ViewNode> {
        self.nodes.iter()
    }

    /// Returns the number of nodes in the fragment.
    #[must_use]
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Returns `true` if the fragment contains no nodes.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Returns the fragment's nodes as a slice.
    #[must_use]
    pub fn nodes(&self) -> &[ViewNode] {
        &self.nodes
    }
}

impl<'a> IntoIterator for &'a ViewFragment {
    type Item = &'a ViewNode;
    type IntoIter = core::slice::Iter<'a, ViewNode>;

    fn into_iter(self) -> Self::IntoIter {
        self.nodes.iter()
    }
}

impl FromIterator<ViewNode> for ViewFragment {
    fn from_iter<I: IntoIterator<Item = ViewNode>>(iter: I) -> Self {
        Self {
            nodes: iter.into_iter().collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_identity::ComponentId;

    #[test]
    fn new_fragment_is_empty() {
        let fragment = ViewFragment::new();
        assert!(fragment.is_empty());
        assert_eq!(fragment.len(), 0);
    }

    #[test]
    fn push_and_iter_preserve_order() {
        let mut fragment = ViewFragment::new();
        fragment.push(ViewNode::text("a"));
        fragment.push(ViewNode::element(ComponentId::from_raw(1)));
        assert_eq!(fragment.len(), 2);
        assert_eq!(fragment.iter().count(), 2);
        assert_eq!(fragment.nodes().len(), 2);
    }

    #[test]
    fn from_nodes_and_from_iter_agree() {
        let nodes = vec![ViewNode::text("a"), ViewNode::text("b")];
        let from_nodes = ViewFragment::from_nodes(nodes.clone());
        let from_iter: ViewFragment = nodes.into_iter().collect();
        assert_eq!(from_nodes, from_iter);
    }
}
