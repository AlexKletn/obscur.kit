/// Why a handshake was rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandshakeRejection {
    /// No mutually supported protocol version exists.
    VersionUnsupported,
    /// The presented token was missing, invalid, or insufficient.
    Unauthorized,
}
