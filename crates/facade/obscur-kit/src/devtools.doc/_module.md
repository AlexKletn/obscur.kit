Bridging a live [`UiRuntime`] into the external DevTools endpoint (spec §7.6).

The DevTools server serves inspect requests from an
[`crate::devtools_server::bridge::InspectSource`] the host
provides; [`inspect_source_from`] builds one from a runtime's
[`inspect_snapshot`](crate::runtime::UiRuntime::inspect_snapshot), so an
external agent connecting over the socket sees the *real* tree, layout and
styles instead of a placeholder. The runtime refreshes the source after a
frame and hands it to the endpoint (the host-provides-device pattern), which
keeps the source `Send`-able onto the serving thread without borrowing the
runtime.

# The pushed half

Inspecting is pull; a subscription is push. [`DiagnosticEventSink`] and
[`RuntimeEventPublisher`] are the producers that give a subscription
something to deliver — they turn the runtime's real diagnostics, tree commits
and focus changes into protocol events on an
[`crate::devtools_server::transport::EventQueue`] that
[`SocketTransport::serve_with_events`](obscur_devtools_server::transport::SocketTransport::serve_with_events)
fans out. A host that serves with plain
[`serve`](obscur_devtools_server::transport::SocketTransport::serve) instead
has no such loop, and its endpoint therefore *denies* subscriptions rather
than granting one nothing would service.
