use super::{CommandId, Subscription};

/// The server's response to a [`SubscribeRequest`](super::SubscribeRequest).
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
