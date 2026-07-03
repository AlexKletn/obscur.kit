//! The semantic purpose of a text field, guiding IME and keyboard behaviour
//! (spec §5.12 IME).

/// The semantic purpose of a text field (spec §5.12).
///
/// The purpose is a hint that platform IMEs, on-screen keyboards, and autofill
/// use to tailor input: a [`Password`](Self::Password) field suppresses
/// learning and suggestions, a [`Number`](Self::Number) field offers a numeric
/// keypad, an [`Email`](Self::Email) field surfaces `@`, and so on. It carries
/// no validation semantics of its own — it only describes intent.
///
/// [`is_sensitive`](Self::is_sensitive) reports whether the purpose implies
/// sensitive-mode handling (per §5.12 "sensitive/password modes are
/// respected").
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
    /// Returns whether this purpose implies sensitive-mode handling: true for
    /// [`Password`](Self::Password) and [`Pin`](Self::Pin) (spec §5.12).
    ///
    /// This is a hint that the field holds a secret, so IME learning,
    /// suggestions, and clipboard exposure should be suppressed even when the
    /// caller did not set an explicit sensitive flag.
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
