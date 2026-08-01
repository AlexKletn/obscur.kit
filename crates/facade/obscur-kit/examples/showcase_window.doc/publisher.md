Turns the runtime's real frame-commit and focus transitions into events on
`events`. Diagnostics take a separate path: a `DiagnosticEventSink`
installed on the runtime feeds the same queue.
