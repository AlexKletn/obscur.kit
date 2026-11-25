/// A calendar date or locale could not be formatted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateFormatError {
    /// The input is not a valid ISO date.
    InvalidDate,
    /// The language tag could not be resolved to formatting data.
    LocaleUnavailable,
}

impl std::fmt::Display for DateFormatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::InvalidDate => "invalid calendar date",
            Self::LocaleUnavailable => "date formatting locale unavailable",
        })
    }
}

impl std::error::Error for DateFormatError {}
