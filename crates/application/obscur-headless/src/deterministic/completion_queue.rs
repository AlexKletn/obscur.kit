//! A queue of controlled async completions (spec §7.4).

use std::collections::VecDeque;

use obscur_identity::OperationId;

use super::completion::Completion;

/// An ordered queue of controlled async completions that a headless test
/// delivers at chosen synchronization points.
///
/// Each entry is a [`Completion<T>`] keyed by an [`OperationId`], so a test can
/// resolve async results deterministically and callers can drop stale results
/// whose operation no longer matches (spec §3.3, §7.4). Completions are
/// delivered in strict FIFO order.
#[derive(Debug, Clone)]
pub struct CompletionQueue<T> {
    /// Pending completions, delivered front-to-back (FIFO).
    entries: VecDeque<Completion<T>>,
}

impl<T> CompletionQueue<T> {
    /// Creates an empty completion queue.
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: VecDeque::new(),
        }
    }

    /// Enqueues a completion pairing `operation` with its result `value`.
    pub fn push(&mut self, operation: OperationId, value: T) {
        self.entries.push_back(Completion::new(operation, value));
    }

    /// Delivers the front (oldest) pending completion, or `None` if empty.
    #[must_use]
    pub fn take_next(&mut self) -> Option<Completion<T>> {
        self.entries.pop_front()
    }

    /// Delivers all pending completions in FIFO order, leaving the queue empty.
    #[must_use]
    pub fn drain(&mut self) -> Vec<Completion<T>> {
        self.entries.drain(..).collect()
    }

    /// Returns the number of pending completions.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns `true` when no completions are pending.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl<T> Default for CompletionQueue<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn op(raw: u64) -> OperationId {
        OperationId::from_raw(raw)
    }

    #[test]
    fn push_and_take_next_are_fifo() {
        let mut queue = CompletionQueue::new();
        assert!(queue.is_empty());

        queue.push(op(1), 10u32);
        queue.push(op(2), 20u32);
        assert_eq!(queue.len(), 2);

        let first = queue.take_next().expect("first completion");
        assert_eq!(first.operation(), op(1));
        assert_eq!(first.into_value(), 10);

        let second = queue.take_next().expect("second completion");
        assert_eq!(second.operation(), op(2));
        assert_eq!(second.into_value(), 20);

        assert!(queue.take_next().is_none());
        assert!(queue.is_empty());
    }

    #[test]
    fn drain_returns_all_in_order() {
        let mut queue = CompletionQueue::new();
        queue.push(op(1), 1u32);
        queue.push(op(2), 2u32);
        queue.push(op(3), 3u32);

        let drained = queue.drain();
        let ops: Vec<u64> = drained.iter().map(|c| c.operation().raw()).collect();
        let values: Vec<u32> = drained.iter().map(|c| *c.value()).collect();

        assert_eq!(ops, vec![1, 2, 3]);
        assert_eq!(values, vec![1, 2, 3]);
        assert!(queue.is_empty());
    }

    #[test]
    fn len_and_is_empty_track_contents() {
        let mut queue: CompletionQueue<u32> = CompletionQueue::default();
        assert_eq!(queue.len(), 0);
        assert!(queue.is_empty());

        queue.push(op(9), 99);
        assert_eq!(queue.len(), 1);
        assert!(!queue.is_empty());
    }
}
