//! The presentation policy an application owns: fullscreen and its optional
//! controls (spec §6.5).

use crate::display::DisplayId;
use crate::presentation::FullscreenMode;

use super::SafeEscapePolicy;

#[doc = include_str!("../../docs/api/presentation/presentation_mode.doc/PresentationMode.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PresentationMode {
    /// The fullscreen form to present in.
    pub fullscreen: FullscreenMode,
    /// Whether to hide the cursor while presenting.
    pub hide_cursor: bool,
    /// Whether to inhibit system/display sleep while presenting.
    pub inhibit_sleep: bool,
    /// The display to present on, or `None` to use the current display.
    pub target_display: Option<DisplayId>,
    /// Whether to restore the previous window state when presentation ends.
    pub restore_previous_state: bool,
    /// How the user may leave the presentation.
    pub safe_escape: SafeEscapePolicy,
}

impl PresentationMode {
    /// Creates a windowed, non-fullscreen policy: cursor shown, sleep not
    /// inhibited, current display, previous state restored on exit, and escape
    /// allowed.
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

    /// Sets the fullscreen form, returning the updated policy.
    #[must_use]
    pub const fn with_fullscreen(mut self, fullscreen: FullscreenMode) -> Self {
        self.fullscreen = fullscreen;
        self
    }

    /// Sets whether to hide the cursor, returning the updated policy.
    #[must_use]
    pub const fn hide_cursor(mut self, hide_cursor: bool) -> Self {
        self.hide_cursor = hide_cursor;
        self
    }

    /// Sets whether to inhibit sleep, returning the updated policy.
    #[must_use]
    pub const fn inhibit_sleep(mut self, inhibit_sleep: bool) -> Self {
        self.inhibit_sleep = inhibit_sleep;
        self
    }

    /// Sets the target display, returning the updated policy.
    #[must_use]
    pub const fn on_display(mut self, display: DisplayId) -> Self {
        self.target_display = Some(display);
        self
    }

    /// Sets whether to restore the previous window state on exit, returning the
    /// updated policy.
    #[must_use]
    pub const fn restore_previous_state(mut self, restore: bool) -> Self {
        self.restore_previous_state = restore;
        self
    }

    /// Sets the safe-escape policy, returning the updated policy.
    #[must_use]
    pub const fn with_safe_escape(mut self, safe_escape: SafeEscapePolicy) -> Self {
        self.safe_escape = safe_escape;
        self
    }

    /// Returns the fullscreen form to present in.
    #[must_use]
    pub const fn fullscreen(&self) -> FullscreenMode {
        self.fullscreen
    }

    /// Returns whether the cursor is hidden while presenting.
    #[must_use]
    pub const fn hides_cursor(&self) -> bool {
        self.hide_cursor
    }

    /// Returns whether sleep is inhibited while presenting.
    #[must_use]
    pub const fn inhibits_sleep(&self) -> bool {
        self.inhibit_sleep
    }

    /// Returns the target display, or `None` for the current display.
    #[must_use]
    pub const fn target_display(&self) -> Option<DisplayId> {
        self.target_display
    }

    /// Returns whether the previous window state is restored on exit.
    #[must_use]
    pub const fn restores_previous_state(&self) -> bool {
        self.restore_previous_state
    }

    /// Returns how the user may leave the presentation.
    #[must_use]
    pub const fn safe_escape(&self) -> SafeEscapePolicy {
        self.safe_escape
    }
}

impl Default for PresentationMode {
    /// The default policy is the windowed baseline from [`PresentationMode::new`].
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
