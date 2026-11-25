//! The semantic purpose of a text field, guiding IME and keyboard behaviour
//! (spec §5.12 IME).

#[doc = include_str!("../../docs/api/ime/input_purpose.doc/InputPurpose.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum InputPurpose {
    /// Ordinary free-form text with no special semantics. The default.
    #[default]
    FreeForm,
    /// A secret password; sensitive-mode handling applies.
    Password,
    /// A numeric PIN; sensitive-mode handling applies.
    Pin,
    /// An email address.
    Email,
    /// A URL.
    Url,
    /// A numeric value.
    Number,
    /// A telephone number.
    Phone,
    /// A search query.
    Search,
}

impl InputPurpose {
    #[doc = include_str!("../../docs/api/ime/input_purpose.doc/is_sensitive.md")]
    #[must_use]
    pub const fn is_sensitive(&self) -> bool {
        matches!(self, Self::Password | Self::Pin)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_free_form() {
        assert_eq!(InputPurpose::default(), InputPurpose::FreeForm);
    }

    #[test]
    fn is_sensitive_only_for_password_and_pin() {
        assert!(InputPurpose::Password.is_sensitive());
        assert!(InputPurpose::Pin.is_sensitive());
        assert!(!InputPurpose::FreeForm.is_sensitive());
        assert!(!InputPurpose::Email.is_sensitive());
        assert!(!InputPurpose::Url.is_sensitive());
        assert!(!InputPurpose::Number.is_sensitive());
        assert!(!InputPurpose::Phone.is_sensitive());
        assert!(!InputPurpose::Search.is_sensitive());
    }
}
