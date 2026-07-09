//! The pending frame-request queue (spec §3.5 coalescing, §3.6 priority):
//! [`SchedulerQueue`].

use std::collections::BTreeSet;

use super::SchedulerPriority;
use crate::frame::{FrameReason, FrameRequest};

/// Every [`FrameReason`] in a fixed, deterministic order.
///
/// The index of a reason in this array is its tie-break ordinal (see
/// [`reason_ord`]); the array also serves as the reverse lookup in
/// [`reason_from_ord`]. Keeping it in one place ties the two directions
/// together.
const REASONS_IN_ORDER: [FrameReason; 8] = [
    FrameReason::Input,
    FrameReason::UiCommit,
    FrameReason::Animation,
    FrameReason::ResourceCompletion,
    FrameReason::WindowChange,
    FrameReason::ExternalRender,
    FrameReason::DiagnosticsCapture,
    FrameReason::TextEdit,
];

/// Returns the deterministic tie-break ordinal of a reason.
///
/// [`FrameReason`] deliberately does not implement [`Ord`], so the queue derives
/// a stable ordinal here to break ties between reasons that share a priority
/// band. This keeps ordering deterministic without imposing an ordering on the
/// reason type itself.
const fn reason_ord(reason: FrameReason) -> u8 {
    match reason {
        FrameReason::Input => 0,
        FrameReason::UiCommit => 1,
        FrameReason::Animation => 2,
        FrameReason::ResourceCompletion => 3,
        FrameReason::WindowChange => 4,
        FrameReason::ExternalRender => 5,
        FrameReason::DiagnosticsCapture => 6,
        FrameReason::TextEdit => 7,
    }
}

/// Recovers the reason from a tie-break ordinal produced by [`reason_ord`].
fn reason_from_ord(ord: u8) -> FrameReason {
    REASONS_IN_ORDER[ord as usize]
}

/// The queue of pending frame requests, coalesced by reason and ordered by
/// priority (spec §3.5 coalescing, §3.6 priority).
///
/// A [`FrameRequest`] carries only a [`FrameReason`], so coalescing means
/// deduplicating by reason: pushing two `Animation` requests collapses them into
/// a single pending entry (§3.5 "updates are coalesced"). Requests are drained
/// most-urgent-first using [`SchedulerPriority`] (§3.6), with ties broken
/// deterministically by a stable per-reason ordinal.
///
/// The ordering is backed by a [`BTreeSet`] keyed on
/// `(SchedulerPriority, reason ordinal)`, so iteration and popping are fully
/// deterministic and free of wall-clock or randomness.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SchedulerQueue {
    entries: BTreeSet<(SchedulerPriority, u8)>,
}

impl SchedulerQueue {
    /// Creates an empty queue.
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: BTreeSet::new(),
        }
    }

    /// Inserts a request, coalescing by reason.
    ///
    /// Returns `true` if the request was newly added, or `false` if an entry
    /// with the same reason was already pending and this push was coalesced into
    /// it.
    pub fn push(&mut self, request: FrameRequest) -> bool {
        let reason = request.reason();
        let key = (SchedulerPriority::from_reason(reason), reason_ord(reason));
        self.entries.insert(key)
    }

    /// Removes and returns the most-urgent pending request, if any.
    pub fn pop(&mut self) -> Option<FrameRequest> {
        self.entries
            .pop_first()
            .map(|(_, ord)| FrameRequest::new(reason_from_ord(ord)))
    }

    /// Returns the most-urgent pending request without removing it, if any.
    #[must_use]
    pub fn peek(&self) -> Option<FrameRequest> {
        self.entries
            .iter()
            .next()
            .map(|&(_, ord)| FrameRequest::new(reason_from_ord(ord)))
    }

    /// Returns whether a request for `reason` is currently pending.
    #[must_use]
    pub fn contains(&self, reason: FrameReason) -> bool {
        let key = (SchedulerPriority::from_reason(reason), reason_ord(reason));
        self.entries.contains(&key)
    }

    /// Returns the number of pending (coalesced) requests.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns whether the queue is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Removes all pending requests.
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Drains every pending request most-urgent-first, emptying the queue.
    ///
    /// The returned order is deterministic: it matches successive [`pop`]s.
    ///
    /// [`pop`]: SchedulerQueue::pop
    #[must_use = "drained requests are removed from the queue and should be handled"]
    pub fn drain_all(&mut self) -> Vec<FrameRequest> {
        std::mem::take(&mut self.entries)
            .into_iter()
            .map(|(_, ord)| FrameRequest::new(reason_from_ord(ord)))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(reason: FrameReason) -> FrameRequest {
        FrameRequest::new(reason)
    }

    #[test]
    fn push_coalesces_by_reason() {
        let mut queue = SchedulerQueue::new();
        assert!(queue.push(request(FrameReason::Animation)));
        assert!(!queue.push(request(FrameReason::Animation)));
        assert_eq!(queue.len(), 1);
    }

    #[test]
    fn pop_returns_most_urgent_first() {
        let mut queue = SchedulerQueue::new();
        queue.push(request(FrameReason::DiagnosticsCapture));
        queue.push(request(FrameReason::Input));
        assert_eq!(queue.pop(), Some(request(FrameReason::Input)));
        assert_eq!(queue.pop(), Some(request(FrameReason::DiagnosticsCapture)));
        assert_eq!(queue.pop(), None);
    }

    #[test]
    fn peek_does_not_remove() {
        let mut queue = SchedulerQueue::new();
        queue.push(request(FrameReason::UiCommit));
        assert_eq!(queue.peek(), Some(request(FrameReason::UiCommit)));
        assert_eq!(queue.len(), 1);
        assert_eq!(queue.peek(), Some(request(FrameReason::UiCommit)));
    }

    #[test]
    fn drain_all_is_most_urgent_first_and_empties() {
        let mut queue = SchedulerQueue::new();
        queue.push(request(FrameReason::DiagnosticsCapture));
        queue.push(request(FrameReason::ResourceCompletion));
        queue.push(request(FrameReason::Input));
        let drained = queue.drain_all();
        assert_eq!(
            drained,
            vec![
                request(FrameReason::Input),
                request(FrameReason::ResourceCompletion),
                request(FrameReason::DiagnosticsCapture),
            ]
        );
        assert!(queue.is_empty());
    }

    #[test]
    fn contains_and_clear() {
        let mut queue = SchedulerQueue::new();
        assert!(!queue.contains(FrameReason::WindowChange));
        queue.push(request(FrameReason::WindowChange));
        assert!(queue.contains(FrameReason::WindowChange));
        queue.clear();
        assert!(!queue.contains(FrameReason::WindowChange));
        assert!(queue.is_empty());
    }

    #[test]
    fn default_is_empty() {
        let queue = SchedulerQueue::default();
        assert!(queue.is_empty());
        assert_eq!(queue.len(), 0);
    }
}
