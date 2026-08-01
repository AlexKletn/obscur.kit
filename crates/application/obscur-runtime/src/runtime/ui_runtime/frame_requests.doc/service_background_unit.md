Services exactly one pending background task-work unit, resetting the
scheduler's fairness count and clearing the pending signal when the last
unit drains (spec §3.6 band 5).

The background unit's effect is deliberately minimal here — it advances
the serviced counter — but it flows through the same scheduler
bookkeeping a real shaping/decode/cache task would: it calls
[`FrameScheduler::service_background`] (resetting fairness so the next
wait starts over) and, when no further units remain,
[`FrameScheduler::note_background_idle`] so subsequent frames stop
counting as preemptions.

[`FrameScheduler::service_background`]: crate::scheduler::FrameScheduler::service_background
[`FrameScheduler::note_background_idle`]: crate::scheduler::FrameScheduler::note_background_idle
