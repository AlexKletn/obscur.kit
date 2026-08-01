A [`DiagnosticSink`] collecting into a shared buffer: `set_diagnostic_sink`
takes the sink by value, so a test that wants to read it needs a handle.
