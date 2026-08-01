#![doc = include_str!("subscription_message.doc/_module.md")]

use super::{CommandId, Topic};

#[doc = include_str!("subscription_message.doc/SUBSCRIPTION_LIMIT.md")]
pub const SUBSCRIPTION_LIMIT: usize = 64;

#[doc = include_str!("subscription_message.doc/MESSAGE_LIMIT.md")]
pub const MESSAGE_LIMIT: usize = 1024;

#[doc = include_str!("subscription_message.doc/SubscriptionId.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct SubscriptionId(pub u64);

impl SubscriptionId {
    #[doc = include_str!("subscription_message.doc/new.md")]
    #[must_use]
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    #[doc = include_str!("subscription_message.doc/raw.md")]
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }
}

#[doc = include_str!("subscription_message.doc/SubscribeRequest.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SubscribeRequest {
    #[doc = include_str!("subscription_message.doc/command.md")]
    pub command: CommandId,
    #[doc = include_str!("subscription_message.doc/topic.md")]
    pub topic: Topic,
}

impl SubscribeRequest {
    #[doc = include_str!("subscription_message.doc/new_2.md")]
    #[must_use]
    pub const fn new(command: CommandId, topic: Topic) -> Self {
        Self { command, topic }
    }
}

#[doc = include_str!("subscription_message.doc/Subscription.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Subscription {
    #[doc = include_str!("subscription_message.doc/id.md")]
    pub id: SubscriptionId,
    #[doc = include_str!("subscription_message.doc/topic_2.md")]
    pub topic: Topic,
    #[doc = include_str!("subscription_message.doc/granted.md")]
    pub granted: bool,
}

impl Subscription {
    #[doc = include_str!("subscription_message.doc/granted_2.md")]
    #[must_use]
    pub const fn granted(id: SubscriptionId, topic: Topic) -> Self {
        Self {
            id,
            topic,
            granted: true,
        }
    }

    #[doc = include_str!("subscription_message.doc/denied.md")]
    #[must_use]
    pub const fn denied(id: SubscriptionId, topic: Topic) -> Self {
        Self {
            id,
            topic,
            granted: false,
        }
    }
}

#[doc = include_str!("subscription_message.doc/SubscribeResponse.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SubscribeResponse {
    #[doc = include_str!("subscription_message.doc/command_2.md")]
    pub command: CommandId,
    #[doc = include_str!("subscription_message.doc/subscription.md")]
    pub subscription: Subscription,
}

impl SubscribeResponse {
    #[doc = include_str!("subscription_message.doc/new_3.md")]
    #[must_use]
    pub const fn new(command: CommandId, subscription: Subscription) -> Self {
        Self {
            command,
            subscription,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn limits_are_sane() {
        let subscriptions = SUBSCRIPTION_LIMIT;
        let messages = MESSAGE_LIMIT;
        assert!(subscriptions > 0);
        assert!(messages >= subscriptions);
    }

    #[test]
    fn subscription_id_round_trip() {
        assert_eq!(SubscriptionId::new(5).raw(), 5);
        assert_eq!(SubscriptionId::default(), SubscriptionId::new(0));
    }

    #[test]
    fn granted_and_denied_records() {
        let g = Subscription::granted(SubscriptionId::new(1), Topic::Frames);
        assert!(g.granted);
        let d = Subscription::denied(SubscriptionId::new(2), Topic::Frames);
        assert!(!d.granted);
    }

    #[test]
    fn response_round_trips() {
        let sub = Subscription::granted(SubscriptionId::new(7), Topic::Focus);
        let resp = SubscribeResponse::new(CommandId::new(3), sub);
        assert_eq!(resp.command, CommandId::new(3));
        assert_eq!(resp.subscription, sub);
        let req = SubscribeRequest::new(CommandId::new(3), Topic::Focus);
        assert_eq!(req.topic, resp.subscription.topic);
    }
}
