//! Subscribe/unsubscribe envelope, subscription record, and limits (§7.7).

/// Maximum number of concurrent subscriptions a single endpoint may hold
/// (spec §7.7 subscription limits).
pub const SUBSCRIPTION_LIMIT: usize = 64;

/// Maximum number of in-flight/queued messages permitted for an endpoint
/// before the server applies backpressure or disconnects (spec §7.7).
pub const MESSAGE_LIMIT: usize = 1024;

#[cfg(test)]
mod tests {
    use super::super::{
        CommandId, SubscribeRequest, SubscribeResponse, Subscription, SubscriptionId, Topic,
    };
    use super::{MESSAGE_LIMIT, SUBSCRIPTION_LIMIT};

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
