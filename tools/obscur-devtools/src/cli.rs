#![doc = include_str!("cli.doc/_module.md")]

use std::io;

use obscur_devtools_protocol::message::{
    CommandId, CommandKind, CommandMessage, HandshakeRequest, HandshakeResponse, InjectedEvent,
    Inspectable, PermissionScope, ProtocolVersion, ScopeSet, SnapshotMessage, Topic, VersionRange,
};
use obscur_devtools_server::transport::{ClientBound, SocketClient};

#[doc = include_str!("cli.doc/TARGETS.md")]
pub const TARGETS: &[(&str, Inspectable)] = &[
    ("tree", Inspectable::Tree),
    ("styles", Inspectable::Styles),
    ("layout", Inspectable::Layout),
    ("event-flow", Inspectable::EventFlow),
    ("focus", Inspectable::Focus),
    ("animations", Inspectable::Animations),
    ("resources", Inspectable::Resources),
    ("accessibility", Inspectable::Accessibility),
    ("scene", Inspectable::Scene),
    ("rendering", Inspectable::Rendering),
    ("caches", Inspectable::Caches),
    ("plugin-state", Inspectable::PluginState),
];

#[doc = include_str!("cli.doc/TOPICS.md")]
pub const TOPICS: &[(&str, Topic)] = &[
    ("diagnostics", Topic::Diagnostics),
    ("lifecycle", Topic::Lifecycle),
    ("frames", Topic::Frames),
    ("focus", Topic::Focus),
];

#[doc = include_str!("cli.doc/parse_target.md")]
#[must_use]
pub fn parse_target(name: &str) -> Option<Inspectable> {
    TARGETS
        .iter()
        .find(|(label, _)| *label == name)
        .map(|(_, target)| *target)
}

#[doc = include_str!("cli.doc/all_targets.md")]
#[must_use]
pub fn all_targets() -> Vec<Inspectable> {
    TARGETS.iter().map(|(_, target)| *target).collect()
}

#[doc = include_str!("cli.doc/target_label.md")]
#[must_use]
pub fn target_label(target: Inspectable) -> &'static str {
    TARGETS
        .iter()
        .find(|(_, candidate)| *candidate == target)
        .map_or("unknown", |(label, _)| label)
}

#[doc = include_str!("cli.doc/parse_topic.md")]
#[must_use]
pub fn parse_topic(name: &str) -> Option<Topic> {
    TOPICS
        .iter()
        .find(|(label, _)| *label == name)
        .map(|(_, topic)| *topic)
}

#[doc = include_str!("cli.doc/handshake.md")]
pub fn handshake(client: &mut SocketClient, token: &str) -> io::Result<HandshakeResponse> {
    let request = HandshakeRequest::new(VersionRange::exact(ProtocolVersion::CURRENT), token);
    client.handshake(&request)
}

#[doc = include_str!("cli.doc/inspect.md")]
pub fn inspect(client: &mut SocketClient, id: u64, target: Inspectable) -> io::Result<ClientBound> {
    client.command(&CommandMessage::new(
        CommandId::new(id),
        CommandKind::Inspect(target),
    ))
}

#[doc = include_str!("cli.doc/subscribe.md")]
pub fn subscribe(client: &mut SocketClient, id: u64, topic: Topic) -> io::Result<ClientBound> {
    client.command(&CommandMessage::new(
        CommandId::new(id),
        CommandKind::Subscribe(topic),
    ))
}

#[doc = include_str!("cli.doc/inject.md")]
pub fn inject(client: &mut SocketClient, id: u64, event: InjectedEvent) -> io::Result<ClientBound> {
    client.command(&CommandMessage::new(
        CommandId::new(id),
        CommandKind::Inject(event),
    ))
}

#[doc = include_str!("cli.doc/format_handshake.md")]
#[must_use]
pub fn format_handshake(response: &HandshakeResponse) -> String {
    match response {
        HandshakeResponse::Accepted { version, granted } => format!(
            "connected · protocol v{}.{} · scopes: {}",
            version.major,
            version.minor,
            format_scopes(*granted)
        ),
        HandshakeResponse::Rejected { reason } => format!("rejected: {reason:?}"),
    }
}

