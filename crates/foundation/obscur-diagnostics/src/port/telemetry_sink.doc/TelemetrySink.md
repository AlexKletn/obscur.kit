A destination for performance telemetry. A host or DevTools backend
implements this to receive spans, counters and budget violations as the
engine measures them (spec §7.9).

The trait is object-safe so it can be stored behind `&mut dyn TelemetrySink`.
