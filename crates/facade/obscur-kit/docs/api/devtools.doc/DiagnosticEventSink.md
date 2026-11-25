Republishes the runtime's structured diagnostics (§7.8) as DevTools
[`Topic::Diagnostics`] events (§7.6), so an external subscriber sees a failure
as it happens rather than by polling for it.

Install it on the runtime with
[`set_diagnostic_sink`](crate::runtime::UiRuntime::set_diagnostic_sink) and
hand a [`clone`](Clone::clone) of the same [`EventQueue`] to
[`SocketTransport::serve_with_events`](obscur_devtools_server::transport::SocketTransport::serve_with_events).
The sink states only *what happened*; the serving thread's endpoint still
decides who — if anyone — is subscribed and may hear it, so installing this
grants a client nothing it was not already authorised for.

# What reaches it

Exactly what reaches the runtime's sink: component-render failures and the
§1.7 boundary decision taken for each. Layout, style, renderer and resource
failures still take their own paths and do not arrive here — see
`set_diagnostic_sink`'s own list. The [`DiagnosticContext`] (node, component)
is **not** carried onto the wire: [`EventKind::Diagnostic`] has no field for
it, so a subscriber gets severity, code and message only.

[`DiagnosticContext`]: crate::diagnostics::DiagnosticContext
