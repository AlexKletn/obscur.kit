use super::{HandshakeRejection, ProtocolVersion, ScopeSet};

#[doc = include_str!("../../docs/api/message/handshake.doc/HandshakeResponse.md")]
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
