#![doc = include_str!("semantic_builder.doc/_module.md")]

use std::collections::BTreeSet;

use obscur_accessibility::semantic::{
    SemanticAction, SemanticNode, SemanticRole, SemanticState, SemanticValue,
};
use obscur_accessibility::tree::{SemanticSnapshot, SemanticTree};
use obscur_identity::NodeId;
use obscur_tree::node::{NodeFlags, NodeKind, RetainedNode};
use obscur_tree::tree::RetainedTree;
use obscur_view::view::{EVENT_CHANGE, EVENT_CLICK, EVENT_INPUT};

#[doc = include_str!("semantic_builder.doc/InheritedState.md")]
#[derive(Debug, Clone, Copy, Default)]
struct InheritedState {
    #[doc = include_str!("semantic_builder.doc/disabled.md")]
    disabled: bool,
    #[doc = include_str!("semantic_builder.doc/readonly.md")]
    readonly: bool,
    #[doc = include_str!("semantic_builder.doc/editable.md")]
    editable: bool,
}

impl InheritedState {
    #[doc = include_str!("semantic_builder.doc/extend.md")]
    fn extend(self, node: &RetainedNode) -> Self {
        let attributes = node.attributes();
        Self {
            disabled: self.disabled || attributes.is_disabled(),
            readonly: self.readonly || attributes.is_readonly(),
            editable: self.editable || node.command_for_event(EVENT_INPUT).is_some(),
        }
    }
}

#[doc = include_str!("semantic_builder.doc/SemanticTreeBuilder.md")]
#[derive(Debug, Clone, Copy, Default)]
pub struct SemanticTreeBuilder;

impl SemanticTreeBuilder {
    #[doc = include_str!("semantic_builder.doc/build.md")]
    #[must_use]
    pub fn build(tree: &RetainedTree, focused: Option<NodeId>) -> SemanticTree {
        let mut semantic = SemanticTree::new();
        if let Some(root) = tree.root() {
            let mut visited = BTreeSet::new();
            let inherited = InheritedState::default();
            Self::walk(
                tree,
                root,
                None,
                focused,
                inherited,
                &mut visited,
                &mut semantic,
            );
            if semantic.contains(root) {
                semantic.set_root(root);
            }
        }
        semantic
    }

    #[doc = include_str!("semantic_builder.doc/snapshot.md")]
    #[must_use]
    pub fn snapshot(tree: &RetainedTree, focused: Option<NodeId>) -> SemanticSnapshot {
        SemanticSnapshot::from_tree(&Self::build(tree, focused))
    }

    #[doc = include_str!("semantic_builder.doc/walk.md")]
    fn walk(
        tree: &RetainedTree,
        id: NodeId,
        parent: Option<NodeId>,
        focused: Option<NodeId>,
        inherited: InheritedState,
        visited: &mut BTreeSet<NodeId>,
        semantic: &mut SemanticTree,
    ) {
        if !visited.insert(id) {
            return;
        }
        let Some(node) = tree.get(id) else {
            return;
        };
        // Walking `children()` follows the same links the reconciler stamps as
        // each node's logical parent, so this inheritance sees exactly the chain
        // the runtime's `is_within_disabled` gate walks upwards.
        let inherited = inherited.extend(node);
        semantic.insert(Self::map_node(tree, id, node, focused, inherited), parent);
        for &child in node.child_nodes() {
            Self::walk(tree, child, Some(id), focused, inherited, visited, semantic);
        }
    }

