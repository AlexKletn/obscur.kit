use std::fmt;

use super::ServiceDispatchError;

/// Failure to issue a transport handle in a service session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServiceSessionError {
    /// The session or host-wide identifier space reached its limit.
    HandleLimit,
    /// The host revoked this session.
    Revoked,
    /// The service or its requested operations could not be granted.
    Dispatch(ServiceDispatchError),
}

impl fmt::Display for ServiceSessionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::HandleLimit => f.write_str("service session handle limit reached"),
            Self::Revoked => f.write_str("service session revoked"),
            Self::Dispatch(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for ServiceSessionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::HandleLimit | Self::Revoked => None,
            Self::Dispatch(error) => Some(error),
        }
    }
}
