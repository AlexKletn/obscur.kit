use std::{error::Error, fmt};

/// Invalid manifest content or a decoding failure from the JSON parser.
#[derive(Debug)]
pub enum ManifestError {
    /// A manifest rule failed after decoding.
    Validation(String),
    /// The JSON parser could not produce a manifest.
    Decode(serde_json::Error),
}

impl From<String> for ManifestError {
    fn from(message: String) -> Self {
        Self::Validation(message)
    }
}

impl From<&str> for ManifestError {
    fn from(message: &str) -> Self {
        Self::Validation(message.into())
    }
}

impl fmt::Display for ManifestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Validation(message) => f.write_str(message),
            Self::Decode(error) => error.fmt(f),
        }
    }
}

impl Error for ManifestError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Decode(error) => Some(error),
            Self::Validation(_) => None,
        }
    }
}
