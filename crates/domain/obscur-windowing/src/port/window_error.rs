#[doc = include_str!("../../docs/api/port/window_backend.doc/WindowError.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WindowError {
    /// The backend cannot create windows at all on this platform/configuration.
    Unsupported,
    /// The request was well-formed for the type system but semantically invalid
    /// for this backend, carrying a human-readable reason.
    InvalidRequest(Box<str>),
    /// The backend attempted creation but the platform failed, carrying a
    /// human-readable reason.
    BackendFailure(Box<str>),
}

impl std::fmt::Display for WindowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unsupported => f.write_str("window creation is unsupported"),
            Self::InvalidRequest(reason) => write!(f, "invalid window request: {reason}"),
            Self::BackendFailure(reason) => write!(f, "window backend failure: {reason}"),
        }
    }
}

impl std::error::Error for WindowError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn window_error_displays_and_is_error() {
        let err = WindowError::InvalidRequest(Box::from("bad size"));
        assert_eq!(err.to_string(), "invalid window request: bad size");
        let _: &dyn std::error::Error = &err;
        assert_eq!(
            WindowError::Unsupported.to_string(),
            "window creation is unsupported"
        );
        assert_eq!(
            WindowError::BackendFailure(Box::from("x11 down")).to_string(),
            "window backend failure: x11 down"
        );
    }
}
