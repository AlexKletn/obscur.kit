#![doc = include_str!("semantic_action.doc/_module.md")]

#[doc = include_str!("semantic_action.doc/SemanticAction.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SemanticAction {
    #[doc = include_str!("semantic_action.doc/Activate.md")]
    #[default]
    Activate,
    #[doc = include_str!("semantic_action.doc/Focus.md")]
    Focus,
    #[doc = include_str!("semantic_action.doc/Increment.md")]
    Increment,
    #[doc = include_str!("semantic_action.doc/Decrement.md")]
    Decrement,
    #[doc = include_str!("semantic_action.doc/Expand.md")]
    Expand,
    #[doc = include_str!("semantic_action.doc/Collapse.md")]
    Collapse,
    #[doc = include_str!("semantic_action.doc/ScrollIntoView.md")]
    ScrollIntoView,
    #[doc = include_str!("semantic_action.doc/Dismiss.md")]
    Dismiss,
    #[doc = include_str!("semantic_action.doc/ShowContextMenu.md")]
    ShowContextMenu,
    #[doc = include_str!("semantic_action.doc/SetValue.md")]
    SetValue,
    #[doc = include_str!("semantic_action.doc/Select.md")]
    Select,
    #[doc = include_str!("semantic_action.doc/Copy.md")]
    Copy,
    #[doc = include_str!("semantic_action.doc/Paste.md")]
    Paste,
    #[doc = include_str!("semantic_action.doc/Cut.md")]
    Cut,
}

impl SemanticAction {
    #[doc = include_str!("semantic_action.doc/is_value_change.md")]
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
