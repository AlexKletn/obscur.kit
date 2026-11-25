use super::Sensitivity;

#[doc = include_str!("../../docs/api/replay/event_recording.doc/RedactionPolicy.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RedactionPolicy {
    /// Whether [`Sensitivity::Secret`] inputs are redacted. Default `true`.
    pub redact_secrets: bool,
    /// Whether [`Sensitivity::Clipboard`] inputs are redacted. Default `true`.
    pub redact_clipboard: bool,
}

impl Default for RedactionPolicy {
    /// The safe default: redact both secrets and protected clipboard data.
    fn default() -> Self {
        Self {
            redact_secrets: true,
            redact_clipboard: true,
        }
    }
}

impl RedactionPolicy {
    /// A policy that redacts both secrets and clipboard data (the default).
    #[must_use]
    pub fn redacting() -> Self {
        Self::default()
    }

    #[doc = include_str!("../../docs/api/replay/event_recording.doc/allow_all.md")]
    #[must_use]
    pub const fn allow_all() -> Self {
        Self {
            redact_secrets: false,
            redact_clipboard: false,
        }
    }

    /// Returns `true` when input of the given `sensitivity` must be redacted
    /// under this policy. [`Sensitivity::Normal`] is never redacted.
    #[must_use]
    pub const fn redacts(&self, sensitivity: Sensitivity) -> bool {
        match sensitivity {
            Sensitivity::Normal => false,
            Sensitivity::Secret => self.redact_secrets,
            Sensitivity::Clipboard => self.redact_clipboard,
        }
    }
}
