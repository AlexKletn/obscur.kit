use std::collections::VecDeque;

use obscur_devtools_protocol::message::{EventKind, Topic};

/// The shared state behind an [`EventQueue`](super::EventQueue).
#[derive(Debug, Default)]
pub(super) struct EventQueueState {
    /// Pending `(topic, event)` pairs, oldest first.
    pub(super) pending: VecDeque<(Topic, EventKind)>,
    /// How many events were dropped because the queue was full.
    pub(super) dropped: u64,
}
