#![doc = include_str!("input_hints.doc/_module.md")]

use super::InputPurpose;

#[doc = include_str!("input_hints.doc/InputHints.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputHints {
    #[doc = include_str!("input_hints.doc/purpose.md")]
    purpose: InputPurpose,
    #[doc = include_str!("input_hints.doc/multiline.md")]
    multiline: bool,
    #[doc = include_str!("input_hints.doc/autocorrect.md")]
    autocorrect: bool,
    #[doc = include_str!("input_hints.doc/capitalization.md")]
    capitalization: Capitalization,
    #[doc = include_str!("input_hints.doc/spellcheck.md")]
    spellcheck: bool,
    #[doc = include_str!("input_hints.doc/sensitive.md")]
    sensitive: bool,
    #[doc = include_str!("input_hints.doc/preferred_language.md")]
    preferred_language: Option<Box<str>>,
}

impl InputHints {
    #[doc = include_str!("input_hints.doc/new.md")]
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

    #[doc = include_str!("input_hints.doc/for_password.md")]
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

    #[doc = include_str!("input_hints.doc/with_purpose.md")]
    #[must_use]
    pub fn with_purpose(mut self, purpose: InputPurpose) -> Self {
        self.purpose = purpose;
        self
    }

    #[doc = include_str!("input_hints.doc/multiline_2.md")]
    #[must_use]
    pub fn multiline(mut self, multiline: bool) -> Self {
        self.multiline = multiline;
        self
    }

    #[doc = include_str!("input_hints.doc/autocorrect_2.md")]
    #[must_use]
    pub fn autocorrect(mut self, autocorrect: bool) -> Self {
        self.autocorrect = autocorrect;
        self
    }

    #[doc = include_str!("input_hints.doc/with_capitalization.md")]
    #[must_use]
    pub fn with_capitalization(mut self, capitalization: Capitalization) -> Self {
        self.capitalization = capitalization;
        self
    }

    #[doc = include_str!("input_hints.doc/spellcheck_2.md")]
    #[must_use]
    pub fn spellcheck(mut self, spellcheck: bool) -> Self {
        self.spellcheck = spellcheck;
        self
    }

    #[doc = include_str!("input_hints.doc/sensitive_2.md")]
    #[must_use]
    pub fn sensitive(mut self, sensitive: bool) -> Self {
        self.sensitive = sensitive;
        self
    }

    #[doc = include_str!("input_hints.doc/with_preferred_language.md")]
    #[must_use]
    pub fn with_preferred_language(mut self, language: impl Into<Box<str>>) -> Self {
        self.preferred_language = Some(language.into());
        self
    }

    #[doc = include_str!("input_hints.doc/purpose_2.md")]
    #[must_use]
    pub const fn purpose(&self) -> InputPurpose {
        self.purpose
    }

    #[doc = include_str!("input_hints.doc/is_multiline.md")]
    #[must_use]
    pub const fn is_multiline(&self) -> bool {
        self.multiline
    }

    #[doc = include_str!("input_hints.doc/is_autocorrect.md")]
    #[must_use]
    pub const fn is_autocorrect(&self) -> bool {
        self.autocorrect
    }

    #[doc = include_str!("input_hints.doc/capitalization_2.md")]
    #[must_use]
    pub const fn capitalization(&self) -> Capitalization {
        self.capitalization
    }

    #[doc = include_str!("input_hints.doc/is_spellcheck.md")]
    #[must_use]
    pub const fn is_spellcheck(&self) -> bool {
        self.spellcheck
    }

    #[doc = include_str!("input_hints.doc/is_sensitive.md")]
    #[must_use]
    pub const fn is_sensitive(&self) -> bool {
        self.sensitive
    }

    #[doc = include_str!("input_hints.doc/preferred_language_2.md")]
    #[must_use]
    pub fn preferred_language(&self) -> Option<&str> {
        self.preferred_language.as_deref()
    }

    #[doc = include_str!("input_hints.doc/effective_sensitive.md")]
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

#[doc = include_str!("input_hints.doc/Capitalization.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Capitalization {
    #[doc = include_str!("input_hints.doc/None.md")]
    #[default]
    None,
    #[doc = include_str!("input_hints.doc/Sentences.md")]
    Sentences,
    #[doc = include_str!("input_hints.doc/Words.md")]
    Words,
    #[doc = include_str!("input_hints.doc/Characters.md")]
    Characters,
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
    fn capitalization_own_default_is_none() {
        assert_eq!(Capitalization::default(), Capitalization::None);
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
