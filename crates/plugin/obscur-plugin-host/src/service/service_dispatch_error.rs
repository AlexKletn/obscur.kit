use std::fmt;

use crate::capability::CapabilityAccessError;

/// Failure to register, authorize, or dispatch a service call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServiceDispatchError {
    /// The service or version is unavailable.
    Unsupported,
    /// This service ID and version already have a provider.
    AlreadyRegistered,
    /// The request failed the capability check.
    Authorization(CapabilityAccessError),
    /// The provider rejected an authorized request.
    Rejected(Box<str>),
    /// The handler panicked while starting or polling its future.
    Panicked,
}

impl fmt::Display for ServiceDispatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unsupported => f.write_str("plugin service version is unavailable"),
            Self::AlreadyRegistered => f.write_str("plugin service version is already registered"),
            Self::Authorization(error) => error.fmt(f),
            Self::Rejected(reason) => f.write_str(reason),
            Self::Panicked => f.write_str("plugin service handler panicked"),
        }
    }
}

impl std::error::Error for ServiceDispatchError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Authorization(error) => Some(error),
            _ => None,
        }
    }
}

impl From<CapabilityAccessError> for ServiceDispatchError {
    fn from(error: CapabilityAccessError) -> Self {
        Self::Authorization(error)
    }
}
