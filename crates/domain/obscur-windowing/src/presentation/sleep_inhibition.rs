//! A request to inhibit system or display sleep while presenting (spec §6.5).

/// What a sleep-inhibition request holds off (spec §6.5).
///
/// `SleepInhibitionScope` is a tightly-coupled part of [`SleepInhibition`]: it
/// names what a request keeps awake and has no meaning apart from it, so it is
/// colocated here rather than given its own module.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SleepInhibitionScope {
    /// Inhibit full system sleep, keeping the machine awake.
    System,
    /// Inhibit only display sleep and the screensaver.
    Display,
}

/// A request to inhibit sleep while presenting (spec §6.5).
///
/// The optional reason is a human-readable string surfaced to the OS, so users
/// and diagnostics can see why sleep is being held off.
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

    /// Returns whether this request inhibits display sleep.
    ///
    /// This is `true` for both scopes: inhibiting full system sleep necessarily
    /// keeps the display awake too.
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
