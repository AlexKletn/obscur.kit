#![doc = include_str!("command_state.doc/_module.md")]

#[doc = include_str!("command_state.doc/CommandState.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommandState {
    #[doc = include_str!("command_state.doc/enabled.md")]
    enabled: bool,
    #[doc = include_str!("command_state.doc/checked.md")]
    checked: Option<bool>,
    #[doc = include_str!("command_state.doc/visible.md")]
    visible: bool,
}

impl CommandState {
    #[doc = include_str!("command_state.doc/new.md")]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            enabled: true,
            checked: None,
            visible: true,
        }
    }

    #[doc = include_str!("command_state.doc/enabled_2.md")]
    #[must_use]
    pub const fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    #[doc = include_str!("command_state.doc/with_checked.md")]
    #[must_use]
    pub const fn with_checked(mut self, checked: Option<bool>) -> Self {
        self.checked = checked;
        self
    }

    #[doc = include_str!("command_state.doc/visible_2.md")]
    #[must_use]
    pub const fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    #[doc = include_str!("command_state.doc/is_enabled.md")]
    #[must_use]
    pub const fn is_enabled(&self) -> bool {
        self.enabled
    }

    #[doc = include_str!("command_state.doc/checked_2.md")]
    #[must_use]
    pub const fn checked(&self) -> Option<bool> {
        self.checked
    }

    #[doc = include_str!("command_state.doc/is_visible.md")]
    #[must_use]
    pub const fn is_visible(&self) -> bool {
        self.visible
    }

    #[doc = include_str!("command_state.doc/is_actionable.md")]
    #[must_use]
    pub const fn is_actionable(&self) -> bool {
        self.enabled && self.visible
    }

    #[doc = include_str!("command_state.doc/is_checked.md")]
    #[must_use]
    pub const fn is_checked(&self) -> bool {
        matches!(self.checked, Some(true))
    }
}

impl Default for CommandState {
    #[doc = include_str!("command_state.doc/default.md")]
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
        assert!(
            CommandState::new()
                .enabled(true)
                .visible(true)
                .is_actionable()
        );
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
