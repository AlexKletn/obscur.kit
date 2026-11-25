Creates a disabled endpoint from an authenticator and a runtime bridge.

The endpoint supports exactly the current protocol version until
[`with_supported_versions`](Self::with_supported_versions) widens it, and
must be [`enable`](Self::enable)d before it will accept a handshake.
