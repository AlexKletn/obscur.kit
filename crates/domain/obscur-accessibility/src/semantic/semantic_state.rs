#![doc = include_str!("semantic_state.doc/_module.md")]

#[doc = include_str!("semantic_state.doc/SemanticState.md")]
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SemanticState {
    #[doc = include_str!("semantic_state.doc/focusable.md")]
    pub focusable: bool,
    #[doc = include_str!("semantic_state.doc/focused.md")]
    pub focused: bool,
    #[doc = include_str!("semantic_state.doc/disabled.md")]
    pub disabled: bool,
    #[doc = include_str!("semantic_state.doc/hidden.md")]
    pub hidden: bool,
    #[doc = include_str!("semantic_state.doc/selected.md")]
    pub selected: bool,
    #[doc = include_str!("semantic_state.doc/busy.md")]
    pub busy: bool,
    #[doc = include_str!("semantic_state.doc/required.md")]
    pub required: bool,
    #[doc = include_str!("semantic_state.doc/invalid.md")]
    pub invalid: bool,
    #[doc = include_str!("semantic_state.doc/readonly.md")]
    pub readonly: bool,
    #[doc = include_str!("semantic_state.doc/checked.md")]
    pub checked: Option<bool>,
    #[doc = include_str!("semantic_state.doc/expanded.md")]
    pub expanded: Option<bool>,
}

impl SemanticState {
    #[doc = include_str!("semantic_state.doc/new.md")]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            focusable: false,
            focused: false,
            disabled: false,
            hidden: false,
            selected: false,
            busy: false,
            required: false,
            invalid: false,
            readonly: false,
            checked: None,
            expanded: None,
        }
    }

    #[doc = include_str!("semantic_state.doc/focusable_2.md")]
    #[must_use]
    pub const fn focusable(mut self, focusable: bool) -> Self {
        self.focusable = focusable;
        self
    }

    #[doc = include_str!("semantic_state.doc/focused_2.md")]
    #[must_use]
    pub const fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    #[doc = include_str!("semantic_state.doc/disabled_2.md")]
    #[must_use]
    pub const fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    #[doc = include_str!("semantic_state.doc/hidden_2.md")]
    #[must_use]
    pub const fn hidden(mut self, hidden: bool) -> Self {
        self.hidden = hidden;
        self
    }

    #[doc = include_str!("semantic_state.doc/selected_2.md")]
    #[must_use]
    pub const fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    #[doc = include_str!("semantic_state.doc/busy_2.md")]
    #[must_use]
    pub const fn busy(mut self, busy: bool) -> Self {
        self.busy = busy;
        self
    }

    #[doc = include_str!("semantic_state.doc/required_2.md")]
    #[must_use]
    pub const fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    #[doc = include_str!("semantic_state.doc/invalid_2.md")]
    #[must_use]
    pub const fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }

    #[doc = include_str!("semantic_state.doc/readonly_2.md")]
    #[must_use]
    pub const fn readonly(mut self, readonly: bool) -> Self {
        self.readonly = readonly;
        self
    }

    #[doc = include_str!("semantic_state.doc/with_checked.md")]
    #[must_use]
    pub const fn with_checked(mut self, checked: Option<bool>) -> Self {
        self.checked = checked;
        self
    }

    #[doc = include_str!("semantic_state.doc/with_expanded.md")]
    #[must_use]
    pub const fn with_expanded(mut self, expanded: Option<bool>) -> Self {
        self.expanded = expanded;
        self
    }

    #[doc = include_str!("semantic_state.doc/is_actionable.md")]
    #[must_use]
    pub const fn is_actionable(&self) -> bool {
        self.focusable && !self.disabled
    }
}

impl Default for SemanticState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_all_false_and_none() {
        let s = SemanticState::default();
        assert!(!s.focusable);
        assert!(!s.focused);
        assert!(!s.disabled);
        assert!(!s.hidden);
        assert!(!s.selected);
        assert!(!s.busy);
        assert!(!s.required);
        assert!(!s.invalid);
        assert!(!s.readonly);
        assert_eq!(s.checked, None);
        assert_eq!(s.expanded, None);
        assert!(!s.is_actionable());
    }

    #[test]
    fn builder_sets_flags() {
        let s = SemanticState::new()
            .focusable(true)
            .focused(true)
            .disabled(true)
            .hidden(true)
            .selected(true)
            .busy(true)
            .required(true)
            .invalid(true)
            .readonly(true)
            .with_checked(Some(true))
            .with_expanded(Some(false));
        assert!(s.focusable);
        assert!(s.focused);
        assert!(s.disabled);
        assert!(s.hidden);
        assert!(s.selected);
        assert!(s.busy);
        assert!(s.required);
        assert!(s.invalid);
        assert!(s.readonly);
        assert_eq!(s.checked, Some(true));
        assert_eq!(s.expanded, Some(false));
    }

    #[test]
    fn is_actionable_requires_focusable_not_disabled() {
        assert!(SemanticState::new().focusable(true).is_actionable());
        assert!(
            !SemanticState::new()
                .focusable(true)
                .disabled(true)
                .is_actionable()
        );
        assert!(!SemanticState::new().is_actionable());
    }

    #[test]
    fn tristate_checked_variants() {
        assert_eq!(SemanticState::new().checked, None);
        assert_eq!(
            SemanticState::new().with_checked(Some(false)).checked,
            Some(false)
        );
        assert_eq!(
            SemanticState::new().with_checked(Some(true)).checked,
            Some(true)
        );
    }
}
