use std::fmt;

/// Reason a service handle could not be issued or used.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityAccessError {
    /// The caller, handle, lifetime, or operation failed authorization.
    Denied,
    /// The issuer exhausted its handle identifiers.
    Exhausted,
}

impl fmt::Display for CapabilityAccessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Denied => "plugin capability access denied",
            Self::Exhausted => "plugin capability handle identifiers exhausted",
        })
    }
}

impl std::error::Error for CapabilityAccessError {}
