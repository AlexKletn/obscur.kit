Per-phase frame timing for profiling, active only when `OBSCUR_FRAME_STATS=1`.

The kit measures nothing itself — §7.9's `PerformanceSpan` names a phase but
carries no duration — so this is what answers "which phase is the frame's cost".
It aggregates over [`PHASE_REPORT_EVERY`] frames and prints one line, mirroring
the facade runner's `[frame-stats]`.
