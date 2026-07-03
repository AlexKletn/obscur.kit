//! The event-driven frame scheduler (spec §3.5 event-driven scheduling, §3.6
//! priorities): [`FrameScheduler`].

use super::{BackpressureDecision, BackpressurePolicy, SchedulerQueue, StarvationGuard};
use crate::frame::{FrameReason, FrameRequest};

/// The event-driven frame scheduler that composes the scheduler primitives into
/// a single scheduling policy (spec §3.5, §3.6).
///
/// The scheduler is **event-driven** and has **no busy loop** (§3.5): callers
/// push frame requests through [`request`](FrameScheduler::request) as events
/// arrive, and pull the most-urgent pending frame through
/// [`next_frame`](FrameScheduler::next_frame) when they are ready to run one.
/// When nothing is pending, `next_frame` simply returns [`None`] — the scheduler
/// never spins waiting for work.
///
/// It is **pure and deterministic**: it observes no wall-clock, `Instant`,
/// `SystemTime`, timers, threads or randomness. Time and vsync belong to the
/// task-port adapter that drives this state machine; the scheduler only reacts
/// to explicit calls.
///
/// The policy is assembled from four primitives:
///
/// - a [`SchedulerQueue`] holding the pending, reason-coalesced frame requests,
///   drained most-urgent-first by [`SchedulerPriority`](super::SchedulerPriority)
///   (§3.6);
/// - a [`BackpressurePolicy`] deciding how requests are admitted under load
///   (§3.5 coalescing and stale-frame skipping);
/// - a [`StarvationGuard`] providing fairness so background work is not starved
///   by a stream of higher-priority frames (§3.6 "starvation prevention for
///   background work");
/// - a `background_pending` flag tracking whether background task work is
///   waiting.
///
/// ## Background work is a separate signal, not a frame
///
/// Background shaping / decode / cache work
/// ([`SchedulerPriority::Background`](super::SchedulerPriority::Background),
/// §3.6 band 5) is **not** a [`FrameReason`] and is **not** in the frame queue:
/// it is separate task work. Its starvation is therefore modeled as a signal on
/// the scheduler rather than a queued frame. The runtime tells the scheduler
/// when background work is waiting
/// ([`note_background_pending`](FrameScheduler::note_background_pending) /
/// [`note_background_idle`](FrameScheduler::note_background_idle)) and when it
/// has been serviced ([`service_background`](FrameScheduler::service_background)),
/// and the scheduler reports when that work has been preempted past the fairness
/// threshold ([`background_starved`](FrameScheduler::background_starved)).
#[derive(Debug, Clone)]
pub struct FrameScheduler {
    /// The pending, reason-coalesced frame requests (§3.5, §3.6).
    queue: SchedulerQueue,
    /// The admission-control policy applied to incoming requests (§3.5).
    backpressure: BackpressurePolicy,
    /// The fairness guard preventing background-work starvation (§3.6).
    starvation: StarvationGuard,
    /// Whether background task work is currently waiting to be serviced.
    background_pending: bool,
}

impl FrameScheduler {
    /// Creates a scheduler with an unbounded backpressure policy and the default
    /// starvation guard.
    #[must_use]
    pub fn new() -> Self {
        Self {
            queue: SchedulerQueue::new(),
            backpressure: BackpressurePolicy::unbounded(),
            starvation: StarvationGuard::default(),
            background_pending: false,
        }
    }

    /// Returns this scheduler using `backpressure` as its admission-control
    /// policy.
    ///
    /// Fluent constructor: consumes and returns `self` so it can be chained after
    /// [`new`](FrameScheduler::new).
    #[must_use]
    pub const fn with_backpressure(mut self, backpressure: BackpressurePolicy) -> Self {
        self.backpressure = backpressure;
        self
    }

    /// Returns this scheduler using `starvation` as its fairness guard.
    ///
    /// Fluent constructor: consumes and returns `self` so it can be chained after
    /// [`new`](FrameScheduler::new).
    #[must_use]
    pub const fn with_starvation_guard(mut self, starvation: StarvationGuard) -> Self {
        self.starvation = starvation;
        self
    }

    /// Admits an incoming frame request through the backpressure policy (§3.5).
    ///
    /// This is the event ingress: a request is a fact ("a frame is wanted for
    /// this reason"), and this method decides whether it joins the pending set.
    /// The [`BackpressurePolicy`] is consulted with whether the reason is already
    /// queued and the current queue length, and the resulting
    /// [`BackpressureDecision`] drives what happens:
    ///
    /// - [`Admit`](BackpressureDecision::Admit) or
    ///   [`DropStale`](BackpressureDecision::DropStale) → the request is pushed.
    ///   Under [`DropStale`](BackpressureDecision::DropStale) a stale queued
    ///   frame is allowed to be skipped later (§3.5 "stale intermediate frames
    ///   may be skipped"); because the queue coalesces by reason, a fresher
    ///   request for a reason already supersedes the stale one, so pushing is the
    ///   correct realization of dropping the stale frame.
    /// - [`Coalesce`](BackpressureDecision::Coalesce) → no-op; the reason is
    ///   already pending and the incoming request folds into it (§3.5 "updates
    ///   are coalesced").
    /// - [`Reject`](BackpressureDecision::Reject) → the request is dropped; the
    ///   pending set is full and stale-skipping is disabled.
    ///
    /// Returns the [`BackpressureDecision`] so callers and diagnostics can
    /// observe how the request was handled.
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

