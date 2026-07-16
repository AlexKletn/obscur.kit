//! A self-contained **demo host** the CLI can talk to (spec §7.6/§7.7).
//!
//! Per §7.6 DevTools is an *external* tool that attaches to a running app's
//! diagnostics endpoint; that app is normally the process embedding a
//! `UiRuntime`. To make the interface runnable and testable on its own, this
//! module stands up the real server stack — an [`Authenticator`], a
//! [`DevToolsEndpoint`] over a [`SnapshotInspectSource`], and the real
//! [`SocketTransport`] — seeded with representative live-state so `inspect`
//! returns non-empty snapshots.
//!
//! It enforces the same §7.7 security core as any host: the endpoint is enabled
//! explicitly, every session authenticates a token, and scopes are honoured. A
//! real deployment would register a [`SnapshotInspectSource`] refreshed from its
//! live runtime instead of the static demo payloads here.
//!
//! # Why this host denies subscriptions, and refuses injection
//!
//! There is no runtime behind these payloads, so nothing here can *produce* an
//! event — and rather than fabricate one, the demo serves request/reply with
//! [`SocketTransport::serve`] and lets the endpoint deny every `Subscribe`
//! honestly. Seeing a live subscription needs a host with something real to
//! report: `obscur-kit`'s `DiagnosticEventSink`/`RuntimeEventPublisher` feeding
//! a queue that
//! [`SocketTransport::serve_with_events`](obscur_devtools_server::transport::SocketTransport::serve_with_events)
//! fans out, as the windowed examples do.
//!
//! The same absence decides the *write* direction. There is no runtime to click
//! on either, so this host installs no
//! [`InjectionSink`](obscur_devtools_server::transport::InjectionSink) and the
//! endpoint answers every `Inject` with
//! [`ProtocolError::InjectionUnavailable`](obscur_devtools_protocol::error::ProtocolError::InjectionUnavailable).
//! `obscur-devtools inject` used to print `acknowledged command #1` here for a
//! click that reached nothing at all. A host with a runtime does the opposite:
//! it holds an
//! [`InjectionQueue`](obscur_devtools_server::transport::InjectionQueue), hands
//! [`sink`](obscur_devtools_server::transport::InjectionQueue::sink) to its
//! endpoint, and applies what it drains with
//! `obscur_kit::devtools::apply_injected`.

use std::io;

use obscur_devtools_protocol::message::{Inspectable, ScopeSet, SnapshotPayload};
use obscur_devtools_server::auth::Authenticator;
use obscur_devtools_server::bridge::{RuntimeBridge, SnapshotInspectSource};
use obscur_devtools_server::endpoint::DevToolsEndpoint;
use obscur_devtools_server::transport::{SocketListener, SocketTransport};

/// Builds an inspect source seeded with representative live-state for every
/// target, so a connected client sees non-empty snapshots.
#[must_use]
pub fn demo_source() -> SnapshotInspectSource {
    SnapshotInspectSource::new()
        .with_snapshot(
            Inspectable::Tree,
            SnapshotPayload::new("app > header > gallery(10) > toolbar", 13, vec![0, 1, 2, 3]),
        )
        .with_snapshot(
            Inspectable::Styles,
            SnapshotPayload::new("29 matched rules across 4 origins", 29, vec![]),
        )
        .with_snapshot(
            Inspectable::Layout,
            SnapshotPayload::new("13 boxes laid out (flex + grid)", 13, vec![0, 1, 2]),
        )
        .with_snapshot(
            Inspectable::EventFlow,
            SnapshotPayload::new("no in-flight dispatch", 0, vec![]),
        )
        .with_snapshot(
            Inspectable::Focus,
            SnapshotPayload::new("no focused node", 0, vec![]),
        )
        .with_snapshot(
            Inspectable::Animations,
            SnapshotPayload::new("0 active timelines", 0, vec![]),
        )
        .with_snapshot(
            Inspectable::Resources,
            SnapshotPayload::new("1 image decoded (gallery.png)", 1, vec![100]),
        )
        .with_snapshot(
            Inspectable::Accessibility,
            SnapshotPayload::new("13-node semantic tree", 13, vec![0]),
        )
        .with_snapshot(
            Inspectable::Scene,
            SnapshotPayload::new("77 paint commands", 77, vec![]),
        )
        .with_snapshot(
            Inspectable::Rendering,
            SnapshotPayload::new("CPU backend · 1 frame committed", 1, vec![]),
        )
        .with_snapshot(
            Inspectable::Caches,
            SnapshotPayload::new("style + layout + image caches warm", 3, vec![]),
        )
        .with_snapshot(
            Inspectable::PluginState,
            SnapshotPayload::new("no plugins loaded", 0, vec![]),
        )
}

/// Accepts and serves exactly one client session, then returns.
///
/// A fresh [`DevToolsEndpoint`] is built for the connection (its own session
/// state), authenticating `token` with `scopes`, enabled, and driven over the
/// real socket by [`SocketTransport::serve`] until the client disconnects.
///
/// # Errors
///
/// Returns an [`io::Error`] on any accept/serve socket failure.
pub fn serve_one(
    listener: &SocketListener,
    source: SnapshotInspectSource,
    token: &str,
    scopes: ScopeSet,
) -> io::Result<()> {
    let stream = listener.accept()?;
    let mut authenticator = Authenticator::new();
    authenticator.register_token(token, scopes);
    let mut endpoint = DevToolsEndpoint::new(authenticator, RuntimeBridge::new(source));
    endpoint.enable();
    SocketTransport::new(stream).serve(&mut endpoint)
}

/// Serves client sessions forever (one connection at a time), each over a fresh
/// clone of the demo source.
///
/// # Errors
///
/// Returns an [`io::Error`] on any accept/serve socket failure.
pub fn serve_forever(listener: &SocketListener, token: &str, scopes: ScopeSet) -> io::Result<()> {
    loop {
        serve_one(listener, demo_source(), token, scopes)?;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_devtools_server::bridge::InspectSource;

    #[test]
    fn demo_source_describes_every_target_non_empty() {
        let source = demo_source();
        // Tree/layout/scene carry real counts; all twelve resolve to a described
        // payload (never the "no snapshot registered" fallback).
        assert_eq!(source.snapshot(Inspectable::Scene).entity_count, 77);
        assert_eq!(source.snapshot(Inspectable::Tree).entity_count, 13);
        for (_, target) in crate::cli::TARGETS {
            let payload = source.snapshot(*target);
            assert!(
                !payload.description.contains("no snapshot registered"),
                "target {target:?} was not seeded"
            );
        }
    }
}
