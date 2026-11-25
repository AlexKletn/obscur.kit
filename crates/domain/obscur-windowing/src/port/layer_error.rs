#[doc = include_str!("../../docs/api/port/layer_backend.doc/LayerError.md")]
#[derive(Debug)]
pub enum LayerError {
    #[doc = include_str!("../../docs/api/port/layer_backend.doc/Unsupported.md")]
    Unsupported,
    #[doc = include_str!("../../docs/api/port/layer_backend.doc/InvalidSettings.md")]
    InvalidSettings(Box<str>),
    /// The backend reported a failure as text.
    BackendFailure(Box<str>),
    /// The backend supplied the error that caused the operation to fail.
    BackendError(Box<dyn std::error::Error + Send + Sync>),
}

impl LayerError {
    /// Wraps a backend error while retaining its concrete type and source chain.
    pub fn backend(error: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self::BackendError(Box::new(error))
    }
}

impl std::fmt::Display for LayerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unsupported => f.write_str("this platform has no desktop layer surfaces"),
            Self::InvalidSettings(reason) => write!(f, "invalid layer settings: {reason}"),
            Self::BackendFailure(reason) => write!(f, "layer backend failure: {reason}"),
            Self::BackendError(error) => write!(f, "layer backend failure: {error}"),
        }
    }
}

impl std::error::Error for LayerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::BackendError(error) => Some(error.as_ref()),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use super::*;

    #[test]
    fn backend_failures_expose_the_original_error() {
        let error = LayerError::backend(std::io::Error::from(std::io::ErrorKind::BrokenPipe));
        let kind = error
            .source()
            .and_then(|source| source.downcast_ref::<std::io::Error>())
            .map(std::io::Error::kind);

        assert_eq!(kind, Some(std::io::ErrorKind::BrokenPipe));
    }

    #[test]
    fn the_error_text_names_the_platform_limit_rather_than_a_protocol() {
        let text = LayerError::Unsupported.to_string();
        assert!(text.contains("desktop layer surfaces"), "{text}");
        assert!(!text.to_lowercase().contains("wayland"), "{text}");
    }
}
