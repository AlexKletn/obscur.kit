Force-renders one frame **ignoring the scheduler**, returning its
[`FrameResult`].

This is the unconditional path for first paint and tests: it runs the same
deterministic core as [`run_due_frame`](Self::run_due_frame) with reason
[`FrameReason::UiCommit`], regardless of whether any request is pending.
Two `render_now` calls over the same view + sheets + pseudo-state yield
identical display lists.
