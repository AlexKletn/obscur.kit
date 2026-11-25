//! Derives the semantic accessibility tree from the retained UI tree (§7.1).

use super::inherited_state::InheritedState;

use std::collections::BTreeSet;

use obscur_accessibility::semantic::{
    SemanticAction, SemanticNode, SemanticRole, SemanticState, SemanticValue,
};
use obscur_accessibility::tree::{SemanticSnapshot, SemanticTree};
use obscur_identity::{InternedSymbol, NodeId};
use obscur_tree::node::{NodeFlags, NodeKind, RetainedNode};
use obscur_tree::tree::RetainedTree;
use obscur_view::view::{
    EVENT_CHANGE, EVENT_CLICK, SLIDER_HIGH, SLIDER_LOW, SLIDER_RANGED, SliderScale, ViewProperty,
    ViewValue,
};

#[doc = include_str!("../../docs/api/runtime/semantic_builder.doc/SemanticTreeBuilder.md")]
#[derive(Debug, Clone, Copy, Default)]
pub struct SemanticTreeBuilder;

impl SemanticTreeBuilder {
    #[doc = include_str!("../../docs/api/runtime/semantic_builder.doc/PROGRESS_VALUE.md")]
    const PROGRESS_VALUE: InternedSymbol = InternedSymbol::from_raw(0x00D1_0702);

    #[doc = include_str!("../../docs/api/runtime/semantic_builder.doc/PROGRESS_TRACKED.md")]
    const PROGRESS_TRACKED: InternedSymbol = InternedSymbol::from_raw(0x00D1_0704);

    #[doc = include_str!("../../docs/api/runtime/semantic_builder.doc/build.md")]
    #[must_use]
    pub fn build(tree: &RetainedTree, focused: Option<NodeId>) -> SemanticTree {
        Self::build_visible(tree, focused, |_| true)
    }

