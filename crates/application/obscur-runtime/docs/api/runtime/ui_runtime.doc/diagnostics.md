The structured-diagnostics destination failures are reported to
(spec §1.6). Defaults to a [`NullDiagnosticSink`], so an unconfigured
runtime builds each diagnostic and drops it — the failure path does not
branch on whether anyone is listening. A host installs a real
[`DiagnosticSink`] via
[`set_diagnostic_sink`](Self::set_diagnostic_sink).

The runtime emits a warning here for each exceeded phase budget. The
telemetry sink also receives the structured budget violation.

[`NullDiagnosticSink`]: obscur_diagnostics::NullDiagnosticSink
