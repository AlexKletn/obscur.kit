//! The sensitivity classification of a piece of UI or data (spec §6.17, §7.12).
//!
//! [`PrivacyClass`] tags a region, field, or value with how sensitive it is, so
//! the runtime can decide whether to exclude it from screen capture / sharing
//! (§6.17) and whether to redact it from diagnostic snapshots (§7.12). Variants
//! are ordered least → most sensitive, so the derived [`Ord`] ranks sensitivity
//! and the predicates below are monotone in it.

/// How sensitive a piece of UI or data is (spec §6.17, §7.12).
///
/// Ordered least → most sensitive so the derived [`Ord`] ranks sensitivity:
/// `Public < Sensitive < CaptureProtected < Secret`. The predicates
/// [`redact_in_diagnostics`](Self::redact_in_diagnostics) and
/// [`exclude_from_capture`](Self::exclude_from_capture) are monotone in that
/// order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub enum PrivacyClass {
    /// Non-sensitive: safe to capture and to include in diagnostics (§6.17).
    #[default]
    Public,
    /// Personally identifying information: redacted from diagnostic snapshots by
    /// default, but not excluded from capture (spec §7.12).
    Sensitive,
    /// Must be excluded from screenshots, screen recording, and screen sharing,
    /// and redacted from diagnostics (spec §6.17).
    CaptureProtected,
    /// Passwords and secrets: never captured and never placed in diagnostics
    /// (spec §6.17, §7.12).
    Secret,
}

impl PrivacyClass {
    /// Returns whether this class is [`Public`](Self::Public) — non-sensitive
    /// data with no capture or diagnostic restrictions (spec §6.17).
    #[must_use]
    pub const fn is_public(&self) -> bool {
        matches!(self, Self::Public)
    }

    /// Returns whether data of this class must be redacted from diagnostic
    /// snapshots — anything above [`Public`](Self::Public) (spec §7.12).
    #[must_use]
    pub const fn redact_in_diagnostics(&self) -> bool {
        !self.is_public()
    }

    /// Returns whether data of this class must be excluded from screen capture
    /// and sharing — [`CaptureProtected`](Self::CaptureProtected) and
    /// [`Secret`](Self::Secret) (spec §6.17).
    #[must_use]
    pub const fn exclude_from_capture(&self) -> bool {
        matches!(self, Self::CaptureProtected | Self::Secret)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn variants_ordered_least_to_most_sensitive() {
        assert!(PrivacyClass::Public < PrivacyClass::Sensitive);
        assert!(PrivacyClass::Sensitive < PrivacyClass::CaptureProtected);
        assert!(PrivacyClass::CaptureProtected < PrivacyClass::Secret);
    }

    #[test]
    fn default_is_public() {
        assert_eq!(PrivacyClass::default(), PrivacyClass::Public);
        assert!(PrivacyClass::default().is_public());
    }

    #[test]
    fn redact_in_diagnostics_per_variant() {
        assert!(!PrivacyClass::Public.redact_in_diagnostics());
        assert!(PrivacyClass::Sensitive.redact_in_diagnostics());
        assert!(PrivacyClass::CaptureProtected.redact_in_diagnostics());
        assert!(PrivacyClass::Secret.redact_in_diagnostics());
    }

    #[test]
    fn exclude_from_capture_per_variant() {
        assert!(!PrivacyClass::Public.exclude_from_capture());
        assert!(!PrivacyClass::Sensitive.exclude_from_capture());
        assert!(PrivacyClass::CaptureProtected.exclude_from_capture());
        assert!(PrivacyClass::Secret.exclude_from_capture());
    }

    #[test]
    fn is_public_only_for_public() {
        assert!(PrivacyClass::Public.is_public());
        assert!(!PrivacyClass::Sensitive.is_public());
        assert!(!PrivacyClass::CaptureProtected.is_public());
        assert!(!PrivacyClass::Secret.is_public());
    }
}
