Rolling per-phase frame timings, printed every [`REPORT_EVERY`] presented
frames when `OBSCUR_FRAME_STATS=1` is set.

This exists because the kit measures nothing itself: §7.9's
[`PerformanceSpan`](crate::diagnostics::performance::PerformanceSpan) carries a
name and a subsystem but **no duration**, and the DevTools protocol has no
timing field, so neither telemetry nor an attached inspector can answer "why is
this slow". Until a span records elapsed time, this is the honest way to profile
a running app.