#[doc = include_str!("cli.doc/format_scopes.md")]
#[must_use]
pub fn format_scopes(scopes: ScopeSet) -> String {
    let mut names = Vec::new();
    for (label, scope) in [
        ("read", PermissionScope::Read),
        ("subscribe", PermissionScope::Subscribe),
        ("control", PermissionScope::Control),
        ("inject", PermissionScope::Inject),
    ] {
        if scopes.contains(scope) {
            names.push(label);
        }
    }
    if names.is_empty() {
        "none".to_owned()
    } else {
        names.join(", ")
    }
}

#[doc = include_str!("cli.doc/format_snapshot.md")]
#[must_use]
pub fn format_snapshot(message: &SnapshotMessage) -> String {
    let ids = if message.payload.entity_ids.is_empty() {
        String::new()
    } else {
        format!(" [{}]", join_ids(&message.payload.entity_ids))
    };
    let mut line = format!(
        "{:<14} {:>4} entit{}  {}{ids}",
        target_label(message.target),
        message.payload.entity_count,
        if message.payload.entity_count == 1 {
            "y "
        } else {
            "ies"
        },
        message.payload.description,
    );
    // A rich host attaches a full structured rendering (e.g. the whole tree with
    // per-node geometry + styles); print it indented under the summary line.
    if !message.payload.detail.is_empty() {
        for detail_line in message.payload.detail.lines() {
            line.push_str("\n    ");
            line.push_str(detail_line);
        }
    }
    line
}

#[doc = include_str!("cli.doc/format_client_bound.md")]
#[must_use]
pub fn format_client_bound(reply: &ClientBound) -> String {
    match reply {
        ClientBound::Handshake(response) => format_handshake(response),
        ClientBound::Snapshot(snapshot) => format_snapshot(snapshot),
        ClientBound::Subscription(subscription) => format!(
            "subscription #{} to {:?} · granted={}",
            subscription.id.0, subscription.topic, subscription.granted
        ),
        ClientBound::Event(event) => format!("event on {:?}: {:?}", event.topic, event.kind),
        ClientBound::Acknowledged(id) => format!("acknowledged command #{}", id.raw()),
        ClientBound::Error(error) => format!("error: {error:?}"),
    }
}

#[doc = include_str!("cli.doc/join_ids.md")]
fn join_ids(ids: &[u64]) -> String {
    const MAX: usize = 8;
    let shown: Vec<String> = ids.iter().take(MAX).map(u64::to_string).collect();
    if ids.len() > MAX {
        format!("{}, +{} more", shown.join(", "), ids.len() - MAX)
    } else {
        shown.join(", ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_devtools_protocol::message::SnapshotPayload;

    #[test]
    fn every_target_round_trips_its_name() {
        for (label, target) in TARGETS {
            assert_eq!(parse_target(label), Some(*target));
            assert_eq!(target_label(*target), *label);
        }
        assert_eq!(all_targets().len(), 12);
    }

    #[test]
    fn unknown_target_and_topic_are_none() {
        assert_eq!(parse_target("bogus"), None);
        assert_eq!(parse_topic("bogus"), None);
    }

    #[test]
    fn topics_round_trip() {
        assert_eq!(parse_topic("frames"), Some(Topic::Frames));
        assert_eq!(parse_topic("diagnostics"), Some(Topic::Diagnostics));
    }

    #[test]
    fn scopes_format_lists_granted_names() {
        assert_eq!(format_scopes(ScopeSet::empty()), "none");
        assert_eq!(format_scopes(ScopeSet::all_read()), "read");
        let all = format_scopes(ScopeSet::all());
        assert!(all.contains("read") && all.contains("control") && all.contains("inject"));
    }

    #[test]
    fn snapshot_format_includes_target_count_and_ids() {
        let msg = SnapshotMessage::new(
            CommandId::new(1),
            Inspectable::Tree,
            SnapshotPayload::new("root + 4 nodes", 5, vec![0, 1, 2, 3, 4]),
        );
        let line = format_snapshot(&msg);
        assert!(line.contains("tree"), "{line}");
        assert!(line.contains('5'), "{line}");
        assert!(line.contains("root + 4 nodes"), "{line}");
        assert!(line.contains("[0, 1, 2, 3, 4]"), "{line}");
    }

    #[test]
    fn handshake_accepted_line_reports_version_and_scopes() {
        let line = format_handshake(&HandshakeResponse::accepted(
            ProtocolVersion::CURRENT,
            ScopeSet::observer(),
        ));
        assert!(line.contains("v1.0"), "{line}");
        assert!(line.contains("read"), "{line}");
    }
}
