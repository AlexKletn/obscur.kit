//! The selector-matcher's view of a retained-tree node: [`TreeElement`].

use obscur_identity::NodeId;
use obscur_style::selector::{MatchElement, PseudoState};
use obscur_tree::node::RetainedNode;
use obscur_tree::tree::RetainedTree;

/// A cheap cursor over one [`RetainedNode`] that implements
/// [`MatchElement`](obscur_style::selector::MatchElement), the seam the
/// `obscur-style` selector matcher walks.
///
/// A `TreeElement` is a shared reference to the [`RetainedTree`] plus the
/// [`NodeId`] it points at, so it is `Copy` and allocation-free: navigation
/// methods ([`parent`](MatchElement::parent),
/// [`previous_sibling_element`](MatchElement::previous_sibling_element)) mint
/// new cursors over the same borrowed tree rather than cloning node data. This
/// is the integrating adapter that lets authored stylesheets match against the
/// retained tree without `obscur-style` depending on `obscur-tree`.
///
/// # First-iteration limitations
///
/// - **Attributes** — the retained tree only preserves an element's `class`,
///   `id`, and *type* (local name) via
///   [`StyleAttributes`](obscur_view::view::StyleAttributes). Consequently
///   [`attribute`](MatchElement::attribute) resolves only `"id"` and `"type"`;
///   any other attribute selector (e.g. `[data-*]`, `[aria-*]`) matches nothing
///   until the tree retains arbitrary attributes.
/// - **Pseudo-state** — interaction-driven states (`:hover`, `:active`,
///   `:focus`, …) are not sourced yet, so
///   [`pseudo_state`](MatchElement::pseudo_state) always reports
///   [`PseudoState::empty`]. Wiring live interaction state is a later concern.
/// - **Top-level siblings** — an element whose logical parent is the synthetic
///   [`Root`](obscur_tree::node::NodeKind::Root) grouping node is treated as
///   top-level: it reports no [`parent`](MatchElement::parent) (so `:root` and
///   child/descendant combinators behave) and is treated as the sole member of
///   its sibling list (index `0`, count `1`).
#[derive(Debug, Clone, Copy)]
pub struct TreeElement<'a> {
    /// The tree the cursor navigates; shared and borrowed for `'a`.
    tree: &'a RetainedTree,
    /// The node this cursor currently points at.
    id: NodeId,
}

impl<'a> TreeElement<'a> {
    /// Creates a cursor over `id` within `tree`.
    #[must_use]
    pub const fn new(tree: &'a RetainedTree, id: NodeId) -> Self {
        Self { tree, id }
    }

    /// Returns the node id this cursor points at.
    ///
    /// Named `node_id` rather than `id` so it does not shadow the
    /// [`MatchElement::id`] trait method (which returns the element's CSS `id`
    /// attribute as `Option<&str>`).
    #[must_use]
    pub const fn node_id(&self) -> NodeId {
        self.id
    }

    /// Resolves the backing node record, or `None` if the id is stale/absent.
    fn node(&self) -> Option<&'a RetainedNode> {
        self.tree.get(self.id)
    }

    /// Returns the ordered element siblings this node participates in
    /// (including itself), or a single-element list of just this node when the
    /// node is top-level (its element parent is the synthetic root or absent).
    ///
    /// Only [`Element`](obscur_tree::node::NodeKind::Element) children count as
    /// siblings; text, synthetic, and root nodes are excluded, matching CSS's
    /// "element sibling" semantics.
    fn element_siblings(&self) -> Vec<NodeId> {
        let Some(parent) = self.parent() else {
            return vec![self.id];
        };
        match parent.node() {
            Some(parent_node) => parent_node
                .children()
                .iter()
                .copied()
                .filter(|cid| self.tree.get(*cid).is_some_and(|c| c.kind().is_element()))
                .collect(),
            None => vec![self.id],
        }
    }
}

