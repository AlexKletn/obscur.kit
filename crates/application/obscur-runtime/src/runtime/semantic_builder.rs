//! Derives the semantic accessibility tree from the retained UI tree (§7.1).

use std::collections::BTreeSet;

use obscur_accessibility::semantic::{SemanticNode, SemanticRole, SemanticState, SemanticValue};
use obscur_accessibility::tree::{SemanticSnapshot, SemanticTree};
use obscur_identity::NodeId;
use obscur_tree::node::{NodeFlags, NodeKind, RetainedNode};
use obscur_tree::tree::RetainedTree;

/// Projects a [`RetainedTree`] into a semantic accessibility tree (spec §7.1).
///
/// §7.1 requires accessibility to use "a semantic tree linked to the retained UI
/// tree". This builder is that link: it walks the live retained tree from its
/// root and emits one [`SemanticNode`] per retained node — keyed by the very same
/// [`NodeId`] — deriving each node's [`role`](SemanticRole),
/// accessible name, and [`state`](SemanticState) from what the retained node
/// actually carries. It invents no data the tree does not hold.
///
/// The mapping is deterministic (`same tree → same semantic tree`) and pure: it
/// reads the retained tree and produces an owned [`SemanticTree`] / immutable
/// [`SemanticSnapshot`], leaving the retained tree untouched. The keeps
/// `obscur-accessibility` free of any dependency on `obscur-tree`.
///
/// # Mapping
///
/// - [`NodeKind::Root`] → [`SemanticRole::Group`] (the structural grouping root);
/// - [`NodeKind::Text`] → [`SemanticRole::Text`], with the literal content as the
///   accessible name and [`SemanticValue::Text`] value;
/// - [`NodeKind::Synthetic`] → [`SemanticRole::Group`], flagged
///   [`decorative`](SemanticNode::decorative) (system parts are not exposed as
///   content);
/// - [`NodeKind::Element`] → the role named by its CSS *type* / local name (e.g.
///   `button` → [`SemanticRole::Button`], `text` → [`SemanticRole::Text`]),
///   defaulting to [`SemanticRole::Group`] when the element carries no
///   recognised local name. An element's accessible name is drawn from its direct
///   text children.
///
/// A retained node's [`NodeFlags::HIDDEN`] maps to
/// [`SemanticState::hidden`], and the `focused` node passed to
/// [`build`](Self::build) is marked [`SemanticState::focused`]; every interactive
/// role is marked [`SemanticState::focusable`].
#[derive(Debug, Clone, Copy, Default)]
pub struct SemanticTreeBuilder;

impl SemanticTreeBuilder {
    /// Builds a [`SemanticTree`] from `tree`, marking `focused` (if live) as the
    /// focused node.
    ///
    /// Walks from the retained root, preserving child order and the retained
    /// [`NodeId`] of every node. Returns an empty tree when the retained tree has
    /// no designated root.
    #[must_use]
    pub fn build(tree: &RetainedTree, focused: Option<NodeId>) -> SemanticTree {
        let mut semantic = SemanticTree::new();
        if let Some(root) = tree.root() {
            let mut visited = BTreeSet::new();
            Self::walk(tree, root, None, focused, &mut visited, &mut semantic);
            if semantic.contains(root) {
                semantic.set_root(root);
            }
        }
        semantic
    }

    /// Builds a [`SemanticTree`] and captures it as an immutable
    /// [`SemanticSnapshot`] for hand-off to an
    /// [`AccessibilityBridge`](obscur_accessibility::port::AccessibilityBridge).
    #[must_use]
    pub fn snapshot(tree: &RetainedTree, focused: Option<NodeId>) -> SemanticSnapshot {
        SemanticSnapshot::from_tree(&Self::build(tree, focused))
    }

    /// Recursively inserts `id` and its descendants into `semantic`, linking each
    /// under `parent`. The `visited` set guards against a malformed (cyclic)
    /// retained tree by inserting each node at most once.
    fn walk(
        tree: &RetainedTree,
        id: NodeId,
        parent: Option<NodeId>,
        focused: Option<NodeId>,
        visited: &mut BTreeSet<NodeId>,
        semantic: &mut SemanticTree,
    ) {
        if !visited.insert(id) {
            return;
        }
        let Some(node) = tree.get(id) else {
            return;
        };
        semantic.insert(Self::map_node(tree, id, node, focused), parent);
        for &child in node.children() {
            Self::walk(tree, child, Some(id), focused, visited, semantic);
        }
    }

