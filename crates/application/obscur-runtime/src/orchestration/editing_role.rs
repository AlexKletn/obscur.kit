//! The role a text node plays for the active editing session — whether and how
//! the live session buffer overrides its text: [`EditingRole`].

use obscur_identity::NodeId;
use obscur_tree::tree::RetainedTree;

use super::EditingView;

/// The role a text `node` plays for the editing session bound to `field` — which
/// decides *whether* and *how* the live session buffer overrides the node's
/// painted/measured text (CSS `::placeholder` / `:placeholder-shown` model).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum EditingRole {
    /// `node` is not the editing field's text at all: the session never touches it.
    None,
    /// `node` is the field's ordinary editable text (the field itself, or a `Text`
    /// descendant that is *not* under a `.placeholder` overlay): the session
    /// **always** overrides it, even when the session text is empty (an emptied
    /// field renders empty, byte-identical to the historical value path).
    Value,
    #[doc = include_str!("../../docs/api/orchestration/editing_role.doc/Placeholder.md")]
    Placeholder,
}

impl EditingRole {
    /// Whether the editing `view`'s session text overrides a node in this role
    /// this frame: the field's value text always, its `.placeholder` overlay only
    /// while the session buffer is non-empty (the `:placeholder-shown` rule).
    #[must_use]
    pub(super) fn overrides(self, view: &EditingView<'_>) -> bool {
        match self {
            Self::None => false,
            Self::Value => true,
            // The `:placeholder-shown` rule is about the field's *effective*
            // displayed text: while composing, an active IME preedit (spec §5.12) is
            // shown inline even over an empty buffer, so the field is NOT
            // placeholder-shown and the session overrides the overlay — the preedit
            // is spliced into the (empty) display and painted in the field's ink.
            Self::Placeholder => {
                !view.text.is_empty()
                    || view.preedit.is_some_and(|preedit| !preedit.text.is_empty())
            }
        }
    }
}

#[doc = include_str!("../../docs/api/orchestration/editing_role.doc/editing_role.md")]
pub(super) fn editing_role(tree: &RetainedTree, node: NodeId, field: NodeId) -> EditingRole {
    let mut current = node;
    let mut in_placeholder = false;
    // At most `len` hops reach the root from any node in a well-formed tree; the
    // inclusive bound tolerates a self-parent without looping.
    for _ in 0..=tree.len() {
        if tree
            .get(current)
            .and_then(|node| node.kind().synthetic_tag())
            == Some(super::generated_boxes::DECORATION_TAG)
        {
            return EditingRole::None;
        }
        if let Some(n) = tree.get(current)
            && n.attributes().has_class("placeholder")
        {
            in_placeholder = true;
        }
        if current == field {
            return if in_placeholder {
                EditingRole::Placeholder
            } else {
                EditingRole::Value
            };
        }
        match tree.get(current).and_then(|n| n.logical_parent().parent()) {
            Some(parent) => current = parent,
            None => return EditingRole::None,
        }
    }
    EditingRole::None
}

