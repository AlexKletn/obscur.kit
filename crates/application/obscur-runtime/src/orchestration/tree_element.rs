//! The selector-matcher's view of a retained-tree node: [`TreeElement`].

use obscur_identity::NodeId;
use obscur_style::selector::{MatchElement, PseudoState};
use obscur_tree::node::RetainedNode;
use obscur_tree::tree::RetainedTree;

use super::PseudoStates;

#[doc = include_str!("../../docs/api/orchestration/tree_element.doc/TreeElement.md")]
#[derive(Debug, Clone, Copy)]
pub struct TreeElement<'a> {
    /// The tree the cursor navigates; shared and borrowed for `'a`.
    tree: &'a RetainedTree,
    /// The node this cursor currently points at.
    id: NodeId,
    /// The interaction-state source, or `None` for stateless matching (every
    /// node then reports [`PseudoState::empty`]).
    pseudo: Option<&'a PseudoStates>,
}

impl<'a> TreeElement<'a> {
    #[doc = include_str!("../../docs/api/orchestration/tree_element.doc/new.md")]
    #[must_use]
    pub const fn new(tree: &'a RetainedTree, id: NodeId) -> Self {
        Self {
            tree,
            id,
            pseudo: None,
        }
    }

    #[doc = include_str!("../../docs/api/orchestration/tree_element.doc/with_pseudo.md")]
    #[must_use]
    pub const fn with_pseudo(tree: &'a RetainedTree, id: NodeId, pseudo: &'a PseudoStates) -> Self {
        Self {
            tree,
            id,
            pseudo: Some(pseudo),
        }
    }

    /// Mints a sibling cursor over `id` in the same tree, preserving the
    /// interaction-state source so pseudo-class matching survives navigation.
    const fn cursor(&self, id: NodeId) -> Self {
        Self {
            tree: self.tree,
            id,
            pseudo: self.pseudo,
        }
    }

    #[doc = include_str!("../../docs/api/orchestration/tree_element.doc/node_id.md")]
    #[must_use]
    pub const fn node_id(&self) -> NodeId {
        self.id
    }

