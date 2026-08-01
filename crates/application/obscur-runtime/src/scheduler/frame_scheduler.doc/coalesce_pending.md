Coalesces every still-pending frame request into the frame just pulled by
[`next_frame`](FrameScheduler::next_frame), clearing them from the queue
(§3.4 step 2 "choose frame reasons" — plural; §3.5 "updates are
coalesced").

A single frame serves the whole scheduling window: input by its dispatch
phase, a commit by its staged plan, a landed resource by the completion
drain. Those reasons are therefore discarded rather than left to schedule
further frames of their own — and without counting as further preemptions
of background work, because only one frame ran and
[`next_frame`](FrameScheduler::next_frame) already recorded its single
preemption.
