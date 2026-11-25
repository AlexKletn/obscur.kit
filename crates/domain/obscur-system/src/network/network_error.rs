#[doc = include_str!("../../docs/api/network/network_service.doc/NetworkError.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetworkError {
    /// This service performs no network access at all — the honest kit default
    /// (spec §5.15).
    Unsupported,
    /// The host declined the fetch by policy (permission, sandbox, allowlist).
    Denied,
    /// The fetch did not complete within the host's time budget.
    Timeout,
    /// The URL was malformed or unroutable; the payload echoes the offending
    /// URL.
    InvalidUrl(Box<str>),
    /// An underlying transport failed; the payload describes the failure.
    Transport(Box<str>),
}

impl NetworkError {
    /// Returns whether this is the policy-refusal [`Denied`](Self::Denied)
    /// variant (spec §5.15).
    #[must_use]
    pub const fn is_denied(&self) -> bool {
        matches!(self, Self::Denied)
    }

    /// Returns whether this is the no-network [`Unsupported`](Self::Unsupported)
    /// variant (spec §5.15).
    #[must_use]
    pub const fn is_unsupported(&self) -> bool {
        matches!(self, Self::Unsupported)
    }
}

impl std::fmt::Display for NetworkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unsupported => f.write_str("network access is unsupported"),
            Self::Denied => f.write_str("network access denied"),
            Self::Timeout => f.write_str("network request timed out"),
            Self::InvalidUrl(url) => write!(f, "invalid url: {url}"),
            Self::Transport(reason) => write!(f, "network transport error: {reason}"),
        }
    }
}

impl std::error::Error for NetworkError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn network_error_display_and_predicates() {
        assert_eq!(
            NetworkError::Unsupported.to_string(),
            "network access is unsupported"
        );
        assert_eq!(NetworkError::Denied.to_string(), "network access denied");
        assert_eq!(
            NetworkError::Timeout.to_string(),
            "network request timed out"
        );
        assert_eq!(
            NetworkError::InvalidUrl(Box::from("bad")).to_string(),
            "invalid url: bad"
        );
        assert_eq!(
            NetworkError::Transport(Box::from("reset")).to_string(),
            "network transport error: reset"
        );

        assert!(NetworkError::Denied.is_denied());
        assert!(!NetworkError::Unsupported.is_denied());
        assert!(NetworkError::Unsupported.is_unsupported());
        assert!(!NetworkError::Denied.is_unsupported());

        let _: &dyn std::error::Error = &NetworkError::Timeout;
    }
}
