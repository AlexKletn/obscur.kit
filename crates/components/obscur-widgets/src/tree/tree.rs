//! `Tree`: a headless hierarchical tree with expansion and single selection
//! (spec §2.6).

use super::TreeNode;

use std::collections::{BTreeSet, HashSet};

use obscur_accessibility::semantic::SemanticRole;
use obscur_identity::{ComponentId, InternedSymbol};
use obscur_primitives::Text;
use obscur_view::view::{View, ViewFragment, ViewNode, ViewProperty, ViewValue};

use super::TreeExpansionState;

#[doc = include_str!("../../docs/api/tree/tree.doc/Tree.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tree {
    nodes: Vec<TreeNode>,
    expanded: BTreeSet<usize>,
    selected: Option<usize>,
}

impl Tree {
    /// Captures expanded branches with stable IDs. Ambiguous or empty IDs reject
    /// the snapshot.
    #[must_use]
    pub fn capture_expansion_state(&self) -> Option<TreeExpansionState> {
        if !self.has_unique_node_ids() {
            return None;
        }
        let mut ids: Vec<_> = self
            .expanded
            .iter()
            .filter_map(|&index| self.nodes.get(index)?.id.clone())
            .collect();
        ids.sort_unstable();
        Some(TreeExpansionState::capture(ids))
    }

    /// Replaces the expansion state of branches with stable IDs. Removed nodes and
    /// IDs that now refer to leaves are ignored. Unkeyed branches retain their
    /// current expansion. An invalid snapshot leaves the tree unchanged.
    pub fn restore_expansion_state(&mut self, state: &TreeExpansionState) -> bool {
        let Some(ids) = state.ids().filter(|_| self.has_unique_node_ids()) else {
            return false;
        };
        let mut expanded_ids = HashSet::with_capacity(ids.len());
        if !ids
            .iter()
            .all(|id| !id.trim().is_empty() && expanded_ids.insert(id.as_ref()))
        {
            return false;
        }
        self.expanded
            .retain(|&index| self.nodes[index].id.is_none());
        self.expanded
            .extend(self.nodes.iter().enumerate().filter_map(|(index, node)| {
                (node.has_children
                    && node
                        .id
                        .as_deref()
                        .is_some_and(|id| expanded_ids.contains(id)))
                .then_some(index)
            }));
        true
    }

    fn has_unique_node_ids(&self) -> bool {
        let mut seen = HashSet::new();
        self.nodes
            .iter()
            .filter_map(|node| node.id.as_deref())
            .all(|id| !id.trim().is_empty() && seen.insert(id))
    }

    #[doc = include_str!("../../docs/api/tree/tree.doc/COMPONENT_ID.md")]
    pub const COMPONENT_ID: ComponentId = ComponentId::from_raw(0x00D0_0009);

    /// Stable component id for the per-node child element.
    const TREE_ITEM_ID: ComponentId = ComponentId::from_raw(0x00D0_0019);

    /// Interned name of the semantic-role property (`role`); shares the
    /// `0x00D1_NNXX` scheme with `NN = 09` (tree): role=`00`, depth=`01`,
    /// expanded=`02`, selected=`03` (on item children).
    const PROP_ROLE: InternedSymbol = InternedSymbol::from_raw(0x00D1_0900);
    /// Interned name of the per-item `depth` property.
    const PROP_DEPTH: InternedSymbol = InternedSymbol::from_raw(0x00D1_0901);
    /// Interned name of the per-item `expanded` state property (parents only).
    const PROP_EXPANDED: InternedSymbol = InternedSymbol::from_raw(0x00D1_0902);
    /// Interned name of the per-item `selected` state property.
    const PROP_SELECTED: InternedSymbol = InternedSymbol::from_raw(0x00D1_0903);

