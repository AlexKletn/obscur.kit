Runs **one** frame if the scheduler has a request pending, returning its
[`FrameResult`]. A queued timeline continuation is due after a positive
[`Self::advance_motion`] tick. While waiting, a request for input, a UI commit,
or another independent frame reason can still run a frame.

This is the scheduler-wired frame loop that closes Phase 2: it pulls at
most one most-urgent request via
[`FrameScheduler::next_frame`](crate::scheduler::FrameScheduler::next_frame),
runs the full deterministic core for it (reconcile → pseudo-aware match →
style → layout with retained boxes → scene), and advances the frame
counter. Requests in the same scheduling window share one frame. The
highest-priority reason classifies the frame, and the runtime merges the
phase requirements of the coalesced reasons. When idle it returns [`None`] without side effects — the
runtime never busy-loops (§3.5).
