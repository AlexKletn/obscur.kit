The [`MotionDriver`] driver port shared by every motion source (spec §4.17).

A *driver* is the **source** of a timeline's progression: it turns some
external input — elapsed logical time, an explicit scrubber value, a
drag/cursor/gesture signal, or a scroll offset — into a single normalized
progress in `[0, 1]`. That progress is what feeds a track's `sample(t)`
downstream. Drivers are deliberately narrow (one read of progress plus a
liveness flag) so a heterogeneous set of sources can be held behind
`dyn MotionDriver`.

Like the rest of the motion engine, drivers are **pure and deterministic**
and **time-agnostic**: a driver never reads a clock. Every input is pushed in
explicitly (a [`TimeDriver`](super::TimeDriver) accumulates an explicit
`delta`, a scroll/signal driver is handed its latest value), so a skipped
frame is just a larger input and runs through the same math.
