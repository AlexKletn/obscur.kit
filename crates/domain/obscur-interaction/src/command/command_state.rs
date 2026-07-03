//! The [`CommandState`] dynamic runtime state of a command (spec §6.7).

/// The dynamic runtime state of a command (spec §6.7).
///
/// A [`Command`](crate::command::Command) is a static descriptor; its live state
/// changes as the application runs. `CommandState` carries that mutable part:
/// whether the command is currently `enabled`, whether it is a toggle and if so
/// `checked`, and whether it is `visible`. The default state is a live command:
/// enabled, not checkable, and visible.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommandState {
    /// Whether the command can currently be invoked.
    enabled: bool,
    /// The toggle state: `None` if the command is not checkable, `Some(true)` if
    /// checked, `Some(false)` if an unchecked toggle.
    checked: Option<bool>,
    /// Whether the command is shown in its surfaces.
    visible: bool,
}

impl CommandState {
    /// Creates the default live state: enabled, not checkable, and visible.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            enabled: true,
            checked: None,
            visible: true,
        }
    }

    /// Returns this state with its `enabled` flag set.
    #[must_use]
    pub const fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Returns this state with its `checked` toggle set (`None` = not
    /// checkable).
    #[must_use]
    pub const fn with_checked(mut self, checked: Option<bool>) -> Self {
        self.checked = checked;
        self
    }

    /// Returns this state with its `visible` flag set.
    #[must_use]
    pub const fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    /// Returns whether the command can currently be invoked.
    #[must_use]
    pub const fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Returns the toggle state: `None` if not checkable, else `Some`.
    #[must_use]
    pub const fn checked(&self) -> Option<bool> {
        self.checked
    }

    /// Returns whether the command is shown in its surfaces.
    #[must_use]
    pub const fn is_visible(&self) -> bool {
        self.visible
    }

    /// Returns whether the command is actionable right now: enabled and visible.
    #[must_use]
    pub const fn is_actionable(&self) -> bool {
        self.enabled && self.visible
    }

    /// Returns whether the command is a checked toggle (`checked == Some(true)`).
    #[must_use]
    pub const fn is_checked(&self) -> bool {
        matches!(self.checked, Some(true))
    }
}

impl Default for CommandState {
    /// The default live state: enabled, not checkable, and visible.
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_live_and_uncheckable() {
        let s = CommandState::new();
        assert_eq!(s, CommandState::default());
        assert!(s.is_enabled());
        assert_eq!(s.checked(), None);
        assert!(s.is_visible());
        assert!(s.is_actionable());
        assert!(!s.is_checked());
    }

    #[test]
    fn is_actionable_requires_enabled_and_visible() {
        assert!(CommandState::new().enabled(true).visible(true).is_actionable());
        assert!(!CommandState::new().enabled(false).is_actionable());
        assert!(!CommandState::new().visible(false).is_actionable());
        assert!(
            !CommandState::new()
                .enabled(false)
                .visible(false)
                .is_actionable()
        );
    }

    #[test]
    fn is_checked_only_for_some_true() {
        assert!(!CommandState::new().with_checked(None).is_checked());
        assert!(!CommandState::new().with_checked(Some(false)).is_checked());
        assert!(CommandState::new().with_checked(Some(true)).is_checked());
    }
}
