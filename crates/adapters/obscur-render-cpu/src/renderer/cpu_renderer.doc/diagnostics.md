Where the §5.2 explicit fallback diagnostics go. Defaults to
[`NullDiagnosticSink`], so an effect this backend will not reproduce follows
exactly the same code path whether or not a host is listening. Shared
(`Arc`) because a `CpuRenderer` is [`Clone`] and a clone reports to the same
host sink.