    /// Creates an empty tree with nothing expanded or selected.
    #[must_use]
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            expanded: BTreeSet::new(),
            selected: None,
        }
    }

    /// Appends a single node, returning the updated tree (fluent builder).
    #[must_use]
    pub fn node(mut self, node: TreeNode) -> Self {
        self.nodes.push(node);
        self
    }

    /// Appends every node from `nodes`, returning the updated tree.
    #[must_use]
    pub fn with_nodes<I>(mut self, nodes: I) -> Self
    where
        I: IntoIterator<Item = TreeNode>,
    {
        self.nodes.extend(nodes);
        self
    }

    /// Expands the parent node at `index`, returning `true` on success. Returns
    /// `false` (leaving state untouched) when `index` is out of range or the
    /// node has no children.
    pub fn expand(&mut self, index: usize) -> bool {
        if self.nodes.get(index).is_some_and(TreeNode::has_children) {
            self.expanded.insert(index);
            true
        } else {
            false
        }
    }

    /// Collapses the parent node at `index`, returning `true` when it was
    /// previously expanded; otherwise returns `false`.
    pub fn collapse(&mut self, index: usize) -> bool {
        self.expanded.remove(&index)
    }

    /// Toggles the expansion of the parent node at `index`, returning the new
    /// expanded state. A no-op returning `false` when `index` is out of range or
    /// the node has no children.
    pub fn toggle_expanded(&mut self, index: usize) -> bool {
        if self.expanded.contains(&index) {
            self.expanded.remove(&index);
            false
        } else {
            self.expand(index)
        }
    }

    /// Returns whether the node at `index` is expanded.
    #[must_use]
    pub fn is_expanded(&self, index: usize) -> bool {
        self.expanded.contains(&index)
    }

    /// Selects the node at `index`, returning `true` on success. Returns `false`
    /// (leaving state untouched) when `index` is out of range.
    pub fn select(&mut self, index: usize) -> bool {
        if index < self.nodes.len() {
            self.selected = Some(index);
            true
        } else {
            false
        }
    }

    /// Returns the selected node index, if any.
    #[must_use]
    pub const fn selected_index(&self) -> Option<usize> {
        self.selected
    }

    #[doc = include_str!("../../docs/api/tree/tree.doc/visible_nodes.md")]
    #[must_use]
    pub fn visible_nodes(&self) -> Vec<usize> {
        let mut visible = Vec::new();
        // When set, everything deeper than `hide_above` is hidden.
        let mut hide_above: Option<u16> = None;
        for (index, node) in self.nodes.iter().enumerate() {
            if let Some(threshold) = hide_above {
                if node.depth > threshold {
                    continue;
                }
                hide_above = None;
            }
            visible.push(index);
            if node.has_children && !self.expanded.contains(&index) {
                hide_above = Some(node.depth);
            }
        }
        visible
    }

    /// Returns the node at its logical index, including collapsed descendants.
    #[must_use]
    pub fn node_at(&self, index: usize) -> Option<&TreeNode> {
        self.nodes.get(index)
    }

    /// Returns the number of nodes.
    #[must_use]
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Returns whether the tree has no nodes.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Returns the accessible role of this control ([`SemanticRole::List`], the
    /// closest role — see the type docs).
    #[must_use]
    pub const fn semantic_role(&self) -> SemanticRole {
        SemanticRole::List
    }

    #[doc = include_str!("../../docs/api/tree/tree.doc/view.md")]
    #[must_use]
    pub fn view(&self) -> View {
        let children: Vec<ViewNode> = self
            .visible_nodes()
            .into_iter()
            .map(|index| {
                let node = &self.nodes[index];
                let mut element = ViewNode::element(Self::TREE_ITEM_ID)
                    .with_prop(ViewProperty::new(
                        Self::PROP_ROLE,
                        ViewValue::from("treeitem"),
                    ))
                    .with_prop(ViewProperty::new(
                        Self::PROP_DEPTH,
                        ViewValue::Int(i64::from(node.depth)),
                    ));
                if node.has_children {
                    element = element.with_prop(ViewProperty::new(
                        Self::PROP_EXPANDED,
                        ViewValue::Bool(self.is_expanded(index)),
                    ));
                }
                element
                    .with_prop(ViewProperty::new(
                        Self::PROP_SELECTED,
                        ViewValue::Bool(self.selected == Some(index)),
                    ))
                    .with_children(ViewFragment::from_nodes(vec![
                        Text::new(node.label.clone()).into_view_node(),
                    ]))
            })
            .collect();
        let node = ViewNode::element(Self::COMPONENT_ID)
            .with_prop(ViewProperty::new(Self::PROP_ROLE, ViewValue::from("tree")))
            .with_children(ViewFragment::from_nodes(children));
        View::single(node)
    }
}

