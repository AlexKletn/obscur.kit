Requests a frame for `reason`, returning the scheduler's admission
[`BackpressureDecision`] (§3.5).

This is the event ingress: a host calls it as input, animation ticks,
resource completions, resizes, etc. arrive. The request is coalesced and
prioritised by the [`FrameScheduler`]; nothing runs until
[`run_due_frame`](Self::run_due_frame) is called.

[`FrameScheduler`]: crate::scheduler::FrameScheduler
