Enqueues one unit of **background task work** (spec §3.6 band 5:
background shaping, decoding and cache work) for the runtime to service.

Background work is not a frame — it is separate, lower-priority subsystem
work — so it is signalled to the [`FrameScheduler`] rather than pushed as
a [`FrameRequest`]. This records the pending unit and, on the first
outstanding unit, tells the scheduler background work is now waiting
([`FrameScheduler::note_background_pending`]). From then on every
higher-priority frame drained by [`run_due_frame`](Self::run_due_frame)
counts as a preemption of this work; once the scheduler's fairness
threshold is crossed the drain loop promotes and services a unit so the
background band cannot be starved (§3.6 "starvation prevention for
background work").

[`FrameScheduler`]: crate::scheduler::FrameScheduler
[`FrameScheduler::note_background_pending`]: crate::scheduler::FrameScheduler::note_background_pending
[`FrameRequest`]: crate::frame::FrameRequest
