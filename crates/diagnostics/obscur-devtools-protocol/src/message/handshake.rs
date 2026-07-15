//! Connection handshake: version negotiation + auth result (spec §7.6/§7.7).

use super::{ProtocolVersion, ScopeSet, VersionRange};

/// A client's request to open a DevTools connection.
///
/// Carries the versions the client can speak and an opaque authentication
/// token/capability. Version negotiation and token validation both happen on
/// the server; this crate only models the envelope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HandshakeRequest {
    /// The range of protocol versions the client supports.
    pub client_versions: VersionRange,
    /// An opaque authentication token/capability presented by the client.
    ///
    /// This crate treats the token as opaque; the server validates it against
    /// its own auth store and must never log it (spec §7.7).
    pub token: Box<str>,
}

impl HandshakeRequest {
    /// Creates a handshake request.
    #[must_use]
    pub fn new(client_versions: VersionRange, token: impl Into<Box<str>>) -> Self {
        Self {
            client_versions,
            token: token.into(),
        }
    }
}

/// Why a handshake was rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandshakeRejection {
    /// No mutually supported protocol version exists.
    VersionUnsupported,
    /// The presented token was missing, invalid, or insufficient.
    Unauthorized,
}

/// The server's response to a [`HandshakeRequest`].
///
/// Combines the negotiation outcome and the authorization result into a single
/// connect envelope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HandshakeResponse {
    /// The connection was accepted at a negotiated version with granted scopes.
    Accepted {
        /// The negotiated protocol version both sides will speak.
        version: ProtocolVersion,
        /// The scopes granted to this endpoint.
        granted: ScopeSet,
    },
    /// The connection was refused.
    Rejected {
        /// Why the handshake was refused.
        reason: HandshakeRejection,
    },
}

impl HandshakeResponse {
    /// Creates an acceptance response.
    #[must_use]
    pub const fn accepted(version: ProtocolVersion, granted: ScopeSet) -> Self {
        Self::Accepted { version, granted }
    }

    /// Creates a rejection response.
    #[must_use]
    pub const fn rejected(reason: HandshakeRejection) -> Self {
        Self::Rejected { reason }
    }

    /// Returns whether the handshake was accepted.
    #[must_use]
    pub const fn is_accepted(&self) -> bool {
        matches!(self, Self::Accepted { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::super::PermissionScope;
    use super::*;

    fn range() -> VersionRange {
        VersionRange::exact(ProtocolVersion::CURRENT)
    }

    #[test]
    fn request_stores_token_and_versions() {
        let req = HandshakeRequest::new(range(), "secret-token");
        assert_eq!(&*req.token, "secret-token");
        assert_eq!(req.client_versions, range());
    }

    #[test]
    fn accepted_shape() {
        let granted = ScopeSet::observer();
        let resp = HandshakeResponse::accepted(ProtocolVersion::CURRENT, granted);
        assert!(resp.is_accepted());
        match resp {
            HandshakeResponse::Accepted { version, granted } => {
                assert_eq!(version, ProtocolVersion::CURRENT);
                assert!(granted.contains(PermissionScope::Subscribe));
            }
            HandshakeResponse::Rejected { .. } => panic!("expected acceptance"),
        }
    }

    #[test]
    fn rejected_shape() {
        let resp = HandshakeResponse::rejected(HandshakeRejection::Unauthorized);
        assert!(!resp.is_accepted());
        assert_eq!(
            resp,
            HandshakeResponse::Rejected {
                reason: HandshakeRejection::Unauthorized
            }
        );
    }
}