impl Default for Tree {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Tree {
        Tree::new().with_nodes([
            TreeNode::new("root", 0, true),
            TreeNode::new("child", 1, false),
            TreeNode::new("sibling", 0, false),
        ])
    }

    #[test]
    fn node_accessors() {
        let n = TreeNode::new("n", 2, true);
        assert_eq!(n.label(), "n");
        assert_eq!(n.depth(), 2);
        assert!(n.has_children());
    }

    #[test]
    fn expand_only_parents() {
        let mut t = sample();
        assert!(t.expand(0)); // parent
        assert!(t.is_expanded(0));
        assert!(!t.expand(1)); // leaf
        assert!(!t.expand(9)); // out of range
    }

    #[test]
    fn collapse_hides_subtree() {
        let mut t = sample();
        t.expand(0);
        assert_eq!(t.visible_nodes(), vec![0, 1, 2]);
        assert!(t.collapse(0));
        // child (index 1) is hidden; root and sibling remain.
        assert_eq!(t.visible_nodes(), vec![0, 2]);
    }

    #[test]
    fn collapsed_by_default_hides_child() {
        let t = sample();
        assert_eq!(t.visible_nodes(), vec![0, 2]);
    }

    #[test]
    fn toggle_expanded_returns_new_state() {
        let mut t = sample();
        assert!(t.toggle_expanded(0)); // now expanded
        assert!(t.is_expanded(0));
        assert!(!t.toggle_expanded(0)); // now collapsed
        assert!(!t.is_expanded(0));
    }

    #[test]
    fn select_node() {
        let mut t = sample();
        assert!(t.select(1));
        assert_eq!(t.selected_index(), Some(1));
        assert!(!t.select(9));
    }

    #[test]
    fn semantic_role_is_list() {
        assert_eq!(sample().semantic_role(), SemanticRole::List);
    }

    #[test]
    fn view_visible_child_count_reflects_expansion() {
        let mut t = sample();
        // collapsed: root + sibling only
        match &t.view().root().nodes()[0] {
            ViewNode::Element { children, .. } => assert_eq!(children.len(), 2),
            ViewNode::Text { .. } => panic!("expected element root"),
        }
        t.expand(0);
        t.select(1);
        let view = t.view();
        match &view.root().nodes()[0] {
            ViewNode::Element {
                component,
                children,
                ..
            } => {
                assert_eq!(*component, Tree::COMPONENT_ID);
                assert_eq!(children.len(), 3);
                // The child node (visible index 1) reflects selection + depth.
                match &children.nodes()[1] {
                    ViewNode::Element { props, .. } => {
                        let depth = props
                            .iter()
                            .find(|p| p.name() == Tree::PROP_DEPTH)
                            .expect("depth prop present");
                        assert_eq!(*depth.value(), ViewValue::Int(1));
                        let sel = props
                            .iter()
                            .find(|p| p.name() == Tree::PROP_SELECTED)
                            .expect("selected prop present");
                        assert_eq!(*sel.value(), ViewValue::Bool(true));
                    }
                    ViewNode::Text { .. } => panic!("expected element child"),
                }
            }
            ViewNode::Text { .. } => panic!("expected element root"),
        }
    }
}
