//! Subscribe/unsubscribe envelope, subscription record, and limits (§7.7).

use super::{CommandId, Topic};

/// Maximum number of concurrent subscriptions a single endpoint may hold
/// (spec §7.7 subscription limits).
pub const SUBSCRIPTION_LIMIT: usize = 64;

/// Maximum number of in-flight/queued messages permitted for an endpoint
/// before the server applies backpressure or disconnects (spec §7.7).
pub const MESSAGE_LIMIT: usize = 1024;

/// A server-assigned identifier for an active subscription.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct SubscriptionId(pub u64);

impl SubscriptionId {
    /// Creates a subscription id from a raw value.
    #[must_use]
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    /// Returns the raw identifier value.
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }
}

/// A request to subscribe to an event stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SubscribeRequest {
    /// Correlation id of the issuing command.
    pub command: CommandId,
    /// The topic to subscribe to.
    pub topic: Topic,
}

impl SubscribeRequest {
    /// Creates a subscribe request.
    #[must_use]
    pub const fn new(command: CommandId, topic: Topic) -> Self {
        Self { command, topic }
    }
}

/// A subscription record returned to (and retained by) the client.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Subscription {
    /// Server-assigned subscription id.
    pub id: SubscriptionId,
    /// The topic this subscription delivers.
    pub topic: Topic,
    /// Whether the subscription was actually granted (denied when the
    /// [`SUBSCRIPTION_LIMIT`] would be exceeded or the scope is missing).
    pub granted: bool,
}

impl Subscription {
    /// Creates a granted subscription record.
    #[must_use]
    pub const fn granted(id: SubscriptionId, topic: Topic) -> Self {
        Self {
            id,
            topic,
            granted: true,
        }
    }

    /// Creates a denied subscription record.
    #[must_use]
    pub const fn denied(id: SubscriptionId, topic: Topic) -> Self {
        Self {
            id,
            topic,
            granted: false,
        }
    }
}

/// The server's response to a [`SubscribeRequest`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SubscribeResponse {
    /// Correlation id of the originating command.
    pub command: CommandId,
    /// The resulting subscription record.
    pub subscription: Subscription,
}

impl SubscribeResponse {
    /// Creates a subscribe response.
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
