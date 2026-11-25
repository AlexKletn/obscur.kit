use serde::{Deserialize, Serialize};

use super::ServiceDispatchError;

/// Stable, redacted status for failures crossing a JSON service boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceWireError {
    /// This registration is unavailable.
    Unsupported,
    /// The host denied the service operation.
    Denied,
    /// The operation name and request payload exceed the request budget.
    RequestTooLarge,
    /// The JSON request does not match the service contract.
    InvalidRequest,
    /// The encoded reply would exceed the response budget.
    ResponseTooLarge,
    /// The provider rejected the call or the response could not be encoded.
    Failed,
}

impl From<ServiceDispatchError> for ServiceWireError {
    fn from(error: ServiceDispatchError) -> Self {
        match error {
            ServiceDispatchError::Unsupported => Self::Unsupported,
            ServiceDispatchError::Authorization(_) => Self::Denied,
            ServiceDispatchError::AlreadyRegistered
            | ServiceDispatchError::Rejected(_)
            | ServiceDispatchError::Panicked => Self::Failed,
        }
    }
}
