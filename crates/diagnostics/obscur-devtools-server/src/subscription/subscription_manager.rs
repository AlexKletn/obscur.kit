#![doc = include_str!("subscription_manager.doc/_module.md")]

use obscur_devtools_protocol::error::ProtocolError;
use obscur_devtools_protocol::message::{
    EventKind, ProtocolEvent, SUBSCRIPTION_LIMIT, SubscriptionId, Topic,
};

#[doc = include_str!("subscription_manager.doc/SubscriptionManager.md")]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SubscriptionManager {
    entries: Vec<(SubscriptionId, Topic)>,
    next_id: u64,
}

impl SubscriptionManager {
    #[doc = include_str!("subscription_manager.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            next_id: 0,
        }
    }

    #[doc = include_str!("subscription_manager.doc/subscribe.md")]
    pub fn subscribe(&mut self, topic: Topic) -> Result<SubscriptionId, ProtocolError> {
        if self.entries.len() >= SUBSCRIPTION_LIMIT {
            return Err(ProtocolError::LimitExceeded {
                limit: SUBSCRIPTION_LIMIT,
            });
        }
        self.next_id += 1;
        let id = SubscriptionId::new(self.next_id);
        self.entries.push((id, topic));
        Ok(id)
    }

    #[doc = include_str!("subscription_manager.doc/unsubscribe.md")]
    pub fn unsubscribe(&mut self, id: SubscriptionId) -> bool {
        let before = self.entries.len();
        self.entries.retain(|(existing, _)| *existing != id);
        self.entries.len() != before
    }

    #[doc = include_str!("subscription_manager.doc/subscribers_of.md")]
    #[must_use]
    pub fn subscribers_of(&self, topic: Topic) -> Vec<SubscriptionId> {
        self.entries
            .iter()
            .filter(|(_, t)| *t == topic)
            .map(|(id, _)| *id)
            .collect()
    }

    #[doc = include_str!("subscription_manager.doc/fan_out.md")]
    #[must_use]
    pub fn fan_out(&self, topic: Topic, kind: &EventKind) -> Vec<ProtocolEvent> {
        self.entries
            .iter()
            .filter(|(_, t)| *t == topic)
            .map(|(id, _)| ProtocolEvent::new(*id, topic, kind.clone()))
            .collect()
    }

    #[doc = include_str!("subscription_manager.doc/len.md")]
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    #[doc = include_str!("subscription_manager.doc/is_empty.md")]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_devtools_protocol::message::EventSeverity;

    #[test]
    fn subscribe_mints_increasing_ids() {
        let mut mgr = SubscriptionManager::new();
        let a = mgr.subscribe(Topic::Frames).expect("under limit");
        let b = mgr.subscribe(Topic::Frames).expect("under limit");
        assert_ne!(a, b);
        assert_eq!(mgr.len(), 2);
    }

    #[test]
    fn subscribe_enforces_limit() {
        let mut mgr = SubscriptionManager::new();
        for _ in 0..SUBSCRIPTION_LIMIT {
            mgr.subscribe(Topic::Diagnostics).expect("within limit");
        }
        assert_eq!(mgr.len(), SUBSCRIPTION_LIMIT);
        assert_eq!(
            mgr.subscribe(Topic::Diagnostics),
            Err(ProtocolError::LimitExceeded {
                limit: SUBSCRIPTION_LIMIT
            })
        );
    }

    #[test]
    fn unsubscribe_reports_presence() {
        let mut mgr = SubscriptionManager::new();
        let id = mgr.subscribe(Topic::Focus).expect("under limit");
        assert!(mgr.unsubscribe(id));
        assert!(!mgr.unsubscribe(id));
        assert!(mgr.is_empty());
    }

    #[test]
    fn fan_out_targets_only_matching_topic() {
        let mut mgr = SubscriptionManager::new();
        let frames_a = mgr.subscribe(Topic::Frames).expect("under limit");
        let frames_b = mgr.subscribe(Topic::Frames).expect("under limit");
        let _focus = mgr.subscribe(Topic::Focus).expect("under limit");

        let events = mgr.fan_out(Topic::Frames, &EventKind::FrameCommitted { id: 9 });
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].subscription, frames_a);
        assert_eq!(events[1].subscription, frames_b);
        assert!(events.iter().all(|e| e.topic == Topic::Frames));

        let none = mgr.fan_out(
            Topic::Lifecycle,
            &EventKind::Diagnostic {
                severity: EventSeverity::Info,
                code: "x".into(),
                message: "y".into(),
            },
        );
        assert!(none.is_empty());
    }

    #[test]
    fn subscribers_of_lists_matching_ids() {
        let mut mgr = SubscriptionManager::new();
        let a = mgr.subscribe(Topic::Frames).expect("under limit");
        let _b = mgr.subscribe(Topic::Focus).expect("under limit");
        assert_eq!(mgr.subscribers_of(Topic::Frames), vec![a]);
        assert!(mgr.subscribers_of(Topic::Diagnostics).is_empty());
    }
}
