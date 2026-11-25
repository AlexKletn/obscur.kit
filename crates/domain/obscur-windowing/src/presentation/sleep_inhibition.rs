//! A request to inhibit system or display sleep while presenting (spec §6.5).

use super::SleepInhibitionScope;

#[doc = include_str!("../../docs/api/presentation/sleep_inhibition.doc/SleepInhibition.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SleepInhibition {
    /// What the request keeps awake.
    pub scope: SleepInhibitionScope,
    /// An optional human-readable reason surfaced to the OS.
    pub reason: Option<Box<str>>,
}

impl SleepInhibition {
    /// Creates a request to inhibit full system sleep with a reason.
    #[must_use]
    pub fn system(reason: impl Into<Box<str>>) -> Self {
        Self {
            scope: SleepInhibitionScope::System,
            reason: Some(reason.into()),
        }
    }

    /// Creates a request to inhibit display sleep and the screensaver with a
    /// reason.
    #[must_use]
    pub fn display(reason: impl Into<Box<str>>) -> Self {
        Self {
            scope: SleepInhibitionScope::Display,
            reason: Some(reason.into()),
        }
    }

    /// Creates a request for `scope` with no stated reason.
    #[must_use]
    pub const fn reasonless(scope: SleepInhibitionScope) -> Self {
        Self {
            scope,
            reason: None,
        }
    }

    /// Returns what the request keeps awake.
    #[must_use]
    pub const fn scope(&self) -> SleepInhibitionScope {
        self.scope
    }

    /// Returns the human-readable reason, or `None` if unstated.
    #[must_use]
    pub fn reason(&self) -> Option<&str> {
        self.reason.as_deref()
    }

    #[doc = include_str!("../../docs/api/presentation/sleep_inhibition.doc/inhibits_display.md")]
    #[must_use]
    pub const fn inhibits_display(&self) -> bool {
        matches!(
            self.scope,
            SleepInhibitionScope::System | SleepInhibitionScope::Display
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_sets_scope_and_reason() {
        let inhibition = SleepInhibition::system("playing video");
        assert_eq!(inhibition.scope(), SleepInhibitionScope::System);
        assert_eq!(inhibition.reason(), Some("playing video"));
        assert!(inhibition.inhibits_display());
    }

    #[test]
    fn display_sets_scope_and_reason() {
        let inhibition = SleepInhibition::display("slideshow");
        assert_eq!(inhibition.scope(), SleepInhibitionScope::Display);
        assert_eq!(inhibition.reason(), Some("slideshow"));
        assert!(inhibition.inhibits_display());
    }

    #[test]
    fn reasonless_has_no_reason() {
        let inhibition = SleepInhibition::reasonless(SleepInhibitionScope::System);
        assert_eq!(inhibition.scope(), SleepInhibitionScope::System);
        assert_eq!(inhibition.reason(), None);
    }
}
