//! Connection handshake: version negotiation + auth result (spec §7.6/§7.7).

use super::VersionRange;

#[doc = include_str!("../../docs/api/message/handshake.doc/HandshakeRequest.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HandshakeRequest {
    /// The range of protocol versions the client supports.
    pub client_versions: VersionRange,
    #[doc = include_str!("../../docs/api/message/handshake.doc/token.md")]
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

#[cfg(test)]
mod tests {
    use super::super::PermissionScope;
    use super::super::{HandshakeRejection, HandshakeResponse, ProtocolVersion, ScopeSet};
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
