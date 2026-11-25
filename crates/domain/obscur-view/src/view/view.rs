//! `View`: the root of a component's desired output.

use super::{ViewFragment, ViewNode};

#[doc = include_str!("../../docs/api/view/view.doc/View.md")]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct View {
    root: ViewFragment,
}

impl View {
    /// Creates a `View` producing no top-level nodes.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            root: ViewFragment::new(),
        }
    }

    /// Creates a `View` with a single top-level node.
    #[must_use]
    pub fn single(node: ViewNode) -> Self {
        Self {
            root: ViewFragment::from_nodes(vec![node]),
        }
    }

    /// Creates a `View` from an existing root fragment.
    #[must_use]
    pub fn from_fragment(root: ViewFragment) -> Self {
        Self { root }
    }

    /// Returns the root fragment of top-level nodes.
    #[must_use]
    pub const fn root(&self) -> &ViewFragment {
        &self.root
    }

    /// Consumes this `View`, returning its root fragment.
    #[must_use]
    pub fn into_fragment(self) -> ViewFragment {
        self.root
    }

    #[doc = include_str!("../../docs/api/view/view.doc/into_single_node.md")]
    #[must_use]
    pub fn into_single_node(self) -> Option<ViewNode> {
        let mut nodes = self.root.into_nodes();
        if nodes.len() == 1 { nodes.pop() } else { None }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_identity::ComponentId;

    #[test]
    fn empty_has_no_nodes() {
        assert!(View::empty().root().is_empty());
    }

    #[test]
    fn single_wraps_one_node() {
        let view = View::single(ViewNode::element(ComponentId::from_raw(1)));
        assert_eq!(view.root().len(), 1);
    }

    #[test]
    fn from_fragment_round_trips() {
        let mut fragment = ViewFragment::new();
        fragment.push(ViewNode::text("a"));
        let view = View::from_fragment(fragment.clone());
        assert_eq!(view.into_fragment(), fragment);
    }

    #[test]
    fn into_single_node_yields_the_only_node() {
        let node = ViewNode::element(ComponentId::from_raw(1));
        assert_eq!(View::single(node.clone()).into_single_node(), Some(node));
    }

    #[test]
    fn into_single_node_is_none_when_empty_or_multi() {
        assert_eq!(View::empty().into_single_node(), None);
        let fragment = ViewFragment::from_nodes(vec![ViewNode::text("a"), ViewNode::text("b")]);
        assert_eq!(View::from_fragment(fragment).into_single_node(), None);
    }
}