/// The editing session that overrides `node`'s text this frame, if any: the
/// session view when `node` is the field's value text, or its `.placeholder`
/// overlay while the session buffer is non-empty; `None` otherwise. Shared by the
/// compose (scene) and measure/arrange (layout) passes so the box is sized from —
/// and the glyphs painted from — the *same* string.
pub(crate) fn editing_override<'a>(
    tree: &RetainedTree,
    node: NodeId,
    editing: Option<EditingView<'a>>,
) -> Option<EditingView<'a>> {
    let view = editing?;
    editing_role(tree, node, view.node)
        .overrides(&view)
        .then_some(view)
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_identity::ComponentId;
    use obscur_text::selection::TextSelection;
    use obscur_tree::node::{LogicalParent, NodeKind};

    /// Allocates an element node under `parent`, wiring the child link and the
    /// logical parent, and returns its id.
    fn add_child(tree: &mut RetainedTree, parent: NodeId, raw_component: u64) -> NodeId {
        let id = tree.alloc(NodeKind::Element {
            component: ComponentId::from_raw(raw_component),
        });
        if let Some(node) = tree.get_mut(parent) {
            node.push_child(id);
        }
        if let Some(node) = tree.get_mut(id) {
            node.set_logical_parent(LogicalParent::of(parent));
        }
        id
    }

    /// Allocates a text node carrying `content` under `parent`, wiring the child
    /// link and the logical parent, and returns its id.
    fn add_text_child(tree: &mut RetainedTree, parent: NodeId, content: &str) -> NodeId {
        let id = tree.alloc(NodeKind::Text {
            content: content.into(),
        });
        if let Some(node) = tree.get_mut(parent) {
            node.push_child(id);
        }
        if let Some(node) = tree.get_mut(id) {
            node.set_logical_parent(LogicalParent::of(parent));
        }
        id
    }

    /// Sets `node`'s `class` attribute, the hook the `.placeholder` overlay is
    /// recognised by.
    fn set_class(tree: &mut RetainedTree, node: NodeId, class: &str) {
        use obscur_view::view::{CLASS, StyleAttributes, ViewProperty, ViewValue};
        let attrs =
            StyleAttributes::extract(&[ViewProperty::new(CLASS, ViewValue::Text(class.into()))]);
        if let Some(n) = tree.get_mut(node) {
            n.set_attributes(attrs);
        }
    }

    #[test]
    fn placeholder_overlay_classifies_as_placeholder_role_not_value() {
        // The `.placeholder` overlay (CSS `::placeholder`) is the `:placeholder-shown`
        // hint, not the field's editable text: text inside a `.placeholder` subtree
        // classifies as `EditingRole::Placeholder`, while the field's ordinary value
        // text (and the field itself) classify as `EditingRole::Value`. This replaces
        // the old boolean `is_editing_text_node`, which lumped the placeholder in with
        // "not the field's text": under the CSS `:placeholder-shown` model the session
        // DOES override the placeholder slot once it has typed text, so the placeholder
        // needs its OWN role (whose `overrides` gates on the session buffer) rather than
        // a flat false — that flat false was the regression (typed text never shown when
        // the value was empty, because the placeholder was the only text node).
        let mut tree = RetainedTree::new();
        let root = tree.alloc(NodeKind::Root);
        tree.set_root(root);
        let field = add_child(&mut tree, root, 501);
        // The `.placeholder` overlay element + its hint text.
        let overlay = add_child(&mut tree, field, 502);
        set_class(&mut tree, overlay, "placeholder");
        let hint = add_text_child(&mut tree, overlay, "Type here…");
        // A separate, ordinary value text of the same field.
        let value = add_text_child(&mut tree, field, "x");

        assert_eq!(
            editing_role(&tree, hint, field),
            EditingRole::Placeholder,
            "placeholder hint text is the `::placeholder` slot, not editable value text"
        );
        assert_eq!(
            editing_role(&tree, value, field),
            EditingRole::Value,
            "the field's ordinary value text is editable value text"
        );
        assert_eq!(
            editing_role(&tree, field, field),
            EditingRole::Value,
            "the field itself is editable value text"
        );

        // The `overrides` gate encodes the `:placeholder-shown` rule: a Value node is
        // always overridden by the session (even when empty), but the Placeholder slot
        // is overridden only while the session buffer has typed text.
        let empty = EditingView::new(field, "", TextSelection::caret(0));
        let typed = EditingView::new(field, "abc", TextSelection::caret(3));
        assert!(
            EditingRole::Value.overrides(&empty),
            "value text: session always overrides"
        );
        assert!(EditingRole::Value.overrides(&typed));
        assert!(
            !EditingRole::Placeholder.overrides(&empty),
            "empty session: the muted placeholder hint stays (placeholder-shown holds)"
        );
        assert!(
            EditingRole::Placeholder.overrides(&typed),
            "non-empty session: typed text overrides the placeholder slot"
        );
        assert!(!EditingRole::None.overrides(&typed));
    }
}
