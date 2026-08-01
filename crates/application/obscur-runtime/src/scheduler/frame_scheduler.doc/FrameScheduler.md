The event-driven frame scheduler that composes the scheduler primitives into
a single scheduling policy (spec §3.5, §3.6).

The scheduler is **event-driven** and has **no busy loop** (§3.5): callers
push frame requests through [`request`](FrameScheduler::request) as events
arrive, and pull the most-urgent pending frame through
[`next_frame`](FrameScheduler::next_frame) when they are ready to run one.
When nothing is pending, `next_frame` simply returns [`None`] — the scheduler
never spins waiting for work.

It is **pure and deterministic**: it observes no wall-clock, `Instant`,
`SystemTime`, timers, threads or randomness. Time and vsync belong to the
task-port adapter that drives this state machine; the scheduler only reacts
to explicit calls.

The policy is assembled from four primitives:

- a [`SchedulerQueue`] holding the pending, reason-coalesced frame requests,
  drained most-urgent-first by [`SchedulerPriority`](super::SchedulerPriority)
  (§3.6);
- a [`BackpressurePolicy`] deciding how requests are admitted under load
  (§3.5 coalescing and stale-frame skipping);
- a [`StarvationGuard`] providing fairness so background work is not starved
  by a stream of higher-priority frames (§3.6 "starvation prevention for
  background work");
- a `background_pending` flag tracking whether background task work is
  waiting.

## Background work is a separate signal, not a frame

Background shaping / decode / cache work
([`SchedulerPriority::Background`](super::SchedulerPriority::Background),
§3.6 band 5) is **not** a [`FrameReason`] and is **not** in the frame queue:
it is separate task work. Its starvation is therefore modeled as a signal on
the scheduler rather than a queued frame. The runtime tells the scheduler
when background work is waiting
([`note_background_pending`](FrameScheduler::note_background_pending) /
[`note_background_idle`](FrameScheduler::note_background_idle)) and when it
has been serviced ([`service_background`](FrameScheduler::service_background)),
and the scheduler reports when that work has been preempted past the fairness
threshold ([`background_starved`](FrameScheduler::background_starved)).
