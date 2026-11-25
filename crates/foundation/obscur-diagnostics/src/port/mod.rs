//! Port module.
mod diagnostic_sink;

pub use diagnostic_sink::DiagnosticSink;
mod null_diagnostic_sink;
pub use null_diagnostic_sink::NullDiagnosticSink;

mod telemetry_sink;

pub use telemetry_sink::TelemetrySink;
mod null_telemetry_sink;
pub use null_telemetry_sink::NullTelemetrySink;
mod collecting_telemetry_sink;
pub use collecting_telemetry_sink::CollectingTelemetrySink;
