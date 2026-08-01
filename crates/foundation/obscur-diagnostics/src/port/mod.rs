#![doc = include_str!("mod.doc/_module.md")]
mod diagnostic_sink;

pub use diagnostic_sink::{DiagnosticSink, NullDiagnosticSink};

mod telemetry_sink;

pub use telemetry_sink::{CollectingTelemetrySink, NullTelemetrySink, TelemetrySink};
