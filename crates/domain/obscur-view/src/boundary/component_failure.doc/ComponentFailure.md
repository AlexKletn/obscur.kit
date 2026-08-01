A structured record of a failure the runtime detected inside a component
(spec §1.7).

This is *not* panic-catching: native Rust is trusted (spec §1.8), so there
is no `catch_unwind` here. It is the honest, structured value the runtime
produces when it detects a contained failure in view generation,
reconciliation, or a local lifecycle hook — carrying the
[`LifecycleState`] the failure happened in, the retry attempt that produced
it, and a [`Diagnostic`] (subsystem [`DiagnosticSubsystem::View`]) describing
it for diagnostics and `DevTools`.