    /// Resolves the backing node record, or `None` if the id is stale/absent.
    fn node(&self) -> Option<&'a RetainedNode> {
        self.tree.get(self.id)
    }

    #[doc = include_str!("../../docs/api/orchestration/tree_element.doc/element_siblings.md")]
    fn element_siblings(&self) -> Vec<NodeId> {
        let Some(parent) = self.parent() else {
            return vec![self.id];
        };
        match parent.node() {
            Some(parent_node) => parent_node
                .child_nodes()
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

    #[doc = include_str!("../../docs/api/orchestration/tree_element.doc/enumerate_classes.md")]
    fn enumerate_classes(&self, visit: &mut dyn FnMut(&str)) -> bool {
        if let Some(node) = self.node() {
            for class in node.attributes().classes() {
                visit(class);
            }
        }
        true
    }

    #[doc = include_str!("../../docs/api/orchestration/tree_element.doc/attribute.md")]
    fn attribute(&self, name: &str) -> Option<&str> {
        if name.eq_ignore_ascii_case("id") {
            self.id()
        } else if name.eq_ignore_ascii_case("type") {
            self.node().and_then(|n| n.attributes().control_type())
        } else {
            None
        }
    }

    #[doc = include_str!("../../docs/api/orchestration/tree_element.doc/pseudo_state.md")]
    fn pseudo_state(&self) -> PseudoState {
        let interaction = self
            .pseudo
            .map_or_else(PseudoState::empty, |states| states.get(self.id));
        let mut structural = PseudoState::empty();
        if let Some(node) = self.node() {
            if node.attributes().is_disabled() {
                structural = structural.insert(PseudoState::DISABLED);
            }
            if node.attributes().is_checked() {
                structural = structural.insert(PseudoState::CHECKED);
            }
        }
        interaction.union(structural)
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
            .then(|| self.cursor(parent_id))
    }

    fn previous_sibling_element(&self) -> Option<Self> {
        let siblings = self.element_siblings();
        let pos = siblings.iter().position(|s| *s == self.id)?;
        let prev = pos.checked_sub(1)?;
        siblings.get(prev).map(|prev_id| self.cursor(*prev_id))
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
        self.node().is_some_and(|n| !n.child_nodes().is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_identity::ComponentId;
    use obscur_tree::reconciliation::Reconciler;
    use obscur_view::view::{
        CHECKED, CLASS, DISABLED, ID, LOCAL_NAME, TYPE, View, ViewFragment, ViewNode, ViewProperty,
        ViewValue,
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
            .map(|n| n.child_nodes().to_vec())
            .unwrap_or_default()
    }

    #[test]
    fn reports_local_name_id_and_class() {
        let view = View::from_fragment(ViewFragment::from_nodes(vec![
            ViewNode::element(ComponentId::from_raw(10))
                .with_prop(text_prop(LOCAL_NAME, "container"))
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
    fn attribute_resolves_id_and_the_authored_type_only() {
        // `[type=…]` reads the attribute the author wrote, which is a different
        // reserved symbol from the element's own name. This assertion used to read
        // `Some("button")` off a node that had no `type` at all — the local name
        // answering for an attribute nobody wrote, which is exactly why
        // `[type=submit]` could not be written to mean the attribute.
        let view = View::from_fragment(ViewFragment::from_nodes(vec![
            ViewNode::element(ComponentId::from_raw(10))
                .with_prop(text_prop(LOCAL_NAME, "button"))
                .with_prop(text_prop(TYPE, "submit"))
                .with_prop(text_prop(ID, "ok")),
        ]));
        let tree = reconcile(&view);
        let child = root_children(&tree)[0];
        let element = TreeElement::new(&tree, child);

        assert_eq!(element.attribute("id"), Some("ok"));
        assert_eq!(element.attribute("ID"), Some("ok"));
        assert_eq!(element.attribute("type"), Some("submit"));
        assert_eq!(element.attribute("TYPE"), Some("submit"));
        // The tag is still the type selector's subject, and it is not the attribute.
        assert_eq!(element.local_name(), Some("button"));
        assert_eq!(element.attribute("data-foo"), None);
    }

    #[test]
    fn an_element_without_an_authored_type_has_no_type_attribute() {
        // The other half: the tag alone must not answer `[type=…]`. Every element
        // in the tree has a local name, so a local name standing in for the
        // attribute made `[type=div]` match every box in the document.
        let view = View::from_fragment(ViewFragment::from_nodes(vec![
            ViewNode::element(ComponentId::from_raw(10)).with_prop(text_prop(LOCAL_NAME, "div")),
        ]));
        let tree = reconcile(&view);
        let child = root_children(&tree)[0];
        let element = TreeElement::new(&tree, child);

        assert_eq!(element.local_name(), Some("div"));
        assert_eq!(element.attribute("type"), None);
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
        let kids: Vec<NodeId> = container_node.child_nodes().to_vec();
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
                .child_nodes()
                .iter()
                .all(|c| tree.get(*c).is_some())
        );
        assert!(TreeElement::new(&tree, container_id).has_children());
        assert!(!first.has_children());
    }

    #[test]
    fn with_pseudo_reads_recorded_interaction_state() {
        let view = View::from_fragment(ViewFragment::from_nodes(vec![ViewNode::element(
            ComponentId::from_raw(10),
        )]));
        let tree = reconcile(&view);
        let child = root_children(&tree)[0];

        // Stateless cursor: always empty.
        assert_eq!(
            TreeElement::new(&tree, child).pseudo_state(),
            PseudoState::empty()
        );

        // Pseudo-aware cursor over a map that records HOVER for the node.
        let mut states = crate::orchestration::PseudoStates::new();
        states.set(child, PseudoState::HOVER);
        let hovered = TreeElement::with_pseudo(&tree, child, &states);
        assert_eq!(hovered.pseudo_state(), PseudoState::HOVER);

        // A node with no recorded state still reads empty through the same map.
        let mut empty_states = crate::orchestration::PseudoStates::new();
        empty_states.set(NodeId::from_raw(9999), PseudoState::FOCUS);
        assert_eq!(
            TreeElement::with_pseudo(&tree, child, &empty_states).pseudo_state(),
            PseudoState::empty()
        );
    }

    #[test]
    fn structural_disabled_and_checked_report_on_both_paths() {
        use obscur_style::selector::{
            CompoundSelector, PseudoClassSelector, SelectorMatcher, SimpleSelector,
        };

        // A `button` carrying the reserved DISABLED attribute plus a `checkbox`
        // carrying CHECKED — the structural state the widgets now emit.
        let view = View::from_fragment(ViewFragment::from_nodes(vec![
            ViewNode::element(ComponentId::from_raw(20))
                .with_prop(text_prop(LOCAL_NAME, "button"))
                .with_prop(ViewProperty::new(DISABLED, ViewValue::Bool(true))),
            ViewNode::element(ComponentId::from_raw(21))
                .with_prop(text_prop(LOCAL_NAME, "checkbox"))
                .with_prop(ViewProperty::new(CHECKED, ViewValue::Bool(true))),
        ]));
        let tree = reconcile(&view);
        let kids = root_children(&tree);
        let button = kids[0];
        let checkbox = kids[1];

        // Stateless cursor (built via `new`) still reports the intrinsic bits.
        let btn = TreeElement::new(&tree, button);
        assert!(btn.pseudo_state().contains(PseudoState::DISABLED));
        assert!(!btn.pseudo_state().contains(PseudoState::CHECKED));

        let chk = TreeElement::new(&tree, checkbox);
        assert!(chk.pseudo_state().contains(PseudoState::CHECKED));
        assert!(!chk.pseudo_state().contains(PseudoState::DISABLED));

        // Structural bits also survive the stateful path, unioned with interaction.
        let mut states = crate::orchestration::PseudoStates::new();
        states.set(button, PseudoState::HOVER);
        let hovered_disabled = TreeElement::with_pseudo(&tree, button, &states);
        assert!(
            hovered_disabled
                .pseudo_state()
                .contains(PseudoState::DISABLED)
        );
        assert!(hovered_disabled.pseudo_state().contains(PseudoState::HOVER));

        // The `:disabled` / `:checked` selectors actually match the right node.
        let disabled_sel = CompoundSelector::new(vec![SimpleSelector::PseudoClass(
            PseudoClassSelector::Disabled,
        )]);
        let checked_sel = CompoundSelector::new(vec![SimpleSelector::PseudoClass(
            PseudoClassSelector::Checked,
        )]);
        assert!(SelectorMatcher::matches_compound(&disabled_sel, &btn));
        assert!(!SelectorMatcher::matches_compound(&disabled_sel, &chk));
        assert!(SelectorMatcher::matches_compound(&checked_sel, &chk));
        assert!(!SelectorMatcher::matches_compound(&checked_sel, &btn));
    }

    #[test]
    fn navigation_preserves_the_pseudo_source() {
        // container -> child; record HOVER on the container, navigate up from the
        // child and confirm the source survived the parent() hop.
        let container = ViewNode::element(ComponentId::from_raw(1)).with_children(
            ViewFragment::from_nodes(vec![ViewNode::element(ComponentId::from_raw(11))]),
        );
        let view = View::from_fragment(ViewFragment::from_nodes(vec![container]));
        let tree = reconcile(&view);
        let container_id = root_children(&tree)[0];
        let child_id = tree.get(container_id).unwrap().child_nodes().to_vec()[0];

        let mut states = crate::orchestration::PseudoStates::new();
        states.set(container_id, PseudoState::HOVER);

        let child = TreeElement::with_pseudo(&tree, child_id, &states);
        let parent = child.parent().expect("child has an element parent");
        assert_eq!(parent.node_id(), container_id);
        assert_eq!(parent.pseudo_state(), PseudoState::HOVER);
    }
}
