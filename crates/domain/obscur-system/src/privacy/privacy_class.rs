#![doc = include_str!("privacy_class.doc/_module.md")]

#[doc = include_str!("privacy_class.doc/PrivacyClass.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub enum PrivacyClass {
    #[doc = include_str!("privacy_class.doc/Public.md")]
    #[default]
    Public,
    #[doc = include_str!("privacy_class.doc/Sensitive.md")]
    Sensitive,
    #[doc = include_str!("privacy_class.doc/CaptureProtected.md")]
    CaptureProtected,
    #[doc = include_str!("privacy_class.doc/Secret.md")]
    Secret,
}

impl PrivacyClass {
    #[doc = include_str!("privacy_class.doc/is_public.md")]
    #[must_use]
    pub const fn is_public(&self) -> bool {
        matches!(self, Self::Public)
    }

    #[doc = include_str!("privacy_class.doc/redact_in_diagnostics.md")]
    #[must_use]
    pub const fn redact_in_diagnostics(&self) -> bool {
        !self.is_public()
    }

    #[doc = include_str!("privacy_class.doc/exclude_from_capture.md")]
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
