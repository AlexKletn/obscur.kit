use super::EventSeverity;

/// The typed body of a pushed [`ProtocolEvent`](super::ProtocolEvent).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventKind {
    /// A structured diagnostic was emitted.
    Diagnostic {
        /// Severity of the diagnostic.
        severity: EventSeverity,
        /// Stable diagnostic code.
        code: Box<str>,
        /// Human-readable message.
        message: Box<str>,
    },
    /// A frame was committed to the compositor.
    FrameCommitted {
        /// Monotonic frame id.
        id: u64,
    },
    /// The focused element changed.
    FocusChanged,
    /// A runtime/component lifecycle transition occurred.
    LifecycleChanged,
}