impl MatchElement for TreeElement<'_> {
    fn local_name(&self) -> Option<&str> {
        self.node().and_then(|n| n.attributes().local_name())
    }

    fn id(&self) -> Option<&str> {
        self.node().and_then(|n| n.attributes().id())
    }

    fn has_class(&self, class: &str) -> bool {
        self.node().is_some_and(|n| n.attributes().has_class(class))
    }

    /// Resolves only the attributes the retained tree preserves: `"id"` and
    /// `"type"` (the local name), both matched ASCII case-insensitively. Every
    /// other attribute name returns `None` — see the type-level note on
    /// first-iteration attribute limitations.
    fn attribute(&self, name: &str) -> Option<&str> {
        if name.eq_ignore_ascii_case("id") {
            self.id()
        } else if name.eq_ignore_ascii_case("type") {
            self.local_name()
        } else {
            None
        }
    }

    /// Always [`PseudoState::empty`]: interaction-driven states are not sourced
    /// yet (see the type-level note).
    fn pseudo_state(&self) -> PseudoState {
        PseudoState::empty()
    }

    /// Returns the logical parent **only when it is an element node**. When the
    /// parent is the synthetic root (or the node is detached), returns `None`,
    /// so the node reads as top-level and `:root`/child combinators behave.
    fn parent(&self) -> Option<Self> {
        let parent_id = self.node()?.logical_parent().parent()?;
        let parent_node = self.tree.get(parent_id)?;
        parent_node
            .kind()
            .is_element()
            .then(|| TreeElement::new(self.tree, parent_id))
    }

    fn previous_sibling_element(&self) -> Option<Self> {
        let siblings = self.element_siblings();
        let pos = siblings.iter().position(|s| *s == self.id)?;
        let prev = pos.checked_sub(1)?;
        siblings
            .get(prev)
            .map(|prev_id| TreeElement::new(self.tree, *prev_id))
    }

    fn index_among_siblings(&self) -> usize {
        self.element_siblings()
            .iter()
            .position(|s| *s == self.id)
            .unwrap_or(0)
    }

    fn sibling_count(&self) -> usize {
        self.element_siblings().len().max(1)
    }

    /// A node is the tree root (`:root`) when it has no element parent — its
    /// logical parent is the synthetic root or it is detached.
    fn is_root(&self) -> bool {
        self.parent().is_none()
    }

    /// Reports whether the node has any child node at all. Text children are
    /// real nodes here, so an element containing only text is *not* `:empty`.
    fn has_children(&self) -> bool {
        self.node().is_some_and(|n| !n.children().is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_identity::ComponentId;
    use obscur_tree::reconciliation::Reconciler;
    use obscur_view::view::{
        CLASS, ID, TYPE, View, ViewFragment, ViewNode, ViewProperty, ViewValue,
    };

    fn text_prop(name: obscur_identity::InternedSymbol, value: &str) -> ViewProperty {
        ViewProperty::new(name, ViewValue::Text(value.into()))
    }

    /// Builds a retained tree from a desired view via the reconciler.
    fn reconcile(view: &View) -> RetainedTree {
        let mut tree = RetainedTree::new();
        let _ = Reconciler::new().reconcile_root(&mut tree, view);
        tree
    }

    /// Returns the root's ordered children in the retained tree.
    fn root_children(tree: &RetainedTree) -> Vec<NodeId> {
        let root = tree.root().expect("tree has a root after reconcile");
        tree.get(root)
            .map(|n| n.children().to_vec())
            .unwrap_or_default()
    }

    #[test]
    fn reports_local_name_id_and_class() {
        let view = View::from_fragment(ViewFragment::from_nodes(vec![
            ViewNode::element(ComponentId::from_raw(10))
                .with_prop(text_prop(TYPE, "container"))
                .with_prop(text_prop(ID, "main"))
                .with_prop(text_prop(CLASS, "counter primary")),
        ]));
        let tree = reconcile(&view);
        let child = root_children(&tree)[0];
        let element = TreeElement::new(&tree, child);

        assert_eq!(element.local_name(), Some("container"));
        assert_eq!(element.id(), Some("main"));
        assert!(element.has_class("counter"));
        assert!(element.has_class("primary"));
        assert!(!element.has_class("missing"));
    }

    #[test]
    fn attribute_resolves_id_and_type_only() {
        let view = View::from_fragment(ViewFragment::from_nodes(vec![
            ViewNode::element(ComponentId::from_raw(10))
                .with_prop(text_prop(TYPE, "button"))
                .with_prop(text_prop(ID, "ok")),
        ]));
        let tree = reconcile(&view);
        let child = root_children(&tree)[0];
        let element = TreeElement::new(&tree, child);

        assert_eq!(element.attribute("id"), Some("ok"));
        assert_eq!(element.attribute("ID"), Some("ok"));
        assert_eq!(element.attribute("type"), Some("button"));
        assert_eq!(element.attribute("data-foo"), None);
    }

    #[test]
    fn top_level_element_is_root_and_single_sibling() {
        let view = View::from_fragment(ViewFragment::from_nodes(vec![ViewNode::element(
            ComponentId::from_raw(10),
        )]));
        let tree = reconcile(&view);
        let child = root_children(&tree)[0];
        let element = TreeElement::new(&tree, child);

        // Logical parent is the synthetic Root, so parent() is None and the node
        // reads as top-level: :root holds and it is its own sole sibling.
        assert!(element.parent().is_none());
        assert!(element.is_root());
        assert_eq!(element.index_among_siblings(), 0);
        assert_eq!(element.sibling_count(), 1);
        assert!(element.previous_sibling_element().is_none());
        assert!(element.is_only_child());
        assert_eq!(element.pseudo_state(), PseudoState::empty());
    }

    #[test]
    fn sibling_navigation_across_container_children() {
        // A container element with three element children.
        let container = ViewNode::element(ComponentId::from_raw(1)).with_children(
            ViewFragment::from_nodes(vec![
                ViewNode::element(ComponentId::from_raw(11)),
                ViewNode::element(ComponentId::from_raw(12)),
                ViewNode::element(ComponentId::from_raw(13)),
            ]),
        );
        let view = View::from_fragment(ViewFragment::from_nodes(vec![container]));
        let tree = reconcile(&view);

        let container_id = root_children(&tree)[0];
        let container_node = tree.get(container_id).expect("container exists");
        let kids: Vec<NodeId> = container_node.children().to_vec();
        assert_eq!(kids.len(), 3, "container reconciled three element children");

        let first = TreeElement::new(&tree, kids[0]);
        let middle = TreeElement::new(&tree, kids[1]);
        let last = TreeElement::new(&tree, kids[2]);

        // The container is an element parent, so children get real navigation.
        assert_eq!(first.parent().map(|p| p.node_id()), Some(container_id));
        assert!(!first.is_root());

        assert_eq!(first.index_among_siblings(), 0);
        assert_eq!(middle.index_among_siblings(), 1);
        assert_eq!(last.index_among_siblings(), 2);

        assert_eq!(first.sibling_count(), 3);
        assert_eq!(middle.sibling_count(), 3);

        assert!(first.previous_sibling_element().is_none());
        assert_eq!(
            middle.previous_sibling_element().map(|e| e.node_id()),
            Some(kids[0])
        );
        assert_eq!(
            last.previous_sibling_element().map(|e| e.node_id()),
            Some(kids[1])
        );

        assert!(
            container_node
                .children()
                .iter()
                .all(|c| tree.get(*c).is_some())
        );
        assert!(TreeElement::new(&tree, container_id).has_children());
        assert!(!first.has_children());
    }
}
