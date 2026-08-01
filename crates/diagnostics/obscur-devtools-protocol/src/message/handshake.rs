#![doc = include_str!("handshake.doc/_module.md")]

use super::{ProtocolVersion, ScopeSet, VersionRange};

#[doc = include_str!("handshake.doc/HandshakeRequest.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HandshakeRequest {
    #[doc = include_str!("handshake.doc/client_versions.md")]
    pub client_versions: VersionRange,
    #[doc = include_str!("handshake.doc/token.md")]
    pub token: Box<str>,
}

impl HandshakeRequest {
    #[doc = include_str!("handshake.doc/new.md")]
    #[must_use]
    pub fn new(client_versions: VersionRange, token: impl Into<Box<str>>) -> Self {
        Self {
            client_versions,
            token: token.into(),
        }
    }
}

#[doc = include_str!("handshake.doc/HandshakeRejection.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandshakeRejection {
    #[doc = include_str!("handshake.doc/VersionUnsupported.md")]
    VersionUnsupported,
    #[doc = include_str!("handshake.doc/Unauthorized.md")]
    Unauthorized,
}

#[doc = include_str!("handshake.doc/HandshakeResponse.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HandshakeResponse {
    #[doc = include_str!("handshake.doc/Accepted.md")]
    Accepted {
        #[doc = include_str!("handshake.doc/version.md")]
        version: ProtocolVersion,
        #[doc = include_str!("handshake.doc/granted.md")]
        granted: ScopeSet,
    },
    #[doc = include_str!("handshake.doc/Rejected.md")]
    Rejected {
        #[doc = include_str!("handshake.doc/reason.md")]
        reason: HandshakeRejection,
    },
}

impl HandshakeResponse {
    #[doc = include_str!("handshake.doc/accepted.md")]
    #[must_use]
    pub const fn accepted(version: ProtocolVersion, granted: ScopeSet) -> Self {
        Self::Accepted { version, granted }
    }

    #[doc = include_str!("handshake.doc/rejected.md")]
    #[must_use]
    pub const fn rejected(reason: HandshakeRejection) -> Self {
        Self::Rejected { reason }
    }

    #[doc = include_str!("handshake.doc/is_accepted.md")]
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
