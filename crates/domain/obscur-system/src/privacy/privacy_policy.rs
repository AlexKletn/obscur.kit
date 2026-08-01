#![doc = include_str!("privacy_policy.doc/_module.md")]

use crate::privacy::PrivacyClass;

#[doc = include_str!("privacy_policy.doc/PrivacyPolicy.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PrivacyPolicy {
    #[doc = include_str!("privacy_policy.doc/redact_diagnostics.md")]
    redact_diagnostics: bool,
    #[doc = include_str!("privacy_policy.doc/enforce_capture_protection.md")]
    enforce_capture_protection: bool,
    #[doc = include_str!("privacy_policy.doc/hide_from_task_switcher.md")]
    hide_from_task_switcher: bool,
}

impl PrivacyPolicy {
    #[doc = include_str!("privacy_policy.doc/new.md")]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            redact_diagnostics: true,
            enforce_capture_protection: true,
            hide_from_task_switcher: false,
        }
    }

    #[doc = include_str!("privacy_policy.doc/redact_diagnostics_2.md")]
    #[must_use]
    pub const fn redact_diagnostics(mut self, redact: bool) -> Self {
        self.redact_diagnostics = redact;
        self
    }

    #[doc = include_str!("privacy_policy.doc/enforce_capture_protection_2.md")]
    #[must_use]
    pub const fn enforce_capture_protection(mut self, enforce: bool) -> Self {
        self.enforce_capture_protection = enforce;
        self
    }

    #[doc = include_str!("privacy_policy.doc/hide_from_task_switcher_2.md")]
    #[must_use]
    pub const fn hide_from_task_switcher(mut self, hide: bool) -> Self {
        self.hide_from_task_switcher = hide;
        self
    }

    #[doc = include_str!("privacy_policy.doc/should_redact.md")]
    #[must_use]
    pub const fn should_redact(&self, class: PrivacyClass) -> bool {
        self.redact_diagnostics && class.redact_in_diagnostics()
    }

    #[doc = include_str!("privacy_policy.doc/should_exclude_from_capture.md")]
    #[must_use]
    pub const fn should_exclude_from_capture(&self, class: PrivacyClass) -> bool {
        self.enforce_capture_protection && class.exclude_from_capture()
    }

    #[doc = include_str!("privacy_policy.doc/hides_from_task_switcher.md")]
    #[must_use]
    pub const fn hides_from_task_switcher(&self) -> bool {
        self.hide_from_task_switcher
    }
}

impl Default for PrivacyPolicy {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_safe() {
        let policy = PrivacyPolicy::default();
        assert!(policy.should_redact(PrivacyClass::Sensitive));
        assert!(policy.should_exclude_from_capture(PrivacyClass::CaptureProtected));
        assert!(!policy.hides_from_task_switcher());
    }

    #[test]
    fn should_redact_across_classes() {
        let policy = PrivacyPolicy::new();
        assert!(!policy.should_redact(PrivacyClass::Public));
        assert!(policy.should_redact(PrivacyClass::Sensitive));
        assert!(policy.should_redact(PrivacyClass::CaptureProtected));
        assert!(policy.should_redact(PrivacyClass::Secret));
    }

    #[test]
    fn should_exclude_across_classes() {
        let policy = PrivacyPolicy::new();
        assert!(!policy.should_exclude_from_capture(PrivacyClass::Public));
        assert!(!policy.should_exclude_from_capture(PrivacyClass::Sensitive));
        assert!(policy.should_exclude_from_capture(PrivacyClass::CaptureProtected));
        assert!(policy.should_exclude_from_capture(PrivacyClass::Secret));
    }

    #[test]
    fn redaction_can_be_disabled() {
        let policy = PrivacyPolicy::new().redact_diagnostics(false);
        assert!(!policy.should_redact(PrivacyClass::Secret));
        // Capture protection is unaffected by the diagnostics toggle.
        assert!(policy.should_exclude_from_capture(PrivacyClass::Secret));
    }

    #[test]
    fn capture_protection_can_be_disabled() {
        let policy = PrivacyPolicy::new().enforce_capture_protection(false);
        assert!(!policy.should_exclude_from_capture(PrivacyClass::Secret));
        // Redaction is unaffected by the capture toggle.
        assert!(policy.should_redact(PrivacyClass::Secret));
    }

    #[test]
    fn task_switcher_toggle() {
        let policy = PrivacyPolicy::new().hide_from_task_switcher(true);
        assert!(policy.hides_from_task_switcher());
    }
}
