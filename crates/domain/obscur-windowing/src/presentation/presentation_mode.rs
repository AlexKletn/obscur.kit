#![doc = include_str!("presentation_mode.doc/_module.md")]

use crate::display::DisplayId;
use crate::presentation::FullscreenMode;

#[doc = include_str!("presentation_mode.doc/SafeEscapePolicy.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SafeEscapePolicy {
    #[doc = include_str!("presentation_mode.doc/Allow.md")]
    Allow,
    #[doc = include_str!("presentation_mode.doc/RequireHold.md")]
    RequireHold,
    #[doc = include_str!("presentation_mode.doc/Disabled.md")]
    Disabled,
}

impl Default for SafeEscapePolicy {
    #[doc = include_str!("presentation_mode.doc/default.md")]
    fn default() -> Self {
        Self::Allow
    }
}

#[doc = include_str!("presentation_mode.doc/PresentationMode.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PresentationMode {
    #[doc = include_str!("presentation_mode.doc/fullscreen.md")]
    pub fullscreen: FullscreenMode,
    #[doc = include_str!("presentation_mode.doc/hide_cursor.md")]
    pub hide_cursor: bool,
    #[doc = include_str!("presentation_mode.doc/inhibit_sleep.md")]
    pub inhibit_sleep: bool,
    #[doc = include_str!("presentation_mode.doc/target_display.md")]
    pub target_display: Option<DisplayId>,
    #[doc = include_str!("presentation_mode.doc/restore_previous_state.md")]
    pub restore_previous_state: bool,
    #[doc = include_str!("presentation_mode.doc/safe_escape.md")]
    pub safe_escape: SafeEscapePolicy,
}

impl PresentationMode {
    #[doc = include_str!("presentation_mode.doc/new.md")]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            fullscreen: FullscreenMode::Normal,
            hide_cursor: false,
            inhibit_sleep: false,
            target_display: None,
            restore_previous_state: true,
            safe_escape: SafeEscapePolicy::Allow,
        }
    }

    #[doc = include_str!("presentation_mode.doc/with_fullscreen.md")]
    #[must_use]
    pub const fn with_fullscreen(mut self, fullscreen: FullscreenMode) -> Self {
        self.fullscreen = fullscreen;
        self
    }

    #[doc = include_str!("presentation_mode.doc/hide_cursor_2.md")]
    #[must_use]
    pub const fn hide_cursor(mut self, hide_cursor: bool) -> Self {
        self.hide_cursor = hide_cursor;
        self
    }

    #[doc = include_str!("presentation_mode.doc/inhibit_sleep_2.md")]
    #[must_use]
    pub const fn inhibit_sleep(mut self, inhibit_sleep: bool) -> Self {
        self.inhibit_sleep = inhibit_sleep;
        self
    }

    #[doc = include_str!("presentation_mode.doc/on_display.md")]
    #[must_use]
    pub const fn on_display(mut self, display: DisplayId) -> Self {
        self.target_display = Some(display);
        self
    }

    #[doc = include_str!("presentation_mode.doc/restore_previous_state_2.md")]
    #[must_use]
    pub const fn restore_previous_state(mut self, restore: bool) -> Self {
        self.restore_previous_state = restore;
        self
    }

    #[doc = include_str!("presentation_mode.doc/with_safe_escape.md")]
    #[must_use]
    pub const fn with_safe_escape(mut self, safe_escape: SafeEscapePolicy) -> Self {
        self.safe_escape = safe_escape;
        self
    }

    #[doc = include_str!("presentation_mode.doc/fullscreen_2.md")]
    #[must_use]
    pub const fn fullscreen(&self) -> FullscreenMode {
        self.fullscreen
    }

    #[doc = include_str!("presentation_mode.doc/hides_cursor.md")]
    #[must_use]
    pub const fn hides_cursor(&self) -> bool {
        self.hide_cursor
    }

    #[doc = include_str!("presentation_mode.doc/inhibits_sleep.md")]
    #[must_use]
    pub const fn inhibits_sleep(&self) -> bool {
        self.inhibit_sleep
    }

    #[doc = include_str!("presentation_mode.doc/target_display_2.md")]
    #[must_use]
    pub const fn target_display(&self) -> Option<DisplayId> {
        self.target_display
    }

    #[doc = include_str!("presentation_mode.doc/restores_previous_state.md")]
    #[must_use]
    pub const fn restores_previous_state(&self) -> bool {
        self.restore_previous_state
    }

    #[doc = include_str!("presentation_mode.doc/safe_escape_2.md")]
    #[must_use]
    pub const fn safe_escape(&self) -> SafeEscapePolicy {
        self.safe_escape
    }
}

impl Default for PresentationMode {
    #[doc = include_str!("presentation_mode.doc/default_2.md")]
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_windowed_baseline() {
        let mode = PresentationMode::default();
        assert_eq!(mode.fullscreen(), FullscreenMode::Normal);
        assert!(!mode.hides_cursor());
        assert!(!mode.inhibits_sleep());
        assert_eq!(mode.target_display(), None);
        assert!(mode.restores_previous_state());
        assert_eq!(mode.safe_escape(), SafeEscapePolicy::Allow);
        assert_eq!(SafeEscapePolicy::default(), SafeEscapePolicy::Allow);
    }

    #[test]
    fn builder_sets_all_fields() {
        let display = DisplayId::from_raw(3);
        let mode = PresentationMode::new()
            .with_fullscreen(FullscreenMode::Kiosk)
            .hide_cursor(true)
            .inhibit_sleep(true)
            .on_display(display)
            .restore_previous_state(false)
            .with_safe_escape(SafeEscapePolicy::RequireHold);

        assert_eq!(mode.fullscreen(), FullscreenMode::Kiosk);
        assert!(mode.hides_cursor());
        assert!(mode.inhibits_sleep());
        assert_eq!(mode.target_display(), Some(display));
        assert!(!mode.restores_previous_state());
        assert_eq!(mode.safe_escape(), SafeEscapePolicy::RequireHold);
    }
}