    #[doc = include_str!("semantic_builder.doc/map_node.md")]
    fn map_node(
        tree: &RetainedTree,
        id: NodeId,
        node: &RetainedNode,
        focused: Option<NodeId>,
        inherited: InheritedState,
    ) -> SemanticNode {
        let role = Self::role_of(node);

        let mut state = SemanticState::new();
        state.hidden = node.has_flags(NodeFlags::HIDDEN);
        state.focusable = role.is_interactive();
        state.focused = Some(id) == focused;
        // `focusable` stays role-derived and `disabled` gates it: that split is
        // what `is_actionable` is for.
        state.disabled = inherited.disabled;
        state.readonly = inherited.readonly;
        state.checked = Self::checked_state(role, node);

        let mut semantic = SemanticNode::new(id, role).with_state(state);

        for action in Self::actions_of(node, state, inherited) {
            semantic.add_action(action);
        }

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

    #[doc = include_str!("semantic_builder.doc/checked_state.md")]
    fn checked_state(role: SemanticRole, node: &RetainedNode) -> Option<bool> {
        matches!(role, SemanticRole::Checkbox | SemanticRole::RadioButton)
            .then(|| node.attributes().is_checked())
    }

    #[doc = include_str!("semantic_builder.doc/actions_of.md")]
    fn actions_of(
        node: &RetainedNode,
        state: SemanticState,
        inherited: InheritedState,
    ) -> Vec<SemanticAction> {
        let mut actions = Vec::new();
        if state.disabled {
            return actions;
        }
        if Self::is_activatable(node, inherited) {
            actions.push(SemanticAction::Activate);
        }
        if state.is_actionable() {
            actions.push(SemanticAction::Focus);
        }
        actions
    }

    #[doc = include_str!("semantic_builder.doc/is_activatable.md")]
    fn is_activatable(node: &RetainedNode, inherited: InheritedState) -> bool {
        node.command_for_event(EVENT_CLICK).is_some()
            || (!inherited.editable && node.command_for_event(EVENT_CHANGE).is_some())
    }

    #[doc = include_str!("semantic_builder.doc/role_of.md")]
    pub(crate) fn role_of(node: &RetainedNode) -> SemanticRole {
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

    #[doc = include_str!("semantic_builder.doc/role_for_local_name.md")]
    fn role_for_local_name(local_name: &str) -> SemanticRole {
        match local_name {
            "button" => SemanticRole::Button,
            "link" | "a" => SemanticRole::Link,
            "text" | "span" | "label" | "p" => SemanticRole::Text,
            "heading" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => SemanticRole::Heading,
            "image" | "img" => SemanticRole::Image,
            "checkbox" => SemanticRole::Checkbox,
            "radio" | "radiobutton" => SemanticRole::RadioButton,
            // HTML's two text-entry tags are one control here, and one role: a
            // `<textarea>` is a multi-line text input, not an unnamed group.
            "input" | "textarea" | "textinput" | "textfield" => SemanticRole::TextInput,
            "list" | "ul" | "ol" => SemanticRole::List,
            "listitem" | "li" => SemanticRole::ListItem,
            "table" => SemanticRole::Table,
            "row" | "tr" => SemanticRole::Row,
            // No `ColumnHeader` role exists to map `th` onto, so it takes the
            // nearest true one: a header cell is a cell. Naming it `Group` — the
            // fallback — would say it is not part of the table at all.
            "cell" | "td" | "th" => SemanticRole::Cell,
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

    #[doc = include_str!("semantic_builder.doc/element_name.md")]
    fn element_name(tree: &RetainedTree, node: &RetainedNode) -> Option<String> {
        let parts: Vec<String> = node
            .child_nodes()
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
    use obscur_identity::{ComponentId, InternedSymbol};
    use obscur_view::view::{
        CHECKED, CLASS, DISABLED, LOCAL_NAME, READONLY, StyleAttributes, ViewEventBinding,
        ViewProperty, ViewValue,
    };

    /// Allocates an element whose local name (CSS type) is `local_name`.
    fn element(tree: &mut RetainedTree, component: u64, local_name: &str) -> NodeId {
        element_with(tree, component, local_name, &[])
    }

    /// Allocates an element with local name `local_name` plus the reserved
    /// structural-state attributes in `state` (e.g. `DISABLED`), extracted
    /// exactly as the reconciler extracts them onto a retained node.
    fn element_with(
        tree: &mut RetainedTree,
        component: u64,
        local_name: &str,
        state: &[InternedSymbol],
    ) -> NodeId {
        let id = tree.alloc(NodeKind::Element {
            component: ComponentId::from_raw(component),
        });
        let mut props = vec![ViewProperty::new(
            LOCAL_NAME,
            ViewValue::Text(local_name.into()),
        )];
        props.extend(
            state
                .iter()
                .map(|&name| ViewProperty::new(name, ViewValue::Bool(true))),
        );
        tree.get_mut(id)
            .unwrap()
            .set_attributes(StyleAttributes::extract(&props));
        id
    }

    /// Binds each of `events` to a command on an already-allocated node.
    fn bind(tree: &mut RetainedTree, id: NodeId, events: &[InternedSymbol]) {
        let bindings: Box<[ViewEventBinding]> = events
            .iter()
            .map(|&event| ViewEventBinding::new(event, InternedSymbol::from_raw(0x00BE_EF01)))
            .collect();
        tree.get_mut(id).unwrap().set_event_bindings(bindings);
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

        let button_node = tree.get_mut(button).unwrap();
        link(button_node, ok);
        let panel_node = tree.get_mut(panel).unwrap();
        link(panel_node, button);
        link(panel_node, hello);
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
        tree.get_mut(button)
            .unwrap()
            .insert_flags(NodeFlags::HIDDEN);

        let semantic = SemanticTreeBuilder::build(&tree, Some(button));
        let button_node = semantic.get(button).unwrap();
        assert!(button_node.state().focused, "passed-in focus is reflected");
        assert!(
            button_node.state().hidden,
            "HIDDEN flag maps to state.hidden"
        );
    }

    /// The headline §7-A regression: the a11y tree used to announce a disabled
    /// button as enabled and actionable, because the builder never read the
    /// `:disabled` attribute the CSS matcher already consumed.
    #[test]
    fn disabled_control_is_not_actionable() {
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        let button = element_with(&mut tree, 11, "button", &[DISABLED]);
        bind(&mut tree, button, &[EVENT_CLICK]);
        tree.get_mut(root).unwrap().push_child(button);
        tree.set_root(root);

        let semantic = SemanticTreeBuilder::build(&tree, None);
        let node = semantic.get(button).unwrap();
        assert!(
            node.state().disabled,
            "`:disabled` must reach the a11y tree"
        );
        assert!(
            !node.state().is_actionable(),
            "a disabled button must never be announced as actionable"
        );
        assert!(
            node.actions().is_empty(),
            "a disabled control fires no command, so it advertises no action"
        );
    }

    /// The runtime's `is_within_disabled` gate is inherited, so the semantic tree
    /// must inherit too — otherwise a control inside a disabled panel is inert to
    /// interaction but announced as actionable.
    #[test]
    fn disabled_is_inherited_by_the_whole_subtree() {
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        let panel = element_with(&mut tree, 10, "region", &[DISABLED]);
        let button = element(&mut tree, 11, "button");
        bind(&mut tree, button, &[EVENT_CLICK]);
        tree.get_mut(panel).unwrap().push_child(button);
        tree.get_mut(root).unwrap().push_child(panel);
        tree.set_root(root);

        let semantic = SemanticTreeBuilder::build(&tree, None);
        let node = semantic.get(button).unwrap();
        assert!(
            node.state().disabled,
            "a control inside a disabled panel is disabled"
        );
        assert!(!node.state().is_actionable());
        // The root above the disabled panel is untouched.
        assert!(!semantic.get(root).unwrap().state().disabled);
    }

    #[test]
    fn enabled_control_stays_actionable_and_offers_actions() {
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        let button = element(&mut tree, 11, "button");
        bind(&mut tree, button, &[EVENT_CLICK]);
        tree.get_mut(root).unwrap().push_child(button);
        tree.set_root(root);

        let semantic = SemanticTreeBuilder::build(&tree, None);
        let node = semantic.get(button).unwrap();
        assert!(!node.state().disabled);
        assert!(node.state().is_actionable());
        assert!(node.supports_action(SemanticAction::Activate));
        assert!(node.supports_action(SemanticAction::Focus));
    }

    #[test]
    fn checked_tristate_only_for_checkable_roles() {
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        let checked = element_with(&mut tree, 20, "checkbox", &[CHECKED]);
        let unchecked = element(&mut tree, 21, "checkbox");
        let radio = element_with(&mut tree, 22, "radio", &[CHECKED]);
        // A button carrying a stray `checked` attribute is still not checkable.
        let button = element_with(&mut tree, 23, "button", &[CHECKED]);
        for child in [checked, unchecked, radio, button] {
            tree.get_mut(root).unwrap().push_child(child);
        }
        tree.set_root(root);

        let semantic = SemanticTreeBuilder::build(&tree, None);
        assert_eq!(semantic.get(checked).unwrap().state().checked, Some(true));
        assert_eq!(
            semantic.get(unchecked).unwrap().state().checked,
            Some(false),
            "a checkable role reports a definite two-state"
        );
        assert_eq!(semantic.get(radio).unwrap().state().checked, Some(true));
        assert_eq!(
            semantic.get(button).unwrap().state().checked,
            None,
            "a non-checkable role must not claim a checked state"
        );
    }

    /// `readonly` is independent of `disabled`: a readonly field is still
    /// focusable and actionable, exactly as the runtime's separate
    /// `is_within_readonly` gate treats it.
    #[test]
    fn readonly_reaches_the_tree_without_disabling() {
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        let input = element_with(&mut tree, 30, "input", &[READONLY]);
        tree.get_mut(root).unwrap().push_child(input);
        tree.set_root(root);

        let semantic = SemanticTreeBuilder::build(&tree, None);
        let node = semantic.get(input).unwrap();
        assert!(node.state().readonly);
        assert!(!node.state().disabled);
        assert!(node.state().is_actionable(), "readonly is still operable");
    }

    /// A checkbox activates on `on-change` (its click *is* its change), but a
    /// text field's `change` is a commit — mirroring the runtime's
    /// `activation_command_for`.
    #[test]
    fn change_binding_activates_only_a_non_editable_control() {
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        let checkbox = element(&mut tree, 40, "checkbox");
        bind(&mut tree, checkbox, &[EVENT_CHANGE]);
        let input = element(&mut tree, 41, "input");
        bind(&mut tree, input, &[EVENT_INPUT, EVENT_CHANGE]);
        for child in [checkbox, input] {
            tree.get_mut(root).unwrap().push_child(child);
        }
        tree.set_root(root);

        let semantic = SemanticTreeBuilder::build(&tree, None);
        assert!(
            semantic
                .get(checkbox)
                .unwrap()
                .supports_action(SemanticAction::Activate),
            "a checkbox's click is its change"
        );
        let input_node = semantic.get(input).unwrap();
        assert!(
            !input_node.supports_action(SemanticAction::Activate),
            "an editable field's `change` is a commit, not a click activation"
        );
        assert!(
            input_node.supports_action(SemanticAction::Focus),
            "but it is still focusable"
        );
    }

    #[test]
    fn a_node_without_bindings_advertises_no_activation() {
        let (tree, _root, panel, ..) = sample_tree();
        let semantic = SemanticTreeBuilder::build(&tree, None);
        // The `region` panel binds nothing and is not interactive.
        let node = semantic.get(panel).unwrap();
        assert!(node.actions().is_empty());
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
        let attrs =
            StyleAttributes::extract(&[ViewProperty::new(CLASS, ViewValue::Text("card".into()))]);
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

    #[test]
    fn the_whole_table_vocabulary_is_announced_as_a_table() {
        // The `Table` widget's parts name themselves `table`/`tr`/`th`/`td` now, so
        // this table is what a screen reader is told. `th` is the entry that had to
        // be added: there is no `ColumnHeader` role to map it onto, so it takes the
        // nearest true one rather than falling through to `Group`, which would say
        // a header cell is not part of the table at all.
        assert_eq!(
            SemanticTreeBuilder::role_for_local_name("table"),
            SemanticRole::Table
        );
        assert_eq!(
            SemanticTreeBuilder::role_for_local_name("tr"),
            SemanticRole::Row
        );
        for cell in ["td", "th"] {
            assert_eq!(
                SemanticTreeBuilder::role_for_local_name(cell),
                SemanticRole::Cell,
                "{cell}"
            );
        }
    }

    #[test]
    fn a_textarea_is_announced_as_a_text_input() {
        // HTML's two text-entry tags are one control here and one role. A
        // multi-line field used to be an unnamed `Group`, because only `input` was
        // in the table — which was invisible while `<textarea>` was a spelling only
        // the `Input` widget could produce, and is not now that markup can write it.
        for tag in ["input", "textarea"] {
            assert_eq!(
                SemanticTreeBuilder::role_for_local_name(tag),
                SemanticRole::TextInput,
                "{tag}"
            );
        }
    }
}
