//! Typed failures of a data transfer (spec §6.20).

/// Why a clipboard or drag-and-drop transfer failed (spec §6.20).
///
/// Each variant is a pure, backend-agnostic value; the informational variants
/// carry a human-readable reason describing the specific failure. This type
/// implements [`std::error::Error`] so it composes with the standard error
/// ecosystem while remaining I/O-free.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransferError {
    /// A capability limit was exceeded (spec §6.20 limits); the payload
    /// describes the limit that was hit.
    LimitExceeded(Box<str>),
    /// No offered representation matched the requested format (spec §6.20); the
    /// payload names the unsupported format or reason.
    Unsupported(Box<str>),
    /// A permission check denied the transfer (spec §6.20 permissions).
    PermissionDenied,
    /// There was nothing to transfer (spec §6.20).
    Empty,
    /// The transfer was cancelled before it completed (spec §6.20).
    Cancelled,
    /// A lazy data provider failed to resolve its bytes (spec §6.20 lazy async
    /// providers); the payload describes the provider failure.
    ProviderFailed(Box<str>),
}

impl TransferError {
    /// Returns whether this error is [`TransferError::Cancelled`] (spec §6.20).
    #[must_use]
    pub const fn is_cancelled(&self) -> bool {
        matches!(self, Self::Cancelled)
    }

    /// Returns whether this error is [`TransferError::PermissionDenied`] (spec
    /// §6.20 permissions).
    #[must_use]
    pub const fn is_denied(&self) -> bool {
        matches!(self, Self::PermissionDenied)
    }
}

impl std::fmt::Display for TransferError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LimitExceeded(reason) => write!(f, "transfer limit exceeded: {reason}"),
            Self::Unsupported(reason) => write!(f, "unsupported transfer format: {reason}"),
            Self::PermissionDenied => f.write_str("transfer permission denied"),
            Self::Empty => f.write_str("nothing to transfer"),
            Self::Cancelled => f.write_str("transfer cancelled"),
            Self::ProviderFailed(reason) => write!(f, "lazy provider failed: {reason}"),
        }
    }
}

impl std::error::Error for TransferError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_renders_each_variant() {
        assert_eq!(
            TransferError::LimitExceeded("too big".into()).to_string(),
            "transfer limit exceeded: too big"
        );
        assert_eq!(
            TransferError::Unsupported("text/xml".into()).to_string(),
            "unsupported transfer format: text/xml"
        );
        assert_eq!(
            TransferError::PermissionDenied.to_string(),
            "transfer permission denied"
        );
        assert_eq!(TransferError::Empty.to_string(), "nothing to transfer");
        assert_eq!(TransferError::Cancelled.to_string(), "transfer cancelled");
        assert_eq!(
            TransferError::ProviderFailed("io".into()).to_string(),
            "lazy provider failed: io"
        );
    }

    #[test]
    fn usable_as_std_error() {
        let err: &dyn std::error::Error = &TransferError::Empty;
        assert_eq!(err.to_string(), "nothing to transfer");
    }

    #[test]
    fn predicates_classify_variants() {
        assert!(TransferError::Cancelled.is_cancelled());
        assert!(!TransferError::Empty.is_cancelled());
        assert!(TransferError::PermissionDenied.is_denied());
        assert!(!TransferError::Cancelled.is_denied());
    }
}
