//! `EventQueue`: what a host hands the serving loop to push to subscribers.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use obscur_devtools_protocol::message::{EventKind, MESSAGE_LIMIT, Topic};

/// A bounded, thread-shared queue of events a host wants pushed to subscribed
/// DevTools clients (spec §7.6 live inspectable state, §7.8 buffered +
/// backpressured).
///
/// # Why a queue and not a direct write
///
/// Subscriptions live inside the [`DevToolsEndpoint`](crate::endpoint::DevToolsEndpoint),
/// which the serving thread owns — a host on the frame thread cannot ask "who is
/// subscribed to this?" without sharing the endpoint's whole session state, and
/// with it the authentication and scope checks. So the host states only *what
/// happened*; the serving thread decides *who hears it*, still through the
/// endpoint's own fan-out. Security stays where it was.
///
/// # Bounded on purpose
///
/// [`MESSAGE_LIMIT`] is the same ceiling §7.7 already imposes elsewhere. A host
/// that outruns a slow client **drops the oldest** event rather than growing
/// without bound: an unbounded diagnostics queue turns a stalled inspector into a
/// memory leak in the app being inspected, which is the opposite of a diagnostic.
/// [`dropped`](Self::dropped) reports how many were lost, so a client is told
/// rather than silently shown a gap.
#[derive(Debug, Clone, Default)]
pub struct EventQueue {
    inner: Arc<Mutex<Inner>>,
}

/// The shared state behind an [`EventQueue`].
#[derive(Debug, Default)]
struct Inner {
    /// Pending `(topic, event)` pairs, oldest first.
    pending: VecDeque<(Topic, EventKind)>,
    /// How many events were dropped because the queue was full.
    dropped: u64,
}

impl EventQueue {
    /// Creates an empty queue.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Queues `kind` for subscribers of `topic`.
    ///
    /// Returns whether it was queued; `false` means the queue was full and the
    /// oldest event was dropped to make room.
    #[must_use = "a false return means an event was dropped for backpressure"]
    pub fn push(&self, topic: Topic, kind: EventKind) -> bool {
        let mut inner = self.lock();
        let mut kept = true;
        if inner.pending.len() >= MESSAGE_LIMIT {
            inner.pending.pop_front();
            inner.dropped += 1;
            kept = false;
        }
        inner.pending.push_back((topic, kind));
        kept
    }

    /// Takes every pending event, leaving the queue empty.
    ///
    /// Called by the serving loop, which then fans each out through the endpoint.
    #[must_use]
    pub fn drain(&self) -> Vec<(Topic, EventKind)> {
        let mut inner = self.lock();
        inner.pending.drain(..).collect()
    }

    /// How many events are waiting.
    #[must_use]
    pub fn len(&self) -> usize {
        self.lock().pending.len()
    }

    /// Whether nothing is waiting.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.lock().pending.is_empty()
    }

    /// How many events have been dropped for backpressure since creation.
    #[must_use]
    pub fn dropped(&self) -> u64 {
        self.lock().dropped
    }

    /// Locks the shared state, recovering from a poisoned lock rather than
    /// propagating it: a panicked serving thread must not take the host down with
    /// it — the queue's invariant is just "these are the pending events".
    fn lock(&self) -> std::sync::MutexGuard<'_, Inner> {
        self.inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

#[cfg(test)]
mod tests {
    use super::EventQueue;
    use obscur_devtools_protocol::message::{EventKind, MESSAGE_LIMIT, Topic};

    fn frame(id: u64) -> EventKind {
        EventKind::FrameCommitted { id }
    }

    #[test]
    fn a_pushed_event_drains_once_and_in_order() {
        let queue = EventQueue::new();
        assert!(queue.is_empty());
        assert!(queue.push(Topic::Frames, frame(1)));
        assert!(queue.push(Topic::Frames, frame(2)));
        assert_eq!(queue.len(), 2);

        let drained = queue.drain();
        assert_eq!(drained.len(), 2);
        assert_eq!(drained[0].1, frame(1), "oldest first");
        assert!(queue.is_empty(), "drain leaves nothing behind");
        assert!(queue.drain().is_empty(), "and does not repeat itself");
    }

    #[test]
    fn a_full_queue_drops_the_oldest_rather_than_growing() {
        // §7.7's message limit, and the reason for it: an app being inspected must
        // not leak memory because its inspector stalled.
        let queue = EventQueue::new();
        for id in 0..MESSAGE_LIMIT {
            assert!(queue.push(Topic::Frames, frame(id as u64)), "within the limit");
        }
        assert_eq!(queue.len(), MESSAGE_LIMIT);

        assert!(
            !queue.push(Topic::Frames, frame(9999)),
            "the limit is reported, not silently exceeded"
        );
        assert_eq!(queue.len(), MESSAGE_LIMIT, "bounded");
        assert_eq!(queue.dropped(), 1, "and the loss is counted, not hidden");

        let drained = queue.drain();
        assert_eq!(drained[0].1, frame(1), "the OLDEST was dropped");
        assert_eq!(
            drained[drained.len() - 1].1,
            frame(9999),
            "the newest survived"
        );
    }

    #[test]
    fn a_clone_shares_one_queue() {
        // The host holds one handle and the serving thread another.
        let host = EventQueue::new();
        let server = host.clone();
        assert!(host.push(Topic::Frames, frame(1)));
        assert_eq!(server.len(), 1);
        assert_eq!(server.drain().len(), 1);
        assert!(host.is_empty(), "both handles see the same queue");
    }
}
