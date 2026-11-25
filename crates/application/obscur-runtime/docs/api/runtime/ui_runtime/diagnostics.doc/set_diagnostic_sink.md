Installs the structured-diagnostics `sink` the runtime reports failures
to (spec §1.6, §7.8), replacing any previously installed sink.

Until one is installed the runtime emits into a [`NullDiagnosticSink`] —
every diagnostic is *built* and offered exactly as it would be to a real
sink, then dropped, so listening changes what is observed and never what
happens. A host that wants them subscribes a
[`ChannelSink`](obscur_diagnostics::ChannelSink) here, which is also the
path a `DevTools` backend takes.

# What reaches it

Component-render failures and the §1.7 boundary decision taken for each;
resource load/decode failures with the resource's identity attached;
frame-budget warnings; and
CSS parse/compile diagnostics, which a host routes here through
[`report_diagnostic`](Self::report_diagnostic) after compiling (the
runtime never sees a sheet's source, only its compiled rules, so it cannot
raise them itself).

[`NullDiagnosticSink`]: obscur_diagnostics::NullDiagnosticSink
