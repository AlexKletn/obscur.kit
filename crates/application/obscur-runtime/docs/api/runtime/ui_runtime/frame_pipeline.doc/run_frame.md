Runs the deterministic core over the runtime-owned state, retaining the
laid-out boxes, computed styles and cascade inputs, and advancing the
frame counter.

Unlike [`FramePipeline::run_matched`](crate::frame::FramePipeline::run_matched),
which re-runs reconcile → match → style → layout → compose every call,
this classifies the frame's [`FrameReason`] into a [`FramePlan`] (spec
§3.4 phase 5) and **skips phases whose inputs are unchanged**, reusing the
previous frame's retained output. It still (a) sources cascade inputs
through the **pseudo-state-aware** matcher and (b) *keeps* the layout
boxes for hit-testing.

The classification is conservative: the first frame (empty tree) and any
structural reason run every phase, and a phase that cannot be proven
unnecessary is run. The concrete wins are that a hover/focus (`Input`) or
motion (`Animation`) frame does **not** re-reconcile, and an idle
diagnostics capture recalculates nothing and simply re-presents the
retained scene.

[`FramePlan`]: crate::runtime::FramePlan
