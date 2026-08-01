Returns the priority band for a given [`FrameReason`].

The mapping follows spec §3.6: input dispatch is the most urgent, UI
commits next, then the current frame's work (animation samples plus the
layout and paint driven by window changes and external render requests),
then resource completion, and finally diagnostics captures.

Note that [`SchedulerPriority::Background`] is never returned: no
[`FrameReason`] maps to it (see its variant documentation).