    /// Maps a single retained `node` (identified by `id`) to its
    /// [`SemanticNode`], deriving role, accessible name, value and state from the
    /// node's kind, attributes and flags.
    fn map_node(
        tree: &RetainedTree,
        id: NodeId,
        node: &RetainedNode,
        focused: Option<NodeId>,
    ) -> SemanticNode {
        let role = Self::role_of(node);

        let mut state = SemanticState::new();
        state.hidden = node.has_flags(NodeFlags::HIDDEN);
        state.focusable = role.is_interactive();
        state.focused = Some(id) == focused;

        let mut semantic = SemanticNode::new(id, role).with_state(state);

        if node.kind().is_synthetic() {
            semantic = semantic.decorative(true);
        }

        if let Some(content) = node.kind().text() {
            // A text node's literal content is both its accessible name and value.
            semantic = semantic
                .with_name(content)
                .with_value(SemanticValue::text(content));
        } else if let Some(name) = Self::element_name(tree, node) {
            semantic = semantic.with_name(name);
        }

        semantic
    }

    /// Chooses a [`SemanticRole`] for a retained node from its kind and, for
    /// elements, its CSS *type* / local name.
    fn role_of(node: &RetainedNode) -> SemanticRole {
        match node.kind() {
            // The structural root and system-generated synthetic parts are both
            // plain groupings with no stronger semantics.
            NodeKind::Root | NodeKind::Synthetic { .. } => SemanticRole::Group,
            NodeKind::Text { .. } => SemanticRole::Text,
            NodeKind::Element { .. } => node
                .attributes()
                .local_name()
                .map_or(SemanticRole::Group, Self::role_for_local_name),
        }
    }

    /// Maps an element's local name (CSS type selector target) onto a
    /// [`SemanticRole`], falling back to [`SemanticRole::Group`] for names with no
    /// established semantics — inventing no role the tree does not name.
    fn role_for_local_name(local_name: &str) -> SemanticRole {
        match local_name {
            "button" => SemanticRole::Button,
            "link" | "a" => SemanticRole::Link,
            "text" | "span" | "label" | "p" => SemanticRole::Text,
            "heading" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => SemanticRole::Heading,
            "image" | "img" => SemanticRole::Image,
            "checkbox" => SemanticRole::Checkbox,
            "radio" | "radiobutton" => SemanticRole::RadioButton,
            "input" | "textinput" | "textfield" => SemanticRole::TextInput,
            "list" | "ul" | "ol" => SemanticRole::List,
            "listitem" | "li" => SemanticRole::ListItem,
            "table" => SemanticRole::Table,
            "row" | "tr" => SemanticRole::Row,
            "cell" | "td" => SemanticRole::Cell,
            "dialog" => SemanticRole::Dialog,
            "menu" => SemanticRole::Menu,
            "menuitem" => SemanticRole::MenuItem,
            "tab" => SemanticRole::Tab,
            "tabpanel" => SemanticRole::TabPanel,
            "slider" => SemanticRole::Slider,
            "progress" | "progressbar" => SemanticRole::ProgressBar,
            "separator" | "hr" => SemanticRole::Separator,
            "toolbar" => SemanticRole::Toolbar,
            "tooltip" => SemanticRole::Tooltip,
            "region" | "section" => SemanticRole::Region,
            _ => SemanticRole::Group,
        }
    }

