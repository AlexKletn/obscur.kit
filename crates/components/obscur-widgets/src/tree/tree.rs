#![doc = include_str!("tree.doc/_module.md")]

use std::collections::BTreeSet;

use obscur_accessibility::semantic::SemanticRole;
use obscur_identity::{ComponentId, InternedSymbol};
use obscur_primitives::Text;
use obscur_view::view::{View, ViewFragment, ViewNode, ViewProperty, ViewValue};

#[doc = include_str!("tree.doc/TreeNode.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeNode {
    label: Box<str>,
    depth: u16,
    has_children: bool,
}

impl TreeNode {
    #[doc = include_str!("tree.doc/new.md")]
    #[must_use]
    pub fn new(label: impl Into<Box<str>>, depth: u16, has_children: bool) -> Self {
        Self {
            label: label.into(),
            depth,
            has_children,
        }
    }

    #[doc = include_str!("tree.doc/label.md")]
    #[must_use]
    pub fn label(&self) -> &str {
        &self.label
    }

    #[doc = include_str!("tree.doc/depth.md")]
    #[must_use]
    pub const fn depth(&self) -> u16 {
        self.depth
    }

    #[doc = include_str!("tree.doc/has_children.md")]
    #[must_use]
    pub const fn has_children(&self) -> bool {
        self.has_children
    }
}

#[doc = include_str!("tree.doc/Tree.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tree {
    nodes: Vec<TreeNode>,
    expanded: BTreeSet<usize>,
    selected: Option<usize>,
}

impl Tree {
    #[doc = include_str!("tree.doc/COMPONENT_ID.md")]
    pub const COMPONENT_ID: ComponentId = ComponentId::from_raw(0x00D0_0009);

    #[doc = include_str!("tree.doc/TREE_ITEM_ID.md")]
    const TREE_ITEM_ID: ComponentId = ComponentId::from_raw(0x00D0_0019);

    #[doc = include_str!("tree.doc/PROP_ROLE.md")]
    const PROP_ROLE: InternedSymbol = InternedSymbol::from_raw(0x00D1_0900);
    #[doc = include_str!("tree.doc/PROP_DEPTH.md")]
    const PROP_DEPTH: InternedSymbol = InternedSymbol::from_raw(0x00D1_0901);
    #[doc = include_str!("tree.doc/PROP_EXPANDED.md")]
    const PROP_EXPANDED: InternedSymbol = InternedSymbol::from_raw(0x00D1_0902);
    #[doc = include_str!("tree.doc/PROP_SELECTED.md")]
    const PROP_SELECTED: InternedSymbol = InternedSymbol::from_raw(0x00D1_0903);

    #[doc = include_str!("tree.doc/new_2.md")]
    #[must_use]
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            expanded: BTreeSet::new(),
            selected: None,
        }
    }

    #[doc = include_str!("tree.doc/node.md")]
    #[must_use]
    pub fn node(mut self, node: TreeNode) -> Self {
        self.nodes.push(node);
        self
    }

    #[doc = include_str!("tree.doc/with_nodes.md")]
    #[must_use]
    pub fn with_nodes<I>(mut self, nodes: I) -> Self
    where
        I: IntoIterator<Item = TreeNode>,
    {
        self.nodes.extend(nodes);
        self
    }

    #[doc = include_str!("tree.doc/expand.md")]
    pub fn expand(&mut self, index: usize) -> bool {
        if self.nodes.get(index).is_some_and(TreeNode::has_children) {
            self.expanded.insert(index);
            true
        } else {
            false
        }
    }

    #[doc = include_str!("tree.doc/collapse.md")]
    pub fn collapse(&mut self, index: usize) -> bool {
        self.expanded.remove(&index)
    }

    #[doc = include_str!("tree.doc/toggle_expanded.md")]
    pub fn toggle_expanded(&mut self, index: usize) -> bool {
        if self.expanded.contains(&index) {
            self.expanded.remove(&index);
            false
        } else {
            self.expand(index)
        }
    }

    #[doc = include_str!("tree.doc/is_expanded.md")]
    #[must_use]
    pub fn is_expanded(&self, index: usize) -> bool {
        self.expanded.contains(&index)
    }

    #[doc = include_str!("tree.doc/select.md")]
    pub fn select(&mut self, index: usize) -> bool {
        if index < self.nodes.len() {
            self.selected = Some(index);
            true
        } else {
            false
        }
    }

    #[doc = include_str!("tree.doc/selected_index.md")]
    #[must_use]
    pub const fn selected_index(&self) -> Option<usize> {
        self.selected
    }

    #[doc = include_str!("tree.doc/visible_nodes.md")]
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

    #[doc = include_str!("tree.doc/len.md")]
    #[must_use]
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    #[doc = include_str!("tree.doc/is_empty.md")]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    #[doc = include_str!("tree.doc/semantic_role.md")]
    #[must_use]
    pub const fn semantic_role(&self) -> SemanticRole {
        SemanticRole::List
    }

    #[doc = include_str!("tree.doc/view.md")]
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