    /// Admits a frame request for `reason`, wrapping it in a [`FrameRequest`].
    ///
    /// Convenience over [`request`](FrameScheduler::request) for callers that
    /// only have a [`FrameReason`] in hand.
    pub fn request_reason(&mut self, reason: FrameReason) -> BackpressureDecision {
        self.request(FrameRequest::new(reason))
    }

    /// Pops the most-urgent pending frame to run, or [`None`] if idle (§3.6).
    ///
    /// The frame is drawn from the queue in priority order. If a frame is
    /// returned while background work is pending, that frame has preempted the
    /// waiting background work, so the starvation guard records a preemption
    /// ([`StarvationGuard::record_preempted`]) — moving the background work one
    /// step closer to being reported as starved
    /// ([`background_starved`](FrameScheduler::background_starved)).
    ///
    /// When the queue is empty this returns [`None`] without side effects: the
    /// scheduler is event-driven and does not busy-loop (§3.5).
    pub fn next_frame(&mut self) -> Option<FrameRequest> {
        let frame = self.queue.pop()?;
        if self.background_pending {
            self.starvation.record_preempted();
        }
        Some(frame)
    }

    /// Records that background task work is now waiting to be serviced.
    ///
    /// The runtime calls this when subsystem shaping / decode / cache work
    /// (§3.6 band 5) has become pending; while set, frames served by
    /// [`next_frame`](FrameScheduler::next_frame) count as preemptions of that
    /// work.
    pub fn note_background_pending(&mut self) {
        self.background_pending = true;
    }

    /// Records that no background task work is waiting.
    ///
    /// Clears the pending flag set by
    /// [`note_background_pending`](FrameScheduler::note_background_pending);
    /// subsequent frames no longer count as preemptions.
    pub fn note_background_idle(&mut self) {
        self.background_pending = false;
    }

    /// Records that the pending background work was run, resetting fairness.
    ///
    /// Calling this resets the starvation guard
    /// ([`StarvationGuard::record_served`]) so the fairness count starts over.
    /// The runtime **should** call this when
    /// [`background_starved`](FrameScheduler::background_starved) returns `true`,
    /// servicing the background work next to honor §3.6 starvation prevention.
    pub fn service_background(&mut self) {
        self.starvation.record_served();
    }

    /// Returns whether pending background work has been starved and must be
    /// serviced next (§3.6).
    ///
    /// This is `true` only when background work is pending **and** the starvation
    /// guard reports it as starved — that is, higher-priority frames have
    /// preempted it past the fairness threshold. When this returns `true` the
    /// runtime should run the background work and call
    /// [`service_background`](FrameScheduler::service_background).
    #[must_use]
    pub const fn background_starved(&self) -> bool {
        self.background_pending && self.starvation.is_starved()
    }

    /// Returns the number of pending (coalesced) frame requests.
    #[must_use]
    pub fn pending_count(&self) -> usize {
        self.queue.len()
    }

    /// Returns whether any frame request is pending.
    #[must_use]
    pub fn has_pending(&self) -> bool {
        !self.queue.is_empty()
    }

    /// Returns the most-urgent pending frame without removing it, if any.
    #[must_use]
    pub fn peek(&self) -> Option<FrameRequest> {
        self.queue.peek()
    }

    /// Returns whether background task work is currently marked pending.
    #[must_use]
    pub const fn is_background_pending(&self) -> bool {
        self.background_pending
    }

    /// Returns the backpressure policy this scheduler admits requests with.
    #[must_use]
    pub const fn backpressure(&self) -> &BackpressurePolicy {
        &self.backpressure
    }

    /// Returns the starvation guard tracking background-work fairness.
    #[must_use]
    pub const fn starvation(&self) -> &StarvationGuard {
        &self.starvation
    }
}

impl Default for FrameScheduler {
    /// The default scheduler is [`FrameScheduler::new`].
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
        let mut scheduler =
            FrameScheduler::new().with_starvation_guard(StarvationGuard::new(1));
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
        assert_eq!(scheduler.peek(), Some(FrameRequest::new(FrameReason::UiCommit)));
        assert_eq!(scheduler.pending_count(), 1);
        assert_eq!(scheduler.backpressure(), &BackpressurePolicy::unbounded());
    }

    #[test]
    fn default_matches_new() {
        let default = FrameScheduler::default();
        assert!(!default.has_pending());
        assert!(!default.is_background_pending());
        assert_eq!(default.backpressure(), &BackpressurePolicy::unbounded());
        assert_eq!(default.starvation().threshold(), StarvationGuard::DEFAULT_THRESHOLD);
    }
}