    /// Builds the accessibility tree from visible nodes, promoting visible descendants past hidden ancestors.
    pub(crate) fn build_visible(
        tree: &RetainedTree,
        focused: Option<NodeId>,
        visible: impl Fn(NodeId) -> bool,
    ) -> SemanticTree {
        let mut semantic = SemanticTree::new();
        if let Some(root) = tree.root() {
            let mut visited = BTreeSet::new();
            let inherited = InheritedState::default();
            Self::walk(
                tree,
                root,
                None,
                (focused, &visible),
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

    /// Builds a [`SemanticTree`] and captures it as an immutable
    /// [`SemanticSnapshot`] for hand-off to an
    /// [`AccessibilityBridge`](obscur_accessibility::port::AccessibilityBridge).
    #[must_use]
    pub fn snapshot(tree: &RetainedTree, focused: Option<NodeId>) -> SemanticSnapshot {
        SemanticSnapshot::from_tree(&Self::build(tree, focused))
    }

    /// Recursively inserts `id` and its descendants into `semantic`, linking each
    /// under `parent` and folding `id`'s own structural state into the
    /// `inherited` state its subtree sees. The `visited` set guards against a
    /// malformed (cyclic) retained tree by inserting each node at most once.
    fn walk(
        tree: &RetainedTree,
        id: NodeId,
        parent: Option<NodeId>,
        projection: (Option<NodeId>, &impl Fn(NodeId) -> bool),
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
        let (focused, visible) = projection;
        let child_parent = if visible(id) {
            let mapped = Self::map_node(tree, id, node, focused, inherited, visible);
            semantic.insert(mapped, parent);
            Some(id)
        } else if parent.is_none() {
            // Keep one structural root even when CSS hides the document root.
            semantic.insert(SemanticNode::new(id, SemanticRole::Group), None);
            Some(id)
        } else {
            parent
        };
        for &child in node.child_nodes() {
            Self::walk(
                tree,
                child,
                child_parent,
                projection,
                inherited,
                visited,
                semantic,
            );
        }
    }

    /// Maps a single retained `node` (identified by `id`) to its
    /// [`SemanticNode`], deriving role, accessible name, value, state and actions
    /// from the node's kind, attributes, event bindings and flags, plus the
    /// `inherited` structural state of its ancestors.
    fn map_node(
        tree: &RetainedTree,
        id: NodeId,
        node: &RetainedNode,
        focused: Option<NodeId>,
        inherited: InheritedState,
        visible: &impl Fn(NodeId) -> bool,
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

        if role == SemanticRole::Row {
            let flag = |name| {
                node.props().iter().find_map(|prop| {
                    match (prop.name() == InternedSymbol::named(name), prop.value()) {
                        (true, ViewValue::Bool(value)) => Some(*value),
                        _ => None,
                    }
                })
            };
            state.selected = flag("aria-selected").unwrap_or(false);
            state.expanded = flag("aria-expanded");
        }

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
        } else if let Some(name) = Self::element_name(tree, node, visible) {
            semantic = semantic.with_name(name);
        }

        let value = Self::value_of(role, node);
        if !value.is_none() {
            semantic = semantic.with_value(value);
        }

        semantic
    }

    #[doc = include_str!("../../docs/api/runtime/semantic_builder.doc/value_of.md")]
    fn value_of(role: SemanticRole, node: &RetainedNode) -> SemanticValue {
        match role {
            SemanticRole::Slider => Self::slider_value(node.props()),
            SemanticRole::ProgressBar => Self::progress_value(node.props()),
            _ => SemanticValue::None,
        }
    }

    #[doc = include_str!("../../docs/api/runtime/semantic_builder.doc/slider_value.md")]
    fn slider_value(props: &[ViewProperty]) -> SemanticValue {
        let Some(low) = Self::number_prop(props, SLIDER_LOW) else {
            return SemanticValue::None;
        };
        let scale = SliderScale::read(props);
        if Self::flag_prop(props, SLIDER_RANGED) {
            let high = Self::number_prop(props, SLIDER_HIGH).unwrap_or(low);
            SemanticValue::range(low, high, scale.min, scale.max)
        } else {
            SemanticValue::number(low, scale.min, scale.max)
        }
    }

    #[doc = include_str!("../../docs/api/runtime/semantic_builder.doc/progress_value.md")]
    fn progress_value(props: &[ViewProperty]) -> SemanticValue {
        // An untracked bar carries a value of zero — it has to carry something —
        // and that zero is not a position. Reading it would announce "not
        // started" for a bar that is running and does not know how far.
        if !Self::is_tracked(props) {
            return SemanticValue::None;
        }
        Self::number_prop(props, Self::PROGRESS_VALUE).map_or(SemanticValue::None, |fraction| {
            // The scale the bar is drawn against, so the announced fraction and
            // the painted one are the same number.
            SemanticValue::number(fraction.clamp(0.0, 1.0), 0.0, 1.0)
        })
    }

    #[doc = include_str!("../../docs/api/runtime/semantic_builder.doc/is_tracked.md")]
    fn is_tracked(props: &[ViewProperty]) -> bool {
        // Absent means tracked, the same default the painter takes: only an
        // explicit denial makes a bar indeterminate.
        !props.iter().any(|p| {
            p.name() == Self::PROGRESS_TRACKED && matches!(p.value(), ViewValue::Bool(false))
        })
    }

    #[doc = include_str!("../../docs/api/runtime/semantic_builder.doc/number_prop.md")]
    fn number_prop(props: &[ViewProperty], name: InternedSymbol) -> Option<f32> {
        props
            .iter()
            .find(|p| p.name() == name)
            .and_then(|p| match p.value() {
                #[allow(clippy::cast_possible_truncation)]
                ViewValue::Float(v) => Some(*v as f32),
                #[allow(clippy::cast_precision_loss)]
                ViewValue::Int(v) => Some(*v as f32),
                ViewValue::Text(text) => text.trim().parse::<f32>().ok(),
                _ => None,
            })
            .filter(|v| v.is_finite())
    }

    /// Reads `name` off `props` as a boolean, defaulting to `false` when the
    /// property is absent or is not a boolean.
    fn flag_prop(props: &[ViewProperty], name: InternedSymbol) -> bool {
        props
            .iter()
            .find(|p| p.name() == name)
            .is_some_and(|p| matches!(p.value(), ViewValue::Bool(true)))
    }

    /// The [`checked`](SemanticState::checked) tri-state for `node`: the same
    /// `:checked` attribute the CSS matcher reads, but reported only for a role
    /// that is genuinely checkable — a `None` here means "not checkable", which
    /// would be a lie for a button, so a non-checkable role never claims a
    /// two-state at all.
    fn checked_state(role: SemanticRole, node: &RetainedNode) -> Option<bool> {
        matches!(role, SemanticRole::Checkbox | SemanticRole::RadioButton)
            .then(|| node.attributes().is_checked())
    }

    #[doc = include_str!("../../docs/api/runtime/semantic_builder.doc/actions_of.md")]
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

    /// Whether `node`'s own bindings make it activatable, mirroring the runtime's
    /// `UiRuntime::activation_command_for`: a bound [`EVENT_CLICK`] activates,
    /// and so does a bound [`EVENT_CHANGE`] — but only on a **non-editable**
    /// control, since a checkbox's click *is* its change while a text field's
    /// `change` fires on commit (blur / Enter), not on the click.
    fn is_activatable(node: &RetainedNode, inherited: InheritedState) -> bool {
        node.command_for_event(EVENT_CLICK).is_some()
            || (!inherited.editable && node.command_for_event(EVENT_CHANGE).is_some())
    }

    /// Chooses a [`SemanticRole`] for a retained node from its kind and, for
    /// elements, its CSS *type* / local name.
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
            // HTML's two text-entry tags are one control here, and one role: a
            // `<textarea>` is a multi-line text input, not an unnamed group.
            "input" | "textarea" | "textinput" | "textfield" => SemanticRole::TextInput,
            "list" | "ul" | "ol" => SemanticRole::List,
            "listitem" | "li" => SemanticRole::ListItem,
            "table" => SemanticRole::Table,
            "treegrid" => SemanticRole::TreeGrid,
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

    /// Derives an element's accessible name from its direct text children, joining
    /// their non-empty trimmed contents with a single space. Returns `None` when
    /// the element has no text children with content.
    fn element_name(
        tree: &RetainedTree,
        node: &RetainedNode,
        visible: &impl Fn(NodeId) -> bool,
    ) -> Option<String> {
        let parts: Vec<String> = node
            .child_nodes()
            .iter()
            .filter(|&&child| visible(child))
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
    use obscur_view::view::EVENT_INPUT;
    use obscur_view::view::{
        CHECKED, CLASS, DISABLED, LOCAL_NAME, READONLY, SLIDER_MAX, SLIDER_MIN, StyleAttributes,
        ViewEventBinding, ViewProperty, ViewValue,
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

    /// Allocates an element with local name `local_name` carrying `props`
    /// verbatim, exactly as the reconciler stamps a widget's property list onto
    /// a retained node (`set_props`) alongside the extracted attributes.
    fn element_carrying(
        tree: &mut RetainedTree,
        component: u64,
        local_name: &str,
        props: &[ViewProperty],
    ) -> NodeId {
        let id = tree.alloc(NodeKind::Element {
            component: ComponentId::from_raw(component),
        });
        let mut all = vec![ViewProperty::new(
            LOCAL_NAME,
            ViewValue::Text(local_name.into()),
        )];
        all.extend_from_slice(props);
        let node = tree.get_mut(id).unwrap();
        node.set_attributes(StyleAttributes::extract(&all));
        node.set_props(all.into_boxed_slice());
        id
    }

    /// A single-node tree whose one child is `local_name` carrying `props`.
    fn control_tree(local_name: &str, props: &[ViewProperty]) -> (RetainedTree, NodeId) {
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        let id = element_carrying(&mut tree, 50, local_name, props);
        tree.get_mut(root).unwrap().push_child(id);
        tree.set_root(root);
        (tree, id)
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

    /// The headline defect: a slider was announced as a slider and then said
    /// nothing about where it was — a control the user is told exists and
    /// cannot read.
    #[test]
    fn a_slider_publishes_its_position_and_its_scale() {
        let (tree, slider) = control_tree(
            "slider",
            &[
                ViewProperty::new(SLIDER_MIN, ViewValue::Float(0.0)),
                ViewProperty::new(SLIDER_MAX, ViewValue::Float(10.0)),
                ViewProperty::new(SLIDER_LOW, ViewValue::Float(4.0)),
                ViewProperty::new(SLIDER_HIGH, ViewValue::Float(4.0)),
                ViewProperty::new(SLIDER_RANGED, ViewValue::Bool(false)),
            ],
        );

        let semantic = SemanticTreeBuilder::build(&tree, None);
        let node = semantic.get(slider).unwrap();
        assert_eq!(node.role(), SemanticRole::Slider);
        assert_eq!(
            node.value(),
            &SemanticValue::number(4.0, 0.0, 10.0),
            "a slider at 4 on a 0..10 scale says so, bounds included"
        );
    }

    /// A two-thumb slider has two positions and neither of them is "the value".
    #[test]
    fn a_ranged_slider_reports_both_ends_rather_than_half_of_itself() {
        let (tree, slider) = control_tree(
            "slider",
            &[
                ViewProperty::new(SLIDER_MIN, ViewValue::Float(0.0)),
                ViewProperty::new(SLIDER_MAX, ViewValue::Float(100.0)),
                ViewProperty::new(SLIDER_LOW, ViewValue::Float(20.0)),
                ViewProperty::new(SLIDER_HIGH, ViewValue::Float(80.0)),
                ViewProperty::new(SLIDER_RANGED, ViewValue::Bool(true)),
            ],
        );

        let semantic = SemanticTreeBuilder::build(&tree, None);
        let value = semantic.get(slider).unwrap().value();
        assert_eq!(value, &SemanticValue::range(20.0, 80.0, 0.0, 100.0));
        assert_eq!(
            value.as_number(),
            None,
            "announcing 20 for a 20..80 control would be half the truth"
        );
    }

    /// A bare `<slider/>` has not been given a position, and reporting `0`
    /// would be a claim the tree never made.
    #[test]
    fn a_slider_with_no_stated_position_publishes_no_number() {
        let (tree, slider) = control_tree("slider", &[]);
        let semantic = SemanticTreeBuilder::build(&tree, None);
        assert!(semantic.get(slider).unwrap().value().is_none());
    }

    /// The property list `Progress::new(0.5).view()` stamps on its host node.
    fn progress_props(value: f64, tracked: bool) -> Vec<ViewProperty> {
        vec![
            ViewProperty::new(SemanticTreeBuilder::PROGRESS_VALUE, ViewValue::Float(value)),
            ViewProperty::new(
                SemanticTreeBuilder::PROGRESS_TRACKED,
                ViewValue::Bool(tracked),
            ),
        ]
    }

    #[test]
    fn a_progress_bar_publishes_its_fraction() {
        let (tree, bar) = control_tree("progress", &progress_props(0.5, true));
        let semantic = SemanticTreeBuilder::build(&tree, None);
        let node = semantic.get(bar).unwrap();
        assert_eq!(node.role(), SemanticRole::ProgressBar);
        assert_eq!(node.value(), &SemanticValue::number(0.5, 0.0, 1.0));
    }

    /// A value written in markup arrives as a string, and spells the same fact.
    #[test]
    fn a_progress_bar_reads_a_fraction_written_as_text() {
        let (tree, bar) = control_tree(
            "progressbar",
            &[ViewProperty::new(
                SemanticTreeBuilder::PROGRESS_VALUE,
                ViewValue::Text("0.25".into()),
            )],
        );
        let semantic = SemanticTreeBuilder::build(&tree, None);
        assert_eq!(
            semantic.get(bar).unwrap().value(),
            &SemanticValue::number(0.25, 0.0, 1.0)
        );
    }

    /// "Running, position unknown" is spelled by the ABSENCE of a number —
    /// exactly as ARIA omits `aria-valuenow`.
    ///
    /// The trap this pins: `Progress::indeterminate()` still carries a value,
    /// and that value is `0.0`, because the property list has to carry
    /// something. Reading it would announce "not started" about a bar that is
    /// running — a different and false thing.
    #[test]
    fn an_indeterminate_progress_bar_publishes_no_number() {
        let (tree, bar) = control_tree("progress", &progress_props(0.0, false));
        let semantic = SemanticTreeBuilder::build(&tree, None);
        let value = semantic.get(bar).unwrap().value();
        assert!(value.is_none(), "an untracked bar states no position");
        assert_eq!(value.as_number(), None);

        // And a bar that states nothing at all is equally without a number.
        let (bare, id) = control_tree("progress", &[]);
        assert!(
            SemanticTreeBuilder::build(&bare, None)
                .get(id)
                .unwrap()
                .value()
                .is_none()
        );
    }

    /// A value that is correct only on the first frame is the defect a user
    /// reports as "the screen reader never hears the update". The projection is
    /// rebuilt from the tree on every publish, so it must follow the control.
    #[test]
    fn the_published_value_follows_the_control() {
        let (mut tree, slider) = control_tree(
            "slider",
            &[
                ViewProperty::new(SLIDER_MIN, ViewValue::Float(0.0)),
                ViewProperty::new(SLIDER_MAX, ViewValue::Float(10.0)),
                ViewProperty::new(SLIDER_LOW, ViewValue::Float(4.0)),
            ],
        );
        assert_eq!(
            SemanticTreeBuilder::build(&tree, None)
                .get(slider)
                .unwrap()
                .value()
                .as_number(),
            Some(4.0)
        );

        // The drag moved the thumb; the reconciler restamped the props.
        tree.get_mut(slider).unwrap().set_props(
            vec![
                ViewProperty::new(LOCAL_NAME, ViewValue::Text("slider".into())),
                ViewProperty::new(SLIDER_MIN, ViewValue::Float(0.0)),
                ViewProperty::new(SLIDER_MAX, ViewValue::Float(10.0)),
                ViewProperty::new(SLIDER_LOW, ViewValue::Float(7.0)),
            ]
            .into_boxed_slice(),
        );

        assert_eq!(
            SemanticTreeBuilder::build(&tree, None)
                .get(slider)
                .unwrap()
                .value()
                .as_number(),
            Some(7.0),
            "the next publish carries the new position, not the first frame's"
        );
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
