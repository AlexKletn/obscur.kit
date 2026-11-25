/// A decimal input or locale could not be formatted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumberFormatError {
    /// The input is not a supported decimal literal.
    InvalidDecimal,
    /// The language tag could not be resolved to formatting data.
    LocaleUnavailable,
}

impl std::fmt::Display for NumberFormatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::InvalidDecimal => "invalid decimal number",
            Self::LocaleUnavailable => "number formatting locale unavailable",
        })
    }
}

impl std::error::Error for NumberFormatError {}
