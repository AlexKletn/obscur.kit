Installs the performance-telemetry `sink` each frame's phases record into
(spec §7.9), replacing any previously installed sink.

Until a sink is installed the runtime records into a
[`NullTelemetrySink`] — telemetry is *produced* every frame but discarded,
so deep tracing stays opt-in and production overhead is negligible. A host
or DevTools backend installs a real
[`TelemetrySink`](obscur_diagnostics::TelemetrySink) (e.g. a
[`CollectingTelemetrySink`](obscur_diagnostics::CollectingTelemetrySink))
to receive per-phase spans/counters and budget violations as frames run.

[`NullTelemetrySink`]: obscur_diagnostics::NullTelemetrySink
