use std::collections::VecDeque;

use obscur_devtools_protocol::message::InjectedEvent;

/// The shared state behind an [`InjectionQueue`](super::InjectionQueue).
#[derive(Debug, Default)]
pub(super) struct InjectionQueueState {
    /// Pending injected events, oldest first.
    pub(super) pending: VecDeque<InjectedEvent>,
    /// How many events were dropped because the queue was full.
    pub(super) dropped: u64,
}
