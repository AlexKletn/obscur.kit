#[doc = include_str!("../../docs/api/document/document_service.doc/DocumentError.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DocumentError {
    /// No document or directory matched the handle's token.
    NotFound,
    /// The operation was not permitted — for example a write to a read-only
    /// handle (spec §6.10 permission token).
    PermissionDenied,
    /// The backend does not support this operation; the payload names what.
    Unsupported(Box<str>),
    /// An underlying I/O operation failed; the payload describes the failure.
    Io(Box<str>),
}

impl std::fmt::Display for DocumentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound => f.write_str("document not found"),
            Self::PermissionDenied => f.write_str("document permission denied"),
            Self::Unsupported(what) => write!(f, "unsupported document operation: {what}"),
            Self::Io(reason) => write!(f, "document i/o error: {reason}"),
        }
    }
}

impl std::error::Error for DocumentError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn document_error_displays_and_is_error() {
        assert_eq!(DocumentError::NotFound.to_string(), "document not found");
        assert_eq!(
            DocumentError::PermissionDenied.to_string(),
            "document permission denied"
        );
        assert_eq!(
            DocumentError::Unsupported("create".into()).to_string(),
            "unsupported document operation: create"
        );
        assert_eq!(
            DocumentError::Io("disk full".into()).to_string(),
            "document i/o error: disk full"
        );
        let _: &dyn std::error::Error = &DocumentError::NotFound;
    }
}
