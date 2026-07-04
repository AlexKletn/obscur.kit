//! `ProtocolError`: typed protocol-level failures (spec §7.7).

use core::fmt;

use crate::message::PermissionScope;

/// A protocol-level error surfaced to a DevTools client.
///
/// These cover the failure modes the endpoint-security rules call out (§7.7):
/// unsupported versions, missing capability scope, exceeded limits, malformed or
/// unknown commands, and control being disabled entirely in production builds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProtocolError {
    /// No mutually supported protocol version could be negotiated.
    VersionUnsupported,
    /// The endpoint lacks the scope required for the attempted operation.
    Unauthorized {
        /// The scope that would have been required.
        required: PermissionScope,
    },
    /// A message, subscription, or rate limit was exceeded.
    LimitExceeded {
        /// The limit value that was exceeded.
        limit: usize,
    },
    /// The command referenced an unknown/unsupported operation.
    UnknownCommand,
    /// A message could not be decoded into a well-formed command.
    MalformedMessage,
    /// Control operations are disabled in this (production) build (spec §7.7).
    ControlDisabled,
}

impl fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::VersionUnsupported => {
                write!(f, "no mutually supported protocol version")
            }
            Self::Unauthorized { required } => {
                write!(f, "operation requires the {required:?} scope")
            }
            Self::LimitExceeded { limit } => {
                write!(f, "protocol limit of {limit} exceeded")
            }
            Self::UnknownCommand => write!(f, "unknown or unsupported command"),
            Self::MalformedMessage => write!(f, "malformed protocol message"),
            Self::ControlDisabled => {
                write!(f, "control operations are disabled in this build")
            }
        }
    }
}

impl core::error::Error for ProtocolError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_is_non_empty() {
        let errs = [
            ProtocolError::VersionUnsupported,
            ProtocolError::Unauthorized { required: PermissionScope::Control },
            ProtocolError::LimitExceeded { limit: 64 },
            ProtocolError::UnknownCommand,
            ProtocolError::MalformedMessage,
            ProtocolError::ControlDisabled,
        ];
        for e in errs {
            assert!(!format!("{e}").is_empty());
        }
    }

    #[test]
    fn is_std_error() {
        fn assert_error<E: core::error::Error>(_: &E) {}
        assert_error(&ProtocolError::UnknownCommand);
    }

    #[test]
    fn unauthorized_carries_required_scope() {
        let e = ProtocolError::Unauthorized { required: PermissionScope::Inject };
        assert!(format!("{e}").contains("Inject"));
    }
}
