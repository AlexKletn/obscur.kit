//! Why a resource failed to load, decode, or compile (spec §5.15 failed state).

#[doc = include_str!("../../docs/api/state/resource_error.doc/ResourceError.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceError {
    /// The requested resource could not be located at its source.
    NotFound,
    /// The resource's format is unknown or unsupported by the available loaders
    /// (spec §5.15); the payload names the offending format or reason.
    Unsupported(Box<str>),
    /// Decoding the resource's bytes into a CPU representation failed (spec
    /// §5.15); the payload describes the decode failure.
    DecodeFailed(Box<str>),
    /// Compiling the resource (for example a shader) failed (spec §5.15); the
    /// payload describes the compilation failure.
    CompileFailed(Box<str>),
    /// A safety limit was exceeded, such as a maximum dimension or byte budget
    /// (spec §5.16); the payload describes the limit that was hit.
    LimitExceeded(Box<str>),
    /// The resource's backing source became unavailable during loading (spec
    /// §5.15); the payload describes the source failure.
    SourceUnavailable(Box<str>),
    /// The load job was cancelled, typically because a newer version superseded
    /// it (spec §5.15 stale-job rejection).
    Cancelled,
}

impl ResourceError {
    /// Returns whether this error is [`ResourceError::Cancelled`] (spec §5.15
    /// stale-job rejection).
    #[must_use]
    pub const fn is_cancelled(&self) -> bool {
        matches!(self, Self::Cancelled)
    }
}

impl std::fmt::Display for ResourceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound => f.write_str("resource not found"),
            Self::Unsupported(reason) => write!(f, "unsupported resource: {reason}"),
            Self::DecodeFailed(reason) => write!(f, "decode failed: {reason}"),
            Self::CompileFailed(reason) => write!(f, "compile failed: {reason}"),
            Self::LimitExceeded(reason) => write!(f, "limit exceeded: {reason}"),
            Self::SourceUnavailable(reason) => write!(f, "source unavailable: {reason}"),
            Self::Cancelled => f.write_str("resource load cancelled"),
        }
    }
}

impl std::error::Error for ResourceError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_renders_each_variant() {
        assert_eq!(ResourceError::NotFound.to_string(), "resource not found");
        assert_eq!(
            ResourceError::Unsupported("webp".into()).to_string(),
            "unsupported resource: webp"
        );
        assert_eq!(
            ResourceError::DecodeFailed("bad header".into()).to_string(),
            "decode failed: bad header"
        );
        assert_eq!(
            ResourceError::CompileFailed("syntax".into()).to_string(),
            "compile failed: syntax"
        );
        assert_eq!(
            ResourceError::LimitExceeded("too large".into()).to_string(),
            "limit exceeded: too large"
        );
        assert_eq!(
            ResourceError::SourceUnavailable("network".into()).to_string(),
            "source unavailable: network"
        );
        assert_eq!(
            ResourceError::Cancelled.to_string(),
            "resource load cancelled"
        );
    }

    #[test]
    fn usable_as_std_error() {
        let err: &dyn std::error::Error = &ResourceError::NotFound;
        assert_eq!(err.to_string(), "resource not found");
    }

    #[test]
    fn is_cancelled_only_for_cancelled() {
        assert!(ResourceError::Cancelled.is_cancelled());
        assert!(!ResourceError::NotFound.is_cancelled());
        assert!(!ResourceError::DecodeFailed("x".into()).is_cancelled());
    }
}
