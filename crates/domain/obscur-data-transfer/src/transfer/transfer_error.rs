#![doc = include_str!("transfer_error.doc/_module.md")]

#[doc = include_str!("transfer_error.doc/TransferError.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransferError {
    #[doc = include_str!("transfer_error.doc/LimitExceeded.md")]
    LimitExceeded(Box<str>),
    #[doc = include_str!("transfer_error.doc/Unsupported.md")]
    Unsupported(Box<str>),
    #[doc = include_str!("transfer_error.doc/PermissionDenied.md")]
    PermissionDenied,
    #[doc = include_str!("transfer_error.doc/Empty.md")]
    Empty,
    #[doc = include_str!("transfer_error.doc/Cancelled.md")]
    Cancelled,
    #[doc = include_str!("transfer_error.doc/ProviderFailed.md")]
    ProviderFailed(Box<str>),
}

impl TransferError {
    #[doc = include_str!("transfer_error.doc/is_cancelled.md")]
    #[must_use]
    pub const fn is_cancelled(&self) -> bool {
        matches!(self, Self::Cancelled)
    }

    #[doc = include_str!("transfer_error.doc/is_denied.md")]
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
