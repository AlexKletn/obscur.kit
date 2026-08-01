Runs **one** frame if the scheduler has a request pending, returning its
[`FrameResult`]; returns [`None`] when nothing is due.

This is the scheduler-wired frame loop that closes Phase 2: it pulls at
most one most-urgent request via
[`FrameScheduler::next_frame`](crate::scheduler::FrameScheduler::next_frame),
runs the full deterministic core for it (reconcile → pseudo-aware match →
style → layout with retained boxes → scene), and advances the frame
counter. Because requests coalesce, a burst of same-reason events produces
exactly one frame. When idle it returns [`None`] without side effects — the
runtime never busy-loops (§3.5).
