Per-frame pipeline-phase classification: [`FramePlan`].

Spec §3.4 phase 5 ("classify invalidation") requires the frame loop to decide
*which* pipeline phases a frame actually needs instead of re-running
reconcile → match → style → layout → compose unconditionally. `FramePlan` is
that decision: it maps the frame's [`FrameReason`] through the live
[`InvalidationRouter`] to the set of phases the runtime must run this frame.
