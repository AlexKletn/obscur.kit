use super::{SubscriptionId, Topic};

/// A subscription record returned to (and retained by) the client.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Subscription {
    /// Server-assigned subscription id.
    pub id: SubscriptionId,
    /// The topic this subscription delivers.
    pub topic: Topic,
    #[doc = include_str!("../../docs/api/message/subscription_message.doc/granted.md")]
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

    #[doc = include_str!("../../docs/api/message/subscription_message.doc/denied.md")]
    #[must_use]
    pub const fn denied(id: SubscriptionId, topic: Topic) -> Self {
        Self {
            id,
            topic,
            granted: false,
        }
    }
}
