The structured-diagnostics destination failures are reported to
(spec §1.6). Defaults to a [`NullDiagnosticSink`], so an unconfigured
runtime builds each diagnostic and drops it — the failure path does not
branch on whether anyone is listening. A host installs a real
[`DiagnosticSink`] via
[`set_diagnostic_sink`](Self::set_diagnostic_sink).

Deliberately not folded into [`telemetry`](Self::telemetry): a
[`BudgetViolation`] goes to the telemetry sink and a component or resource
failure goes here, and no operation touches both. It is handed to the tiers
that report through it (see [`ResourceReporting`]) rather than reached
through the tier that measures.

[`NullDiagnosticSink`]: obscur_diagnostics::NullDiagnosticSink
[`ResourceReporting`]: crate::runtime::ResourceReporting
