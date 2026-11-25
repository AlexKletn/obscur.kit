§3.6 (5): background shaping, decoding and cache work.

This band has no corresponding [`FrameReason`]: it covers subsystem work
that is driven by budgeting and starvation prevention rather than by a
frame request, so [`SchedulerPriority::from_reason`] never returns it.
