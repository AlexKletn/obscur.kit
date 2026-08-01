A self-contained **demo host** the CLI can talk to (spec §7.6/§7.7).

Per §7.6 DevTools is an *external* tool that attaches to a running app's
diagnostics endpoint; that app is normally the process embedding a
`UiRuntime`. To make the interface runnable and testable on its own, this
module stands up the real server stack — an [`Authenticator`], a
[`DevToolsEndpoint`] over a [`SnapshotInspectSource`], and the real
[`SocketTransport`] — seeded with representative live-state so `inspect`
returns non-empty snapshots.

It enforces the same §7.7 security core as any host: the endpoint is enabled
explicitly, every session authenticates a token, and scopes are honoured. A
real deployment would register a [`SnapshotInspectSource`] refreshed from its
live runtime instead of the static demo payloads here.

# Why this host denies subscriptions, and refuses injection

There is no runtime behind these payloads, so nothing here can *produce* an
event — and rather than fabricate one, the demo serves request/reply with
[`SocketTransport::serve`] and lets the endpoint deny every `Subscribe`
honestly. Seeing a live subscription needs a host with something real to
report: `obscur-kit`'s `DiagnosticEventSink`/`RuntimeEventPublisher` feeding
a queue that
[`SocketTransport::serve_with_events`](obscur_devtools_server::transport::SocketTransport::serve_with_events)
fans out, as the windowed examples do.

The same absence decides the *write* direction. There is no runtime to click
on either, so this host installs no
[`InjectionSink`](obscur_devtools_server::transport::InjectionSink) and the
endpoint answers every `Inject` with
[`ProtocolError::InjectionUnavailable`](obscur_devtools_protocol::error::ProtocolError::InjectionUnavailable).
`obscur-devtools inject` used to print `acknowledged command #1` here for a
click that reached nothing at all. A host with a runtime does the opposite:
it holds an
[`InjectionQueue`](obscur_devtools_server::transport::InjectionQueue), hands
[`sink`](obscur_devtools_server::transport::InjectionQueue::sink) to its
endpoint, and applies what it drains with
`obscur_kit::devtools::apply_injected`.
