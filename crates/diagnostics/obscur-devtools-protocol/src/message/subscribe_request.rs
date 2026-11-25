use super::{CommandId, Topic};

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
