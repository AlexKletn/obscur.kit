Performs the connection handshake: version negotiation *and* token
authentication.

On success the granted scopes and negotiated version are stored, the
session is marked connected, and an [`HandshakeResponse::Accepted`] is
returned. A disabled endpoint, an incompatible version range, or an
unrecognised/empty-scope token each yields an
[`HandshakeResponse::Rejected`].
