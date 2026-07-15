//! Server→subscriber pushed events (spec §7.6/§7.8).

use super::{SubscriptionId, Topic};

/// Severity of a diagnostic event, mirroring the runtime's severity ladder.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EventSeverity {
    /// Fine-grained tracing detail.
    Trace,
    /// Informational message.
    Info,
    /// A recoverable concern worth surfacing.
    Warning,
    /// A recoverable error handled by fallback.
    Error,
    /// An internal invariant failure.
    Critical,
}

/// The typed body of a pushed [`ProtocolEvent`].
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

/// An event pushed to a subscriber over an active subscription.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolEvent {
    /// The subscription this event is delivered on.
    pub subscription: SubscriptionId,
    /// The topic the event belongs to.
    pub topic: Topic,
    /// The typed event body.
    pub kind: EventKind,
}

impl ProtocolEvent {
    /// Creates a protocol event.
    #[must_use]
    pub fn new(subscription: SubscriptionId, topic: Topic, kind: EventKind) -> Self {
        Self {
            subscription,
            topic,
            kind,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn severity_orders_low_to_high() {
        assert!(EventSeverity::Trace < EventSeverity::Critical);
        assert!(EventSeverity::Warning < EventSeverity::Error);
    }

    #[test]
    fn diagnostic_event_round_trips() {
        let kind = EventKind::Diagnostic {
            severity: EventSeverity::Warning,
            code: "OKUI-0001".into(),
            message: "slow frame".into(),
        };
        let event = ProtocolEvent::new(SubscriptionId::new(3), Topic::Diagnostics, kind.clone());
        assert_eq!(event.subscription, SubscriptionId::new(3));
        assert_eq!(event.topic, Topic::Diagnostics);
        assert_eq!(event.kind, kind);
    }

    #[test]
    fn frame_event_carries_id() {
        let event = ProtocolEvent::new(
            SubscriptionId::new(1),
            Topic::Frames,
            EventKind::FrameCommitted { id: 128 },
        );
        assert_eq!(event.kind, EventKind::FrameCommitted { id: 128 });
    }
}
