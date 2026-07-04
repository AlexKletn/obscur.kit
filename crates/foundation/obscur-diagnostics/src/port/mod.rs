//! Port module.
mod diagnostic_sink;

pub use diagnostic_sink::DiagnosticSink;

mod telemetry_sink;

pub use telemetry_sink::{CollectingTelemetrySink, NullTelemetrySink, TelemetrySink};
