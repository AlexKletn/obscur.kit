Where spans, counters and violations are recorded.

A [`NullTelemetrySink`] until a host installs one, so telemetry is
*produced* every frame and discarded — the frame path does not branch on
whether anyone is listening, and an unconfigured runtime pays essentially
nothing.
