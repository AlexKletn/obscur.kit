//! An action an accessibility node supports or can receive (spec §7.1).

#[doc = include_str!("../../docs/api/semantic/semantic_action.doc/SemanticAction.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SemanticAction {
    /// Activate the node (click or press). The default.
    #[default]
    Activate,
    /// Give the node keyboard focus.
    Focus,
    /// Increase the node's value by one step.
    Increment,
    /// Decrease the node's value by one step.
    Decrement,
    /// Expand the node to reveal its content.
    Expand,
    /// Collapse the node to hide its content.
    Collapse,
    /// Scroll the node into the visible viewport.
    ScrollIntoView,
    /// Dismiss the node (e.g. close a dialog or tooltip).
    Dismiss,
    /// Show the node's context menu.
    ShowContextMenu,
    /// Set the node's value directly.
    SetValue,
    /// Select the node within its container.
    Select,
    /// Moves the caret or changes the directional selection in editable text.
    SetTextSelection,
    /// Copy the node's content to the clipboard.
    Copy,
    /// Paste clipboard content into the node.
    Paste,
    /// Cut the node's content to the clipboard.
    Cut,
}

impl SemanticAction {
    /// Returns whether this action mutates the node's value (spec §7.1):
    /// [`Increment`](Self::Increment), [`Decrement`](Self::Decrement), and
    /// [`SetValue`](Self::SetValue).
    #[must_use]
    pub const fn is_value_change(&self) -> bool {
        matches!(self, Self::Increment | Self::Decrement | Self::SetValue)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_activate() {
        assert_eq!(SemanticAction::default(), SemanticAction::Activate);
    }

    #[test]
    fn value_change_actions() {
        assert!(SemanticAction::Increment.is_value_change());
        assert!(SemanticAction::Decrement.is_value_change());
        assert!(SemanticAction::SetValue.is_value_change());
        assert!(!SemanticAction::Activate.is_value_change());
        assert!(!SemanticAction::Focus.is_value_change());
        assert!(!SemanticAction::Copy.is_value_change());
    }
}