    /// Derives an element's accessible name from its direct text children, joining
    /// their non-empty trimmed contents with a single space. Returns `None` when
    /// the element has no text children with content.
    fn element_name(tree: &RetainedTree, node: &RetainedNode) -> Option<String> {
        let parts: Vec<String> = node
            .children()
            .iter()
            .filter_map(|&child| tree.get(child).and_then(|c| c.kind().text()).map(str::trim))
            .filter(|text| !text.is_empty())
            .map(str::to_owned)
            .collect();
        if parts.is_empty() {
            None
        } else {
            Some(parts.join(" "))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_identity::ComponentId;
    use obscur_view::view::{StyleAttributes, ViewProperty, ViewValue, CLASS, TYPE};

    /// Allocates an element whose local name (CSS type) is `local_name`.
    fn element(tree: &mut RetainedTree, component: u64, local_name: &str) -> NodeId {
        let id = tree.alloc(NodeKind::Element {
            component: ComponentId::from_raw(component),
        });
        let attrs = StyleAttributes::extract(&[ViewProperty::new(
            TYPE,
            ViewValue::Text(local_name.into()),
        )]);
        tree.get_mut(id).unwrap().set_attributes(attrs);
        id
    }

    fn text(tree: &mut RetainedTree, content: &str) -> NodeId {
        tree.alloc(NodeKind::Text {
            content: content.into(),
        })
    }

    fn link(parent: &mut RetainedNode, child: NodeId) {
        parent.push_child(child);
    }

    /// Builds `root → panel(element) → [ button(element) → "OK"(text),
    /// "hello"(text) ]` and returns the ids.
    fn sample_tree() -> (RetainedTree, NodeId, NodeId, NodeId, NodeId, NodeId) {
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        let panel = element(&mut tree, 10, "region");
        let button = element(&mut tree, 11, "button");
        let ok = text(&mut tree, "OK");
        let hello = text(&mut tree, "hello");

        // button → "OK"
        let button_node = tree.get_mut(button).unwrap();
        link(button_node, ok);
        // panel → [button, hello]
        let panel_node = tree.get_mut(panel).unwrap();
        link(panel_node, button);
        link(panel_node, hello);
        // root → panel
        let root_node = tree.get_mut(root).unwrap();
        link(root_node, panel);

        tree.set_root(root);
        (tree, root, panel, button, ok, hello)
    }

    #[test]
    fn build_mirrors_the_retained_tree_structure() {
        let (tree, root, panel, button, ok, hello) = sample_tree();
        let semantic = SemanticTreeBuilder::build(&tree, None);

        // Every retained node has a semantic node keyed by the same id.
        assert_eq!(semantic.len(), 5);
        assert_eq!(semantic.root(), Some(root));
        for id in [root, panel, button, ok, hello] {
            assert!(semantic.contains(id), "{id:?} missing from semantic tree");
        }
        // Child links are preserved in order.
        assert_eq!(semantic.children(root), &[panel]);
        assert_eq!(semantic.children(panel), &[button, hello]);
        assert_eq!(semantic.children(button), &[ok]);
    }

    #[test]
    fn roles_and_names_correspond_to_the_tree() {
        let (tree, root, panel, button, ok, _hello) = sample_tree();
        let semantic = SemanticTreeBuilder::build(&tree, None);

        // Root → Group.
        assert_eq!(semantic.get(root).unwrap().role(), SemanticRole::Group);
        // Element with local name `region` → Region.
        assert_eq!(semantic.get(panel).unwrap().role(), SemanticRole::Region);
        // Element with local name `button` → Button, named by its text child.
        let button_node = semantic.get(button).unwrap();
        assert_eq!(button_node.role(), SemanticRole::Button);
        assert_eq!(button_node.name(), Some("OK"));
        assert!(button_node.state().focusable, "button is interactive");
        // Text node → static Text role with the content as the accessible name.
        let ok_node = semantic.get(ok).unwrap();
        assert_eq!(ok_node.role(), SemanticRole::Text);
        assert_eq!(ok_node.name(), Some("OK"));
        assert_eq!(ok_node.value().as_text(), Some("OK"));
    }

    #[test]
    fn focus_and_hidden_flags_reflect_the_tree() {
        let (mut tree, _root, _panel, button, _ok, _hello) = sample_tree();
        tree.get_mut(button).unwrap().insert_flags(NodeFlags::HIDDEN);

        let semantic = SemanticTreeBuilder::build(&tree, Some(button));
        let button_node = semantic.get(button).unwrap();
        assert!(button_node.state().focused, "passed-in focus is reflected");
        assert!(button_node.state().hidden, "HIDDEN flag maps to state.hidden");
    }

    #[test]
    fn synthetic_nodes_are_decorative() {
        use obscur_identity::InternedSymbol;
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        let synthetic = tree.alloc(NodeKind::Synthetic {
            tag: InternedSymbol::from_raw(1),
        });
        tree.get_mut(root).unwrap().push_child(synthetic);
        tree.set_root(root);

        let semantic = SemanticTreeBuilder::build(&tree, None);
        assert!(semantic.get(synthetic).unwrap().is_ignored());
    }

    #[test]
    fn element_without_local_name_defaults_to_group() {
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        let id = tree.alloc(NodeKind::Element {
            component: ComponentId::from_raw(7),
        });
        // A CLASS-only attribute set carries no local name.
        let attrs = StyleAttributes::extract(&[ViewProperty::new(
            CLASS,
            ViewValue::Text("card".into()),
        )]);
        tree.get_mut(id).unwrap().set_attributes(attrs);
        tree.get_mut(root).unwrap().push_child(id);
        tree.set_root(root);

        let semantic = SemanticTreeBuilder::build(&tree, None);
        assert_eq!(semantic.get(id).unwrap().role(), SemanticRole::Group);
    }

    #[test]
    fn empty_tree_without_root_produces_empty_semantic_tree() {
        let tree = RetainedTree::new();
        let semantic = SemanticTreeBuilder::build(&tree, None);
        assert!(semantic.is_empty());
        assert_eq!(semantic.root(), None);
    }

    #[test]
    fn build_is_deterministic() {
        let (tree, ..) = sample_tree();
        let a = SemanticTreeBuilder::build(&tree, None);
        let b = SemanticTreeBuilder::build(&tree, None);
        assert_eq!(a, b);
    }

    #[test]
    fn snapshot_captures_the_built_tree() {
        let (tree, root, ..) = sample_tree();
        let snapshot = SemanticTreeBuilder::snapshot(&tree, None);
        assert_eq!(snapshot.len(), 5);
        assert_eq!(snapshot.root(), Some(root));
    }
}
