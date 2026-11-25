//! The bundle of input hints describing how a text field expects to be edited
//! (spec §5.12 IME).

use super::InputPurpose;

use super::Capitalization;

#[doc = include_str!("../../docs/api/ime/input_hints.doc/InputHints.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputHints {
    /// The semantic purpose of the field.
    purpose: InputPurpose,
    /// Whether the field accepts multiple lines.
    multiline: bool,
    /// Whether automatic spelling correction is enabled.
    autocorrect: bool,
    /// The automatic-capitalization policy.
    capitalization: Capitalization,
    /// Whether spellchecking is enabled.
    spellcheck: bool,
    /// Whether the field is treated as sensitive (secrets), suppressing IME
    /// learning and suggestions.
    sensitive: bool,
    /// An optional BCP-47 language tag the IME should prefer.
    preferred_language: Option<Box<str>>,
}

impl InputHints {
    /// Creates the default hints: a free-form single-line field with autocorrect
    /// and spellcheck on, [`Sentences`](Capitalization::Sentences)
    /// capitalization, not sensitive, and no preferred language (spec §5.12).
    #[must_use]
    pub const fn new() -> Self {
        Self {
            purpose: InputPurpose::FreeForm,
            multiline: false,
            autocorrect: true,
            capitalization: Capitalization::Sentences,
            spellcheck: true,
            sensitive: false,
            preferred_language: None,
        }
    }

    /// Creates hints suited to a password field: [`Password`](InputPurpose::Password)
    /// purpose, sensitive, with autocorrect, spellcheck, and automatic
    /// capitalization all disabled (spec §5.12 sensitive/password modes are
    /// respected).
    #[must_use]
    pub const fn for_password() -> Self {
        Self {
            purpose: InputPurpose::Password,
            multiline: false,
            autocorrect: false,
            capitalization: Capitalization::None,
            spellcheck: false,
            sensitive: true,
            preferred_language: None,
        }
    }

    /// Returns a copy with the [`purpose`](Self::purpose) replaced.
    #[must_use]
    pub fn with_purpose(mut self, purpose: InputPurpose) -> Self {
        self.purpose = purpose;
        self
    }

    /// Returns a copy with the [`multiline`](Self::multiline) flag set.
    #[must_use]
    pub fn multiline(mut self, multiline: bool) -> Self {
        self.multiline = multiline;
        self
    }

    /// Returns a copy with the [`autocorrect`](Self::autocorrect) flag set.
    #[must_use]
    pub fn autocorrect(mut self, autocorrect: bool) -> Self {
        self.autocorrect = autocorrect;
        self
    }

    /// Returns a copy with the [`capitalization`](Self::capitalization) policy
    /// replaced.
    #[must_use]
    pub fn with_capitalization(mut self, capitalization: Capitalization) -> Self {
        self.capitalization = capitalization;
        self
    }

    /// Returns a copy with the [`spellcheck`](Self::spellcheck) flag set.
    #[must_use]
    pub fn spellcheck(mut self, spellcheck: bool) -> Self {
        self.spellcheck = spellcheck;
        self
    }

    /// Returns a copy with the [`sensitive`](Self::sensitive) flag set.
    #[must_use]
    pub fn sensitive(mut self, sensitive: bool) -> Self {
        self.sensitive = sensitive;
        self
    }

    /// Returns a copy with the [`preferred_language`](Self::preferred_language)
    /// (a BCP-47 tag) replaced.
    #[must_use]
    pub fn with_preferred_language(mut self, language: impl Into<Box<str>>) -> Self {
        self.preferred_language = Some(language.into());
        self
    }

    /// Returns the semantic purpose of the field.
    #[must_use]
    pub const fn purpose(&self) -> InputPurpose {
        self.purpose
    }

    /// Returns whether the field accepts multiple lines.
    #[must_use]
    pub const fn is_multiline(&self) -> bool {
        self.multiline
    }

    /// Returns whether automatic spelling correction is enabled.
    #[must_use]
    pub const fn is_autocorrect(&self) -> bool {
        self.autocorrect
    }

    /// Returns the automatic-capitalization policy.
    #[must_use]
    pub const fn capitalization(&self) -> Capitalization {
        self.capitalization
    }

    /// Returns whether spellchecking is enabled.
    #[must_use]
    pub const fn is_spellcheck(&self) -> bool {
        self.spellcheck
    }

    /// Returns the explicit sensitive flag. Prefer
    /// [`effective_sensitive`](Self::effective_sensitive) when deciding whether
    /// sensitive handling applies, since a sensitive purpose implies it too.
    #[must_use]
    pub const fn is_sensitive(&self) -> bool {
        self.sensitive
    }

    /// Returns the preferred BCP-47 language tag, if any.
    #[must_use]
    pub fn preferred_language(&self) -> Option<&str> {
        self.preferred_language.as_deref()
    }

    /// Returns whether sensitive handling applies: the explicit
    /// [`sensitive`](Self::sensitive) flag OR a sensitive
    /// [`purpose`](InputPurpose::is_sensitive) (spec §5.12).
    #[must_use]
    pub const fn effective_sensitive(&self) -> bool {
        self.sensitive || self.purpose.is_sensitive()
    }
}

impl Default for InputHints {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_hints() {
        let h = InputHints::default();
        assert_eq!(h.purpose(), InputPurpose::FreeForm);
        assert!(!h.is_multiline());
        assert!(h.is_autocorrect());
        assert_eq!(h.capitalization(), Capitalization::Sentences);
        assert!(h.is_spellcheck());
        assert!(!h.is_sensitive());
        assert_eq!(h.preferred_language(), None);
        assert!(!h.effective_sensitive());
    }

    #[test]
    fn builder_replaces_fields() {
        let h = InputHints::new()
            .with_purpose(InputPurpose::Email)
            .multiline(true)
            .autocorrect(false)
            .with_capitalization(Capitalization::Words)
            .spellcheck(false)
            .sensitive(true)
            .with_preferred_language("en-US");
        assert_eq!(h.purpose(), InputPurpose::Email);
        assert!(h.is_multiline());
        assert!(!h.is_autocorrect());
        assert_eq!(h.capitalization(), Capitalization::Words);
        assert!(!h.is_spellcheck());
        assert!(h.is_sensitive());
        assert_eq!(h.preferred_language(), Some("en-US"));
    }

    #[test]
    fn for_password_disables_assistance() {
        let h = InputHints::for_password();
        assert_eq!(h.purpose(), InputPurpose::Password);
        assert!(h.is_sensitive());
        assert!(!h.is_autocorrect());
        assert!(!h.is_spellcheck());
        assert_eq!(h.capitalization(), Capitalization::None);
        assert!(h.effective_sensitive());
    }

    #[test]
    fn effective_sensitive_folds_purpose() {
        // Not flagged, but the purpose is sensitive.
        let h = InputHints::new().with_purpose(InputPurpose::Pin);
        assert!(!h.is_sensitive());
        assert!(h.effective_sensitive());
        // Flagged explicitly, non-sensitive purpose.
        let h = InputHints::new().sensitive(true);
        assert!(h.effective_sensitive());
        // Neither.
        assert!(!InputHints::new().effective_sensitive());
    }
}
