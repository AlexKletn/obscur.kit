#[doc = include_str!("../../docs/api/validation/expression_validator.doc/ValidationDiagnostic.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationDiagnostic {
    message: Box<str>,
    offset: u32,
}

impl ValidationDiagnostic {
    /// Creates a diagnostic with `message` at byte `offset`.
    #[must_use]
    pub fn new(message: impl Into<Box<str>>, offset: u32) -> Self {
        Self {
            message: message.into(),
            offset,
        }
    }

    /// Returns the diagnostic message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns the byte offset into the source where the issue was detected.
    #[must_use]
    pub const fn offset(&self) -> u32 {
        self.offset
    }
}
