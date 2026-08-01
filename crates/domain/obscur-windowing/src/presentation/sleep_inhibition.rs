#![doc = include_str!("sleep_inhibition.doc/_module.md")]

#[doc = include_str!("sleep_inhibition.doc/SleepInhibitionScope.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SleepInhibitionScope {
    #[doc = include_str!("sleep_inhibition.doc/System.md")]
    System,
    #[doc = include_str!("sleep_inhibition.doc/Display.md")]
    Display,
}

#[doc = include_str!("sleep_inhibition.doc/SleepInhibition.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SleepInhibition {
    #[doc = include_str!("sleep_inhibition.doc/scope.md")]
    pub scope: SleepInhibitionScope,
    #[doc = include_str!("sleep_inhibition.doc/reason.md")]
    pub reason: Option<Box<str>>,
}

impl SleepInhibition {
    #[doc = include_str!("sleep_inhibition.doc/system.md")]
    #[must_use]
    pub fn system(reason: impl Into<Box<str>>) -> Self {
        Self {
            scope: SleepInhibitionScope::System,
            reason: Some(reason.into()),
        }
    }

    #[doc = include_str!("sleep_inhibition.doc/display.md")]
    #[must_use]
    pub fn display(reason: impl Into<Box<str>>) -> Self {
        Self {
            scope: SleepInhibitionScope::Display,
            reason: Some(reason.into()),
        }
    }

    #[doc = include_str!("sleep_inhibition.doc/reasonless.md")]
    #[must_use]
    pub const fn reasonless(scope: SleepInhibitionScope) -> Self {
        Self {
            scope,
            reason: None,
        }
    }

    #[doc = include_str!("sleep_inhibition.doc/scope_2.md")]
    #[must_use]
    pub const fn scope(&self) -> SleepInhibitionScope {
        self.scope
    }

    #[doc = include_str!("sleep_inhibition.doc/reason_2.md")]
    #[must_use]
    pub fn reason(&self) -> Option<&str> {
        self.reason.as_deref()
    }

    #[doc = include_str!("sleep_inhibition.doc/inhibits_display.md")]
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
