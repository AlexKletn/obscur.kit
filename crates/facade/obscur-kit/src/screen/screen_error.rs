//! [`ScreenError`] — why an `.okui` source could not become an [`OkuiScreen`].

#[doc = include_str!("../../docs/api/screen/screen_error.doc/ScreenError.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScreenError {
    /// The source did not parse; one message per parse error.
    Parse(Vec<String>),
    /// The source parsed but failed to compile; one message per diagnostic.
    Compile(Vec<String>),
}

impl ScreenError {
    /// The diagnostics behind this failure, in report order.
    #[must_use]
    pub fn messages(&self) -> &[String] {
        match self {
            Self::Parse(messages) | Self::Compile(messages) => messages,
        }
    }
}

impl std::fmt::Display for ScreenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parse(errors) => {
                write!(f, "the .okui screen did not parse: {}", errors.join("; "))
            }
            Self::Compile(diagnostics) => write!(
                f,
                "the .okui screen failed to compile: {}",
                diagnostics.join("; ")
            ),
        }
    }
}

impl std::error::Error for ScreenError {}
