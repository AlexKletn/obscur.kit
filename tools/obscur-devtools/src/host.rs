#![doc = include_str!("../docs/api/host.doc/_module.md")]

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

#[doc = include_str!("../docs/api/host.doc/serve_one.md")]
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

#[doc = include_str!("../docs/api/host.doc/serve_forever.md")]
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
