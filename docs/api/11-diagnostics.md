# 11 — Diagnostics

Four separate things, with different maturity: structured diagnostics, the
in-window inspector, an external DevTools endpoint, and frame timing.

---

## Structured diagnostics

```rust
rt.set_diagnostic_sink(Box::new(my_sink));
rt.report_diagnostic(&diagnostic);
```

`obscur_kit::diagnostics` holds `Diagnostic`, `DiagnosticCode`,
`DiagnosticSeverity`, `DiagnosticSubsystem`, `RecoveryAction`, `DiagnosticSink`
and `NullDiagnosticSink` (the default — nothing is printed unless a host asks).

The kit reports through this channel: component failures on the frame path,
unresolvable resource URLs, refused slot outlets, budget violations.

> **partial** — several producers construct a `Diagnostic` and drop it into the
> null sink. The render subsystem's §5.2 effect diagnostics are built on both
> render paths and land nowhere unless a host installs a sink. They are
> *reachable*, not *reached*.

## The in-window inspector

```rust
rt.set_inspect_mode(true);
if let Some(info) = rt.hovered_inspect() {
    println!("{}", info);       // type#id.class, component id, absolute box
}
```

`inspect_snapshot() -> TreeInspection` gives the whole tree — one
`NodeInspection` per node in pre-order, with boxes and styles — which is what a
headless agent asserts on.

## External DevTools

A socket endpoint (`obscur_kit::devtools`, `devtools_protocol`,
`devtools_server`) that serves a *live* runtime rather than a placeholder.

```rust
let source   = obscur_kit::devtools::inspect_source_from(&rt);
let listener = SocketListener::bind_loopback()?;      // or bind_unix(path)
let addr     = listener.local_addr()?;                // Some(addr) for TCP, None for a unix socket
let stream   = listener.accept()?;
SocketTransport::new(stream).serve(&mut endpoint)?;
```

Binding is on the **listener**; the transport is what serves one accepted
stream. `SocketTransport::bind_unix` does not exist and never did — it is
`SocketListener::bind_unix`.

| | |
|---|---|
| Listen | `SocketListener::bind_unix(path)` / `bind_loopback()`, then `accept() -> SocketStream` |
| Serve | `SocketTransport::new(stream)`, then `serve` / `serve_one` / `serve_with_events` |
| Connect | `SocketClient::connect_unix(path)` / `connect_loopback(addr)` |
| Pull | `Inspectable` targets, `SnapshotPayload` |
| Push | `EventQueue` + `serve_with_events`; `DiagnosticEventSink` and `RuntimeEventPublisher` turn real diagnostics, tree commits and focus changes into protocol events |
| Security | token authentication, `ScopeSet` / `PermissionScope`, an `audit_log()` |

A host that serves with plain `serve` has no event loop, and its endpoint
therefore **denies** subscriptions rather than granting one nothing would
service. That honesty is deliberate.

Event injection (`InjectedEvent`, `deliver`) is refused when no live
`InjectionSink` is installed — an `Inject` with nowhere to go is an error, not an
acknowledgement.

> **unwired** — no shipped host installs an `InjectionSink`, so injection is
> always refused in practice.

There is deliberately **no arbitrary-code channel**: the protocol cannot be asked
to evaluate anything, and that is enforced by the message enum having no such
variant.

## Frame timing

```sh
OBSCUR_FRAME_STATS=1 cargo run --example calculator --features full
```

Every 60 presented frames: mean build / snapshot / present, implied fps if
saturated, and the worst single frame.

```sh
OBSCUR_NO_VSYNC=1 …    # unpace the swapchain
```

Without this, a frame that is merely *paced* to the refresh rate is
indistinguishable from one that is slow.

> **partial, and this is why the env vars exist.** §7.9's `PerformanceSpan`
> carries a name and a subsystem but **no duration**, and the DevTools protocol
> has no timing field. Neither telemetry nor an attached inspector can answer
> "why is this slow". `set_telemetry_sink` and `set_frame_budget` are real and
> called each frame; what they carry is not enough to profile with.

## Other diagnostic switches

| | |
|---|---|
| `OBSCUR_FULL_RESTYLE=1` | forces the pre-incremental restyle path. If a bug vanishes under it, incremental style invalidation is the cause |
| `OBSCUR_VLIST_DEBUG=1` | prints virtual-list windowing decisions |

## Headless and replay

`HeadlessHost` drives a real `UiRuntime` with no display — the same frame path a
window takes, which is what makes it an oracle rather than a mock.

`obscur_kit::headless::replay` has `EventRecording`, `RecordedEvent`,
`ReplaySession` and `CollectingReplayTarget` for deterministic record/replay.

> **unwired** — the replay tier's only caller is a spec test. No shipped host
> records or replays.

## Accessibility as a diagnostic surface

`semantic_snapshot()` is an immutable, versioned projection of the tree — useful
for assertions even when no AT is attached. Arranged nodes carry window-relative
physical bounds. Those bounds account for scrolling, the camera, and authored
CSS transforms. See
[03 — The runtime](03-runtime.md#accessibility).
