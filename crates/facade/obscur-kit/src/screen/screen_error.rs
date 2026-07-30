//! [`ScreenError`] — why an `.okui` source could not become an [`OkuiScreen`].

/// Why [`OkuiScreen::compile`](super::OkuiScreen::compile) rejected a source.
///
/// The two variants are the two halves of the front end: the `.okui` **parser**
/// (syntax) and the **compiler** (§2.13 contract validation, §2.12 slots,
/// capabilities). Each carries one message per diagnostic rather than a single
/// joined string, so a host can report them individually — and neither is ever a
/// silently empty screen, which is what a `.okui` root that failed to compile used
/// to render as.
///
/// `AppError` converts from this, so a windowed
/// application sees one error type; the split is kept here because the screen
/// front end is display-free and usable without `App`.
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
