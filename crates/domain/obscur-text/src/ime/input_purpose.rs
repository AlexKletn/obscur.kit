#![doc = include_str!("input_purpose.doc/_module.md")]

#[doc = include_str!("input_purpose.doc/InputPurpose.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum InputPurpose {
    #[doc = include_str!("input_purpose.doc/FreeForm.md")]
    #[default]
    FreeForm,
    #[doc = include_str!("input_purpose.doc/Password.md")]
    Password,
    #[doc = include_str!("input_purpose.doc/Pin.md")]
    Pin,
    #[doc = include_str!("input_purpose.doc/Email.md")]
    Email,
    #[doc = include_str!("input_purpose.doc/Url.md")]
    Url,
    #[doc = include_str!("input_purpose.doc/Number.md")]
    Number,
    #[doc = include_str!("input_purpose.doc/Phone.md")]
    Phone,
    #[doc = include_str!("input_purpose.doc/Search.md")]
    Search,
}

impl InputPurpose {
    #[doc = include_str!("input_purpose.doc/is_sensitive.md")]
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
