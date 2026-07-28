//! `View`: the root of a component's desired output.

use super::{ViewFragment, ViewNode};

/// The root of a component's desired declarative output for one render.
///
/// A component may return zero, one, or many top-level nodes, so `View` wraps a
/// root [`ViewFragment`] rather than a single node. The whole structure is
/// plain data (spec §2.2): cloneable, comparable, and free of retained-tree
/// references, runtime handles, and closures, which keeps it suitable for
/// reconciliation and later lowering to UI IR (spec §2.15).
///
/// Because nodes carry [`super::ViewValue`] (which contains an `f64`), this
/// type derives only `Debug`, `Clone`, and `PartialEq`.
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

    /// Consumes this `View`, returning its sole top-level [`ViewNode`], or [`None`]
    /// unless there is exactly one.
    ///
    /// A widget's `view()` (and `Button::new(..).view()`) produces a single-root
    /// `View`; this is the one call that turns such a view back into a placeable,
    /// stylable node, replacing the `into_fragment().into_nodes()` + take-the-only
    /// -element dance. `None` for an empty or multi-root view keeps the caller
    /// honest about the shape it assumed rather than silently taking the first.
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
