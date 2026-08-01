#![doc = include_str!("frame_scheduler.doc/_module.md")]

use super::{BackpressureDecision, BackpressurePolicy, SchedulerQueue, StarvationGuard};
use crate::frame::{FrameReason, FrameRequest};

#[doc = include_str!("frame_scheduler.doc/FrameScheduler.md")]
#[derive(Debug, Clone)]
pub struct FrameScheduler {
    #[doc = include_str!("frame_scheduler.doc/queue.md")]
    queue: SchedulerQueue,
    #[doc = include_str!("frame_scheduler.doc/backpressure.md")]
    backpressure: BackpressurePolicy,
    #[doc = include_str!("frame_scheduler.doc/starvation.md")]
    starvation: StarvationGuard,
    #[doc = include_str!("frame_scheduler.doc/background_pending.md")]
    background_pending: bool,
}

impl FrameScheduler {
    #[doc = include_str!("frame_scheduler.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        Self {
            queue: SchedulerQueue::new(),
            backpressure: BackpressurePolicy::unbounded(),
            starvation: StarvationGuard::default(),
            background_pending: false,
        }
    }

    #[doc = include_str!("frame_scheduler.doc/with_backpressure.md")]
    #[must_use]
    pub const fn with_backpressure(mut self, backpressure: BackpressurePolicy) -> Self {
        self.backpressure = backpressure;
        self
    }

    #[doc = include_str!("frame_scheduler.doc/with_starvation_guard.md")]
    #[must_use]
    pub const fn with_starvation_guard(mut self, starvation: StarvationGuard) -> Self {
        self.starvation = starvation;
        self
    }

    #[doc = include_str!("frame_scheduler.doc/request.md")]
    pub fn request(&mut self, request: FrameRequest) -> BackpressureDecision {
        let already = self.queue.contains(request.reason());
        let decision = self.backpressure.decide(already, self.queue.len());
        match decision {
            BackpressureDecision::Admit | BackpressureDecision::DropStale => {
                self.queue.push(request);
            }
            BackpressureDecision::Coalesce | BackpressureDecision::Reject => {}
        }
        decision
    }

    #[doc = include_str!("frame_scheduler.doc/request_reason.md")]
    pub fn request_reason(&mut self, reason: FrameReason) -> BackpressureDecision {
        self.request(FrameRequest::new(reason))
    }

    #[doc = include_str!("frame_scheduler.doc/next_frame.md")]
    pub fn next_frame(&mut self) -> Option<FrameRequest> {
        let frame = self.queue.pop()?;
        if self.background_pending {
            self.starvation.record_preempted();
        }
        Some(frame)
    }

    #[doc = include_str!("frame_scheduler.doc/coalesce_pending.md")]
    pub fn coalesce_pending(&mut self) {
        self.queue.clear();
    }

    #[doc = include_str!("frame_scheduler.doc/note_background_pending.md")]
    pub fn note_background_pending(&mut self) {
        self.background_pending = true;
    }

    #[doc = include_str!("frame_scheduler.doc/note_background_idle.md")]
    pub fn note_background_idle(&mut self) {
        self.background_pending = false;
    }

    #[doc = include_str!("frame_scheduler.doc/service_background.md")]
    pub fn service_background(&mut self) {
        self.starvation.record_served();
    }

    #[doc = include_str!("frame_scheduler.doc/background_starved.md")]
    #[must_use]
    pub const fn background_starved(&self) -> bool {
        self.background_pending && self.starvation.is_starved()
    }

    #[doc = include_str!("frame_scheduler.doc/pending_count.md")]
    #[must_use]
    pub fn pending_count(&self) -> usize {
        self.queue.len()
    }

    #[doc = include_str!("frame_scheduler.doc/has_pending.md")]
    #[must_use]
    pub fn has_pending(&self) -> bool {
        !self.queue.is_empty()
    }

    #[doc = include_str!("frame_scheduler.doc/peek.md")]
    #[must_use]
    pub fn peek(&self) -> Option<FrameRequest> {
        self.queue.peek()
    }

    #[doc = include_str!("frame_scheduler.doc/is_background_pending.md")]
    #[must_use]
    pub const fn is_background_pending(&self) -> bool {
        self.background_pending
    }

    #[doc = include_str!("frame_scheduler.doc/backpressure_2.md")]
    #[must_use]
    pub const fn backpressure(&self) -> &BackpressurePolicy {
        &self.backpressure
    }

    #[doc = include_str!("frame_scheduler.doc/starvation_2.md")]
    #[must_use]
    pub const fn starvation(&self) -> &StarvationGuard {
        &self.starvation
    }
}

