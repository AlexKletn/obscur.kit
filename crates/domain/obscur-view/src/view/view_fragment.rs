#![doc = include_str!("view_fragment.doc/_module.md")]

use super::ViewNode;

#[doc = include_str!("view_fragment.doc/ViewFragment.md")]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ViewFragment {
    nodes: Vec<ViewNode>,
}

impl ViewFragment {
    #[doc = include_str!("view_fragment.doc/new.md")]
    #[must_use]
    pub const fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    #[doc = include_str!("view_fragment.doc/from_nodes.md")]
    #[must_use]
    pub fn from_nodes(nodes: Vec<ViewNode>) -> Self {
        Self { nodes }
    }

    #[doc = include_str!("view_fragment.doc/push.md")]
    pub fn push(&mut self, node: ViewNode) {
        self.nodes.push(node);
    }

    #[doc = include_str!("view_fragment.doc/iter.md")]
    pub fn iter(&self) -> core::slice::Iter<'_, ViewNode> {
        self.nodes.iter()
    }

    #[doc = include_str!("view_fragment.doc/len.md")]
    #[must_use]
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    #[doc = include_str!("view_fragment.doc/is_empty.md")]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    #[doc = include_str!("view_fragment.doc/nodes.md")]
    #[must_use]
    pub fn nodes(&self) -> &[ViewNode] {
        &self.nodes
    }

    #[doc = include_str!("view_fragment.doc/into_nodes.md")]
    #[must_use]
    pub fn into_nodes(self) -> Vec<ViewNode> {
        self.nodes
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
