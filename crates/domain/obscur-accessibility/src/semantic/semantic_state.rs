//! The boolean and tri-state flags of an accessibility node (spec §7.1).

/// The state flags of an accessibility node (spec §7.1 state, selection, focus).
///
/// These flags carry the changeable condition of a node — whether it can take
/// focus and currently holds it, whether it is disabled, programmatically
/// hidden, selected, busy, required, invalid, or read-only — plus two tri-state
/// conditions expressed as [`Option<bool>`]: [`checked`](Self::checked) (where
/// `None` means the node is not checkable) and [`expanded`](Self::expanded)
/// (where `None` means it is not expandable).
///
/// The [`hidden`](Self::hidden) flag means *programmatically* hidden (e.g. an
/// `aria-hidden` node), distinct from the decorative hiding tracked on the
/// owning [`SemanticNode`](super::SemanticNode).
///
/// The many independent boolean fields mirror the flat set of accessibility
/// state flags in spec §7.1 and are deliberately kept as separate fields rather
/// than folded into a bitflag type.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SemanticState {
    /// Whether the node can receive keyboard focus.
    pub focusable: bool,
    /// Whether the node currently holds focus.
    pub focused: bool,
    /// Whether the node is disabled and cannot be operated.
    pub disabled: bool,
    /// Whether the node is programmatically hidden from assistive tech.
    pub hidden: bool,
    /// Whether the node is selected within its container.
    pub selected: bool,
    /// Whether the node is busy updating and not yet stable.
    pub busy: bool,
    /// Whether a value is required before the node is considered complete.
    pub required: bool,
    /// Whether the node's current value fails validation.
    pub invalid: bool,
    /// Whether the node's value can be read but not edited.
    pub readonly: bool,
    /// The checked state, or `None` if the node is not checkable.
    pub checked: Option<bool>,
    /// The expanded state, or `None` if the node is not expandable.
    pub expanded: Option<bool>,
}

impl SemanticState {
    /// Creates the default state: every flag `false` and both tri-states
    /// [`None`] (not checkable, not expandable) (spec §7.1).
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

    /// Returns a copy with the [`focusable`](Self::focusable) flag set.
    #[must_use]
    pub const fn focusable(mut self, focusable: bool) -> Self {
        self.focusable = focusable;
        self
    }

    /// Returns a copy with the [`focused`](Self::focused) flag set.
    #[must_use]
    pub const fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    /// Returns a copy with the [`disabled`](Self::disabled) flag set.
    #[must_use]
    pub const fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Returns a copy with the [`hidden`](Self::hidden) flag set.
    #[must_use]
    pub const fn hidden(mut self, hidden: bool) -> Self {
        self.hidden = hidden;
        self
    }

    /// Returns a copy with the [`selected`](Self::selected) flag set.
    #[must_use]
    pub const fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Returns a copy with the [`busy`](Self::busy) flag set.
    #[must_use]
    pub const fn busy(mut self, busy: bool) -> Self {
        self.busy = busy;
        self
    }

    /// Returns a copy with the [`required`](Self::required) flag set.
    #[must_use]
    pub const fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    /// Returns a copy with the [`invalid`](Self::invalid) flag set.
    #[must_use]
    pub const fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }

    /// Returns a copy with the [`readonly`](Self::readonly) flag set.
    #[must_use]
    pub const fn readonly(mut self, readonly: bool) -> Self {
        self.readonly = readonly;
        self
    }

    /// Returns a copy with the [`checked`](Self::checked) tri-state replaced.
    #[must_use]
    pub const fn with_checked(mut self, checked: Option<bool>) -> Self {
        self.checked = checked;
        self
    }

    /// Returns a copy with the [`expanded`](Self::expanded) tri-state replaced.
    #[must_use]
    pub const fn with_expanded(mut self, expanded: Option<bool>) -> Self {
        self.expanded = expanded;
        self
    }

    /// Returns whether the node can currently be acted on: it is
    /// [`focusable`](Self::focusable) and not [`disabled`](Self::disabled)
    /// (spec §7.1).
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
        assert!(!SemanticState::new()
            .focusable(true)
            .disabled(true)
            .is_actionable());
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