impl Default for FrameScheduler {
    #[doc = include_str!("frame_scheduler.doc/default.md")]
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_admits_and_next_frame_returns_it() {
        let mut scheduler = FrameScheduler::new();
        assert_eq!(
            scheduler.request_reason(FrameReason::Input),
            BackpressureDecision::Admit
        );
        assert_eq!(scheduler.pending_count(), 1);
        assert!(scheduler.has_pending());
        assert_eq!(
            scheduler.next_frame(),
            Some(FrameRequest::new(FrameReason::Input))
        );
        assert!(!scheduler.has_pending());
    }

    #[test]
    fn requesting_same_reason_twice_coalesces() {
        let mut scheduler = FrameScheduler::new();
        assert_eq!(
            scheduler.request_reason(FrameReason::Animation),
            BackpressureDecision::Admit
        );
        assert_eq!(
            scheduler.request_reason(FrameReason::Animation),
            BackpressureDecision::Coalesce
        );
        assert_eq!(scheduler.pending_count(), 1);
    }

    #[test]
    fn next_frame_returns_most_urgent_first() {
        let mut scheduler = FrameScheduler::new();
        scheduler.request_reason(FrameReason::DiagnosticsCapture);
        scheduler.request_reason(FrameReason::Input);
        assert_eq!(
            scheduler.next_frame(),
            Some(FrameRequest::new(FrameReason::Input))
        );
        assert_eq!(
            scheduler.next_frame(),
            Some(FrameRequest::new(FrameReason::DiagnosticsCapture))
        );
    }

    #[test]
    fn bounded_backpressure_rejects_at_capacity() {
        let mut scheduler = FrameScheduler::new()
            .with_backpressure(BackpressurePolicy::bounded(2).with_skip_stale(false));
        assert_eq!(
            scheduler.request_reason(FrameReason::Input),
            BackpressureDecision::Admit
        );
        assert_eq!(
            scheduler.request_reason(FrameReason::UiCommit),
            BackpressureDecision::Admit
        );
        // At capacity with a fresh reason and stale-skipping disabled → rejected.
        assert_eq!(
            scheduler.request_reason(FrameReason::Animation),
            BackpressureDecision::Reject
        );
        assert_eq!(scheduler.pending_count(), 2);
    }

    #[test]
    fn background_work_starves_and_is_cleared_by_service() {
        let threshold = 3;
        let mut scheduler =
            FrameScheduler::new().with_starvation_guard(StarvationGuard::new(threshold));
        scheduler.note_background_pending();
        assert!(!scheduler.background_starved());

        // Serve `threshold` frames while background work waits.
        for _ in 0..threshold {
            scheduler.request_reason(FrameReason::Input);
            assert_eq!(
                scheduler.next_frame(),
                Some(FrameRequest::new(FrameReason::Input))
            );
        }
        assert!(scheduler.background_starved());

        // Running the background work resets fairness and clears starvation.
        scheduler.service_background();
        assert!(!scheduler.background_starved());
    }

    #[test]
    fn next_frame_when_idle_returns_none() {
        let mut scheduler = FrameScheduler::new();
        assert_eq!(scheduler.next_frame(), None);
    }

    #[test]
    fn preemption_does_not_count_without_pending_background() {
        let mut scheduler = FrameScheduler::new().with_starvation_guard(StarvationGuard::new(1));
        scheduler.request_reason(FrameReason::Input);
        assert!(scheduler.next_frame().is_some());
        // No background pending, so no preemption was recorded.
        assert_eq!(scheduler.starvation().waited(), 0);
        assert!(!scheduler.background_starved());
    }

    #[test]
    fn note_background_idle_clears_pending() {
        let mut scheduler = FrameScheduler::new();
        scheduler.note_background_pending();
        assert!(scheduler.is_background_pending());
        scheduler.note_background_idle();
        assert!(!scheduler.is_background_pending());
    }

    #[test]
    fn peek_and_accessors() {
        let mut scheduler = FrameScheduler::new();
        assert_eq!(scheduler.peek(), None);
        scheduler.request_reason(FrameReason::UiCommit);
        assert_eq!(
            scheduler.peek(),
            Some(FrameRequest::new(FrameReason::UiCommit))
        );
        assert_eq!(scheduler.pending_count(), 1);
        assert_eq!(scheduler.backpressure(), &BackpressurePolicy::unbounded());
    }

    #[test]
    fn default_matches_new() {
        let default = FrameScheduler::default();
        assert!(!default.has_pending());
        assert!(!default.is_background_pending());
        assert_eq!(default.backpressure(), &BackpressurePolicy::unbounded());
        assert_eq!(
            default.starvation().threshold(),
            StarvationGuard::DEFAULT_THRESHOLD
        );
    }
}
