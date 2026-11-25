use obscur_tree::node::RetainedNode;
use obscur_view::view::EVENT_INPUT;

#[doc = include_str!("../../docs/api/runtime/semantic_builder.doc/InheritedState.md")]
#[derive(Debug, Clone, Copy, Default)]
pub(super) struct InheritedState {
    /// The node or an ancestor carries `:disabled` — the whole subtree is inert.
    pub(super) disabled: bool,
    /// The node or an ancestor carries `readonly` — the subtree's value is
    /// immutable (but still focusable and selectable).
    pub(super) readonly: bool,
    /// The node or an ancestor binds [`EVENT_INPUT`], making it a text field
    /// whose `EVENT_CHANGE` is a *commit*, not a click activation.
    pub(super) editable: bool,
}

impl InheritedState {
    /// Folds `node`'s own structural state into the state it and its subtree
    /// see. These are the very attributes the CSS matcher reads for
    /// `:disabled` / `:checked`, so both paths answer from one source.
    pub(super) fn extend(self, node: &RetainedNode) -> Self {
        let attributes = node.attributes();
        Self {
            disabled: self.disabled || attributes.is_disabled(),
            readonly: self.readonly || attributes.is_readonly(),
            editable: self.editable || node.command_for_event(EVENT_INPUT).is_some(),
        }
    }
}
