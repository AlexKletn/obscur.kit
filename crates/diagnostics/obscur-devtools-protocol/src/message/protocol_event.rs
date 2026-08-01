#![doc = include_str!("protocol_event.doc/_module.md")]

use super::{SubscriptionId, Topic};

#[doc = include_str!("protocol_event.doc/EventSeverity.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EventSeverity {
    #[doc = include_str!("protocol_event.doc/Trace.md")]
    Trace,
    #[doc = include_str!("protocol_event.doc/Info.md")]
    Info,
    #[doc = include_str!("protocol_event.doc/Warning.md")]
    Warning,
    #[doc = include_str!("protocol_event.doc/Error.md")]
    Error,
    #[doc = include_str!("protocol_event.doc/Critical.md")]
    Critical,
}

#[doc = include_str!("protocol_event.doc/EventKind.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventKind {
    #[doc = include_str!("protocol_event.doc/Diagnostic.md")]
    Diagnostic {
        #[doc = include_str!("protocol_event.doc/severity.md")]
        severity: EventSeverity,
        #[doc = include_str!("protocol_event.doc/code.md")]
        code: Box<str>,
        #[doc = include_str!("protocol_event.doc/message.md")]
        message: Box<str>,
    },
    #[doc = include_str!("protocol_event.doc/FrameCommitted.md")]
    FrameCommitted {
        #[doc = include_str!("protocol_event.doc/id.md")]
        id: u64,
    },
    #[doc = include_str!("protocol_event.doc/FocusChanged.md")]
    FocusChanged,
    #[doc = include_str!("protocol_event.doc/LifecycleChanged.md")]
    LifecycleChanged,
}

#[doc = include_str!("protocol_event.doc/ProtocolEvent.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolEvent {
    #[doc = include_str!("protocol_event.doc/subscription.md")]
    pub subscription: SubscriptionId,
    #[doc = include_str!("protocol_event.doc/topic.md")]
    pub topic: Topic,
    #[doc = include_str!("protocol_event.doc/kind.md")]
    pub kind: EventKind,
}

impl ProtocolEvent {
    #[doc = include_str!("protocol_event.doc/new.md")]
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
