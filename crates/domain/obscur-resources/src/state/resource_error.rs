#![doc = include_str!("resource_error.doc/_module.md")]

#[doc = include_str!("resource_error.doc/ResourceError.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceError {
    #[doc = include_str!("resource_error.doc/NotFound.md")]
    NotFound,
    #[doc = include_str!("resource_error.doc/Unsupported.md")]
    Unsupported(Box<str>),
    #[doc = include_str!("resource_error.doc/DecodeFailed.md")]
    DecodeFailed(Box<str>),
    #[doc = include_str!("resource_error.doc/CompileFailed.md")]
    CompileFailed(Box<str>),
    #[doc = include_str!("resource_error.doc/LimitExceeded.md")]
    LimitExceeded(Box<str>),
    #[doc = include_str!("resource_error.doc/SourceUnavailable.md")]
    SourceUnavailable(Box<str>),
    #[doc = include_str!("resource_error.doc/Cancelled.md")]
    Cancelled,
}

impl ResourceError {
    #[doc = include_str!("resource_error.doc/is_cancelled.md")]
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
