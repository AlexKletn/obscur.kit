//! ShutdownRequest (skeleton stub for the spec-defined entity; not yet implemented).

use obscur_identity::WindowId;

/// Identifies the source of a shutdown request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShutdownRequest {
    /// The host or application requested shutdown.
    Host,
    /// The standalone host received a close request for its window.
    WindowClose(WindowId),
}
