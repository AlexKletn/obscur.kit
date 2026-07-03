//! How [`PrivacyClass`] tags are enforced across capture and diagnostics
//! (spec §6.17, §7.12).
//!
//! [`PrivacyPolicy`] is the safe-by-default configuration the runtime consults:
//! sensitive data is redacted from diagnostic snapshots and capture-protected
//! data is excluded from screen capture unless a caller explicitly opts out
//! (spec §7.12). It answers two questions — should this class be redacted, and
//! should it be excluded from capture — given a [`PrivacyClass`].

use crate::privacy::PrivacyClass;

/// How privacy classes are enforced across diagnostics and capture (spec §7.12).
///
/// Safe by default: [`new`](Self::new) / [`Default`] redact sensitive data from
/// diagnostic snapshots and honour capture protection. Callers must explicitly
/// disable a guard to turn it off.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PrivacyPolicy {
    /// Redact [`PrivacyClass::Sensitive`] and above from diagnostic snapshots
    /// (spec §7.12).
    redact_diagnostics: bool,
    /// Honour [`PrivacyClass::CaptureProtected`] / [`PrivacyClass::Secret`] in
    /// screen capture and sharing (spec §6.17).
    enforce_capture_protection: bool,
    /// Hide protected windows from the task switcher and previews (spec §6.17).
    hide_from_task_switcher: bool,
}

impl PrivacyPolicy {
    /// Creates the safe-by-default policy: diagnostics are redacted and capture
    /// protection is enforced, but windows are not hidden from the task switcher
    /// (spec §7.12).
    #[must_use]
    pub const fn new() -> Self {
        Self {
            redact_diagnostics: true,
            enforce_capture_protection: true,
            hide_from_task_switcher: false,
        }
    }

    /// Sets whether sensitive data is redacted from diagnostic snapshots
    /// (spec §7.12).
    #[must_use]
    pub const fn redact_diagnostics(mut self, redact: bool) -> Self {
        self.redact_diagnostics = redact;
        self
    }

    /// Sets whether capture protection is honoured in screen capture and sharing
    /// (spec §6.17).
    #[must_use]
    pub const fn enforce_capture_protection(mut self, enforce: bool) -> Self {
        self.enforce_capture_protection = enforce;
        self
    }

    /// Sets whether protected windows are hidden from the task switcher and
    /// previews (spec §6.17).
    #[must_use]
    pub const fn hide_from_task_switcher(mut self, hide: bool) -> Self {
        self.hide_from_task_switcher = hide;
        self
    }

    /// Returns whether data of `class` must be redacted from a diagnostic
    /// snapshot under this policy (spec §7.12).
    #[must_use]
    pub const fn should_redact(&self, class: PrivacyClass) -> bool {
        self.redact_diagnostics && class.redact_in_diagnostics()
    }

    /// Returns whether data of `class` must be excluded from screen capture and
    /// sharing under this policy (spec §6.17).
    #[must_use]
    pub const fn should_exclude_from_capture(&self, class: PrivacyClass) -> bool {
        self.enforce_capture_protection && class.exclude_from_capture()
    }

    /// Returns whether protected windows are hidden from the task switcher and
    /// previews (spec §6.17).
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
