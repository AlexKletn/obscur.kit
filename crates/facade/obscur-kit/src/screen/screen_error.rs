#![doc = include_str!("screen_error.doc/_module.md")]

#[doc = include_str!("screen_error.doc/ScreenError.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScreenError {
    #[doc = include_str!("screen_error.doc/Parse.md")]
    Parse(Vec<String>),
    #[doc = include_str!("screen_error.doc/Compile.md")]
    Compile(Vec<String>),
}

impl ScreenError {
    #[doc = include_str!("screen_error.doc/messages.md")]
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
