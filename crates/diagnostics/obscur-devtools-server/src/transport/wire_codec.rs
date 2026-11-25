#![doc = include_str!("../../docs/api/transport/wire_codec.doc/_module.md")]

mod wire_cursor;

use wire_cursor::WireCursor;

use obscur_devtools_protocol::error::ProtocolError;
use obscur_devtools_protocol::message::{
    CommandId, CommandKind, CommandMessage, ControlAction, EventKind, EventSeverity,
    HandshakeRejection, HandshakeRequest, HandshakeResponse, InjectedEvent, Inspectable,
    PermissionScope, ProtocolEvent, ProtocolVersion, ScopeSet, SnapshotMessage, SnapshotPayload,
    Subscription, SubscriptionId, Topic, VersionRange,
};

use crate::transport::{ClientBound, ServerBound};

use super::WireError;

#[doc = include_str!("../../docs/api/transport/wire_codec.doc/WireCodec.md")]
#[derive(Debug, Default, Clone, Copy)]
pub struct WireCodec;

impl WireCodec {
    /// Encodes a client→server envelope into a fresh byte buffer.
    #[must_use]
    pub fn encode_server_bound(message: &ServerBound) -> Vec<u8> {
        let mut buf = Vec::new();
        match message {
            ServerBound::Handshake(request) => {
                push_u8(&mut buf, 0);
                push_handshake_request(&mut buf, request);
            }
            ServerBound::Command(command) => {
                push_u8(&mut buf, 1);
                push_command(&mut buf, command);
            }
        }
        buf
    }

    #[doc = include_str!("../../docs/api/transport/wire_codec.doc/decode_server_bound.md")]
    pub fn decode_server_bound(bytes: &[u8]) -> Result<ServerBound, WireError> {
        let mut cursor = WireCursor::new(bytes);
        let out = match cursor.read_u8()? {
            0 => ServerBound::Handshake(read_handshake_request(&mut cursor)?),
            1 => ServerBound::Command(read_command(&mut cursor)?),
            other => return Err(WireError::UnknownTag(other)),
        };
        Ok(out)
    }

    /// Encodes a server→client envelope into a fresh byte buffer.
    #[must_use]
    pub fn encode_client_bound(message: &ClientBound) -> Vec<u8> {
        let mut buf = Vec::new();
        match message {
            ClientBound::Handshake(response) => {
                push_u8(&mut buf, 0);
                push_handshake_response(&mut buf, response);
            }
            ClientBound::Snapshot(snapshot) => {
                push_u8(&mut buf, 1);
                push_snapshot(&mut buf, snapshot);
            }
            ClientBound::Event(event) => {
                push_u8(&mut buf, 2);
                push_event(&mut buf, event);
            }
            ClientBound::Subscription(subscription) => {
                push_u8(&mut buf, 3);
                push_subscription(&mut buf, subscription);
            }
            ClientBound::Acknowledged(id) => {
                push_u8(&mut buf, 4);
                push_u64(&mut buf, id.raw());
            }
            ClientBound::Error(error) => {
                push_u8(&mut buf, 5);
                push_protocol_error(&mut buf, error);
            }
        }
        buf
    }

    #[doc = include_str!("../../docs/api/transport/wire_codec.doc/decode_client_bound.md")]
    pub fn decode_client_bound(bytes: &[u8]) -> Result<ClientBound, WireError> {
        let mut cursor = WireCursor::new(bytes);
        let out = match cursor.read_u8()? {
            0 => ClientBound::Handshake(read_handshake_response(&mut cursor)?),
            1 => ClientBound::Snapshot(read_snapshot(&mut cursor)?),
            2 => ClientBound::Event(read_event(&mut cursor)?),
            3 => ClientBound::Subscription(read_subscription(&mut cursor)?),
            4 => ClientBound::Acknowledged(CommandId::new(cursor.read_u64()?)),
            5 => ClientBound::Error(read_protocol_error(&mut cursor)?),
            other => return Err(WireError::UnknownTag(other)),
        };
        Ok(out)
    }
}

fn push_handshake_request(buf: &mut Vec<u8>, request: &HandshakeRequest) {
    push_version_range(buf, request.client_versions);
    push_str(buf, &request.token);
}

fn read_handshake_request(cursor: &mut WireCursor<'_>) -> Result<HandshakeRequest, WireError> {
    let versions = read_version_range(cursor)?;
    let token = cursor.read_str()?.to_owned();
    Ok(HandshakeRequest::new(versions, token))
}

fn push_handshake_response(buf: &mut Vec<u8>, response: &HandshakeResponse) {
    match response {
        HandshakeResponse::Accepted { version, granted } => {
            push_u8(buf, 0);
            push_version(buf, *version);
            push_scope_set(buf, *granted);
        }
        HandshakeResponse::Rejected { reason } => {
            push_u8(buf, 1);
            push_u8(buf, rejection_tag(*reason));
        }
    }
}

fn read_handshake_response(cursor: &mut WireCursor<'_>) -> Result<HandshakeResponse, WireError> {
    match cursor.read_u8()? {
        0 => {
            let version = read_version(cursor)?;
            let granted = read_scope_set(cursor)?;
            Ok(HandshakeResponse::accepted(version, granted))
        }
        1 => Ok(HandshakeResponse::rejected(read_rejection(cursor)?)),
        other => Err(WireError::UnknownTag(other)),
    }
}

const fn rejection_tag(reason: HandshakeRejection) -> u8 {
    match reason {
        HandshakeRejection::VersionUnsupported => 0,
        HandshakeRejection::Unauthorized => 1,
    }
}

fn read_rejection(cursor: &mut WireCursor<'_>) -> Result<HandshakeRejection, WireError> {
    match cursor.read_u8()? {
        0 => Ok(HandshakeRejection::VersionUnsupported),
        1 => Ok(HandshakeRejection::Unauthorized),
        other => Err(WireError::UnknownTag(other)),
    }
}

fn push_command(buf: &mut Vec<u8>, command: &CommandMessage) {
    use obscur_devtools_protocol::message::StableSelector;
    push_u64(buf, command.id.raw());
    match &command.kind {
        CommandKind::Inspect(target) => {
            push_u8(buf, 0);
            push_u8(buf, inspectable_tag(*target));
        }
        CommandKind::Subscribe(topic) => {
            push_u8(buf, 1);
            push_u8(buf, topic_tag(*topic));
        }
        CommandKind::Unsubscribe(id) => {
            push_u8(buf, 2);
            push_u64(buf, id.raw());
        }
        CommandKind::Control(action) => {
            push_u8(buf, 3);
            push_control_action(buf, *action);
        }
        CommandKind::Inject(event) => {
            push_u8(buf, 4);
            push_injected_event(buf, *event);
        }
        CommandKind::SelectSemantic { role, name } => {
            push_u8(buf, 5);
            push_str(buf, role);
            push_u8(buf, u8::from(name.is_some()));
            if let Some(name) = name {
                push_str(buf, name);
            }
        }
        CommandKind::SelectStableId(selector) => {
            push_u8(buf, 6);
            match selector {
                StableSelector::TestId(id) => {
                    push_u8(buf, 0);
                    push_str(buf, id);
                }
                StableSelector::AutomationId(id) => {
                    push_u8(buf, 1);
                    push_str(buf, id);
                }
                StableSelector::Component(id) => {
                    push_u8(buf, 2);
                    push_u64(buf, *id);
                }
                StableSelector::Node(id) => {
                    push_u8(buf, 3);
                    push_u64(buf, *id);
                }
            }
        }
    }
}

fn read_command(cursor: &mut WireCursor<'_>) -> Result<CommandMessage, WireError> {
    use obscur_devtools_protocol::message::StableSelector;
    let id = CommandId::new(cursor.read_u64()?);
    let kind = match cursor.read_u8()? {
        0 => CommandKind::Inspect(read_inspectable(cursor)?),
        1 => CommandKind::Subscribe(read_topic(cursor)?),
        2 => CommandKind::Unsubscribe(SubscriptionId::new(cursor.read_u64()?)),
        3 => CommandKind::Control(read_control_action(cursor)?),
        4 => CommandKind::Inject(read_injected_event(cursor)?),
        5 => {
            let role = cursor.read_str()?.into();
            let name = match cursor.read_u8()? {
                0 => None,
                1 => Some(cursor.read_str()?.into()),
                other => return Err(WireError::UnknownTag(other)),
            };
            CommandKind::SelectSemantic { role, name }
        }
        6 => CommandKind::SelectStableId(match cursor.read_u8()? {
            0 => StableSelector::TestId(cursor.read_str()?.into()),
            1 => StableSelector::AutomationId(cursor.read_str()?.into()),
            2 => StableSelector::Component(cursor.read_u64()?),
            3 => StableSelector::Node(cursor.read_u64()?),
            other => return Err(WireError::UnknownTag(other)),
        }),
        other => return Err(WireError::UnknownTag(other)),
    };
    Ok(CommandMessage::new(id, kind))
}

fn push_control_action(buf: &mut Vec<u8>, action: ControlAction) {
    match action {
        ControlAction::Pause => push_u8(buf, 0),
        ControlAction::Resume => push_u8(buf, 1),
        ControlAction::StepFrame => push_u8(buf, 2),
        ControlAction::SetInspectMode(on) => {
            push_u8(buf, 3);
            push_u8(buf, u8::from(on));
        }
    }
}

fn read_control_action(cursor: &mut WireCursor<'_>) -> Result<ControlAction, WireError> {
    match cursor.read_u8()? {
        0 => Ok(ControlAction::Pause),
        1 => Ok(ControlAction::Resume),
        2 => Ok(ControlAction::StepFrame),
        3 => Ok(ControlAction::SetInspectMode(cursor.read_u8()? != 0)),
        other => Err(WireError::UnknownTag(other)),
    }
}

fn push_injected_event(buf: &mut Vec<u8>, event: InjectedEvent) {
    match event {
        InjectedEvent::PointerDown { x, y } => {
            push_u8(buf, 0);
            push_i32(buf, x);
            push_i32(buf, y);
        }
        InjectedEvent::PointerUp { x, y } => {
            push_u8(buf, 1);
            push_i32(buf, x);
            push_i32(buf, y);
        }
        InjectedEvent::KeyDown { key } => {
            push_u8(buf, 2);
            push_u32(buf, key);
        }
        InjectedEvent::KeyUp { key } => {
            push_u8(buf, 3);
            push_u32(buf, key);
        }
    }
}

fn read_injected_event(cursor: &mut WireCursor<'_>) -> Result<InjectedEvent, WireError> {
    match cursor.read_u8()? {
        0 => Ok(InjectedEvent::PointerDown {
            x: cursor.read_i32()?,
            y: cursor.read_i32()?,
        }),
        1 => Ok(InjectedEvent::PointerUp {
            x: cursor.read_i32()?,
            y: cursor.read_i32()?,
        }),
        2 => Ok(InjectedEvent::KeyDown {
            key: cursor.read_u32()?,
        }),
        3 => Ok(InjectedEvent::KeyUp {
            key: cursor.read_u32()?,
        }),
        other => Err(WireError::UnknownTag(other)),
    }
}

fn push_snapshot(buf: &mut Vec<u8>, snapshot: &SnapshotMessage) {
    push_u64(buf, snapshot.command.raw());
    push_u8(buf, inspectable_tag(snapshot.target));
    push_str(buf, &snapshot.payload.description);
    push_u64(buf, snapshot.payload.entity_count);
    push_u32(buf, len_u32(snapshot.payload.entity_ids.len()));
    for id in &snapshot.payload.entity_ids {
        push_u64(buf, *id);
    }
    push_str(buf, &snapshot.payload.detail);
}

fn read_snapshot(cursor: &mut WireCursor<'_>) -> Result<SnapshotMessage, WireError> {
    let command = CommandId::new(cursor.read_u64()?);
    let target = read_inspectable(cursor)?;
    let description = cursor.read_str()?.to_owned();
    let entity_count = cursor.read_u64()?;
    let id_count = cursor.read_u32()?;
    let mut entity_ids = Vec::new();
    for _ in 0..id_count {
        entity_ids.push(cursor.read_u64()?);
    }
    let detail = cursor.read_str()?.to_owned();
    let payload = SnapshotPayload::new(description, entity_count, entity_ids).with_detail(detail);
    Ok(SnapshotMessage::new(command, target, payload))
}

fn push_event(buf: &mut Vec<u8>, event: &ProtocolEvent) {
    push_u64(buf, event.subscription.raw());
    push_u8(buf, topic_tag(event.topic));
    match &event.kind {
        EventKind::Diagnostic {
            severity,
            code,
            message,
        } => {
            push_u8(buf, 0);
            push_u8(buf, severity_tag(*severity));
            push_str(buf, code);
            push_str(buf, message);
        }
        EventKind::FrameCommitted { id } => {
            push_u8(buf, 1);
            push_u64(buf, *id);
        }
        EventKind::FocusChanged => push_u8(buf, 2),
        EventKind::LifecycleChanged => push_u8(buf, 3),
    }
}

fn read_event(cursor: &mut WireCursor<'_>) -> Result<ProtocolEvent, WireError> {
    let subscription = SubscriptionId::new(cursor.read_u64()?);
    let topic = read_topic(cursor)?;
    let kind = match cursor.read_u8()? {
        0 => EventKind::Diagnostic {
            severity: read_severity(cursor)?,
            code: cursor.read_str()?.into(),
            message: cursor.read_str()?.into(),
        },
        1 => EventKind::FrameCommitted {
            id: cursor.read_u64()?,
        },
        2 => EventKind::FocusChanged,
        3 => EventKind::LifecycleChanged,
        other => return Err(WireError::UnknownTag(other)),
    };
    Ok(ProtocolEvent::new(subscription, topic, kind))
}

fn push_subscription(buf: &mut Vec<u8>, subscription: &Subscription) {
    push_u64(buf, subscription.id.raw());
    push_u8(buf, topic_tag(subscription.topic));
    push_u8(buf, u8::from(subscription.granted));
}

fn read_subscription(cursor: &mut WireCursor<'_>) -> Result<Subscription, WireError> {
    let id = SubscriptionId::new(cursor.read_u64()?);
    let topic = read_topic(cursor)?;
    let granted = cursor.read_u8()? != 0;
    Ok(if granted {
        Subscription::granted(id, topic)
    } else {
        Subscription::denied(id, topic)
    })
}

fn push_protocol_error(buf: &mut Vec<u8>, error: &ProtocolError) {
    match error {
        ProtocolError::VersionUnsupported => push_u8(buf, 0),
        ProtocolError::Unauthorized { required } => {
            push_u8(buf, 1);
            push_u8(buf, scope_tag(*required));
        }
        ProtocolError::LimitExceeded { limit } => {
            push_u8(buf, 2);
            push_u64(buf, len_u64(*limit));
        }
        ProtocolError::UnknownCommand => push_u8(buf, 3),
        ProtocolError::MalformedMessage => push_u8(buf, 4),
        ProtocolError::ControlDisabled => push_u8(buf, 5),
        ProtocolError::InjectionUnavailable => push_u8(buf, 6),
    }
}

fn read_protocol_error(cursor: &mut WireCursor<'_>) -> Result<ProtocolError, WireError> {
    match cursor.read_u8()? {
        0 => Ok(ProtocolError::VersionUnsupported),
        1 => Ok(ProtocolError::Unauthorized {
            required: read_scope(cursor)?,
        }),
        2 => Ok(ProtocolError::LimitExceeded {
            limit: usize::try_from(cursor.read_u64()?).unwrap_or(usize::MAX),
        }),
        3 => Ok(ProtocolError::UnknownCommand),
        4 => Ok(ProtocolError::MalformedMessage),
        5 => Ok(ProtocolError::ControlDisabled),
        6 => Ok(ProtocolError::InjectionUnavailable),
        other => Err(WireError::UnknownTag(other)),
    }
}

fn push_version(buf: &mut Vec<u8>, version: ProtocolVersion) {
    push_u16(buf, version.major);
    push_u16(buf, version.minor);
}

fn read_version(cursor: &mut WireCursor<'_>) -> Result<ProtocolVersion, WireError> {
    Ok(ProtocolVersion::new(cursor.read_u16()?, cursor.read_u16()?))
}

fn push_version_range(buf: &mut Vec<u8>, range: VersionRange) {
    push_version(buf, range.min);
    push_version(buf, range.max);
}

fn read_version_range(cursor: &mut WireCursor<'_>) -> Result<VersionRange, WireError> {
    let min = read_version(cursor)?;
    let max = read_version(cursor)?;
    Ok(VersionRange::new(min, max))
}

/// Encodes a scope set as a single `u8` bitset via the public `contains` API.
fn push_scope_set(buf: &mut Vec<u8>, scopes: ScopeSet) {
    let mut bits = 0u8;
    for (bit, scope) in scope_bits() {
        if scopes.contains(scope) {
            bits |= bit;
        }
    }
    push_u8(buf, bits);
}

/// Rebuilds a scope set from its `u8` bitset via the public `with` API.
fn read_scope_set(cursor: &mut WireCursor<'_>) -> Result<ScopeSet, WireError> {
    let bits = cursor.read_u8()?;
    let mut set = ScopeSet::empty();
    for (bit, scope) in scope_bits() {
        if bits & bit != 0 {
            set = set.with(scope);
        }
    }
    Ok(set)
}

const fn scope_bits() -> [(u8, PermissionScope); 4] {
    [
        (1 << 0, PermissionScope::Read),
        (1 << 1, PermissionScope::Subscribe),
        (1 << 2, PermissionScope::Control),
        (1 << 3, PermissionScope::Inject),
    ]
}

const fn scope_tag(scope: PermissionScope) -> u8 {
    match scope {
        PermissionScope::Read => 0,
        PermissionScope::Subscribe => 1,
        PermissionScope::Control => 2,
        PermissionScope::Inject => 3,
    }
}

fn read_scope(cursor: &mut WireCursor<'_>) -> Result<PermissionScope, WireError> {
    match cursor.read_u8()? {
        0 => Ok(PermissionScope::Read),
        1 => Ok(PermissionScope::Subscribe),
        2 => Ok(PermissionScope::Control),
        3 => Ok(PermissionScope::Inject),
        other => Err(WireError::UnknownTag(other)),
    }
}

const fn inspectable_tag(target: Inspectable) -> u8 {
    match target {
        Inspectable::Tree => 0,
        Inspectable::Styles => 1,
        Inspectable::Layout => 2,
        Inspectable::EventFlow => 3,
        Inspectable::Focus => 4,
        Inspectable::Animations => 5,
        Inspectable::Resources => 6,
        Inspectable::Accessibility => 7,
        Inspectable::Scene => 8,
        Inspectable::Rendering => 9,
        Inspectable::Caches => 10,
        Inspectable::PluginState => 11,
    }
}

fn read_inspectable(cursor: &mut WireCursor<'_>) -> Result<Inspectable, WireError> {
    match cursor.read_u8()? {
        0 => Ok(Inspectable::Tree),
        1 => Ok(Inspectable::Styles),
        2 => Ok(Inspectable::Layout),
        3 => Ok(Inspectable::EventFlow),
        4 => Ok(Inspectable::Focus),
        5 => Ok(Inspectable::Animations),
        6 => Ok(Inspectable::Resources),
        7 => Ok(Inspectable::Accessibility),
        8 => Ok(Inspectable::Scene),
        9 => Ok(Inspectable::Rendering),
        10 => Ok(Inspectable::Caches),
        11 => Ok(Inspectable::PluginState),
        other => Err(WireError::UnknownTag(other)),
    }
}

const fn topic_tag(topic: Topic) -> u8 {
    match topic {
        Topic::Diagnostics => 0,
        Topic::Lifecycle => 1,
        Topic::Frames => 2,
        Topic::Focus => 3,
    }
}

fn read_topic(cursor: &mut WireCursor<'_>) -> Result<Topic, WireError> {
    match cursor.read_u8()? {
        0 => Ok(Topic::Diagnostics),
        1 => Ok(Topic::Lifecycle),
        2 => Ok(Topic::Frames),
        3 => Ok(Topic::Focus),
        other => Err(WireError::UnknownTag(other)),
    }
}

const fn severity_tag(severity: EventSeverity) -> u8 {
    match severity {
        EventSeverity::Trace => 0,
        EventSeverity::Info => 1,
        EventSeverity::Warning => 2,
        EventSeverity::Error => 3,
        EventSeverity::Critical => 4,
    }
}

fn read_severity(cursor: &mut WireCursor<'_>) -> Result<EventSeverity, WireError> {
    match cursor.read_u8()? {
        0 => Ok(EventSeverity::Trace),
        1 => Ok(EventSeverity::Info),
        2 => Ok(EventSeverity::Warning),
        3 => Ok(EventSeverity::Error),
        4 => Ok(EventSeverity::Critical),
        other => Err(WireError::UnknownTag(other)),
    }
}

fn len_u32(len: usize) -> u32 {
    u32::try_from(len).unwrap_or(u32::MAX)
}

fn len_u64(len: usize) -> u64 {
    u64::try_from(len).unwrap_or(u64::MAX)
}

fn push_u8(buf: &mut Vec<u8>, value: u8) {
    buf.push(value);
}

fn push_u16(buf: &mut Vec<u8>, value: u16) {
    buf.extend_from_slice(&value.to_le_bytes());
}

fn push_u32(buf: &mut Vec<u8>, value: u32) {
    buf.extend_from_slice(&value.to_le_bytes());
}

fn push_i32(buf: &mut Vec<u8>, value: i32) {
    buf.extend_from_slice(&value.to_le_bytes());
}

fn push_u64(buf: &mut Vec<u8>, value: u64) {
    buf.extend_from_slice(&value.to_le_bytes());
}

fn push_str(buf: &mut Vec<u8>, value: &str) {
    let bytes = value.as_bytes();
    push_u32(buf, len_u32(bytes.len()));
    buf.extend_from_slice(bytes);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_bound_handshake_round_trips() {
        let request = HandshakeRequest::new(
            VersionRange::exact(ProtocolVersion::CURRENT),
            "secret-token",
        );
        let message = ServerBound::Handshake(request);
        let bytes = WireCodec::encode_server_bound(&message);
        assert_eq!(WireCodec::decode_server_bound(&bytes), Ok(message));
    }

    #[test]
    fn server_bound_commands_round_trip() {
        let commands = [
            CommandKind::Inspect(Inspectable::PluginState),
            CommandKind::SelectSemantic {
                role: "Button".into(),
                name: Some("Сохранить".into()),
            },
            CommandKind::SelectSemantic {
                role: "Button".into(),
                name: None,
            },
            CommandKind::Subscribe(Topic::Frames),
            CommandKind::Unsubscribe(SubscriptionId::new(9)),
            CommandKind::Control(ControlAction::SetInspectMode(true)),
            CommandKind::Inject(InjectedEvent::PointerDown { x: -3, y: 7 }),
            CommandKind::Inject(InjectedEvent::KeyUp { key: 65 }),
        ];
        for (i, kind) in commands.into_iter().enumerate() {
            let message = ServerBound::Command(CommandMessage::new(
                CommandId::new(u64::try_from(i).expect("small index")),
                kind,
            ));
            let bytes = WireCodec::encode_server_bound(&message);
            assert_eq!(WireCodec::decode_server_bound(&bytes), Ok(message));
        }
    }

    #[test]
    fn client_bound_variants_round_trip() {
        let messages = [
            ClientBound::Handshake(HandshakeResponse::accepted(
                ProtocolVersion::CURRENT,
                ScopeSet::observer(),
            )),
            ClientBound::Handshake(HandshakeResponse::rejected(
                HandshakeRejection::Unauthorized,
            )),
            ClientBound::Snapshot(SnapshotMessage::new(
                CommandId::new(4),
                Inspectable::Tree,
                SnapshotPayload::new("tree", 3, vec![1, 2, 3]),
            )),
            ClientBound::Event(ProtocolEvent::new(
                SubscriptionId::new(2),
                Topic::Diagnostics,
                EventKind::Diagnostic {
                    severity: EventSeverity::Warning,
                    code: "OKUI-1".into(),
                    message: "slow".into(),
                },
            )),
            ClientBound::Subscription(Subscription::granted(SubscriptionId::new(1), Topic::Focus)),
            ClientBound::Acknowledged(CommandId::new(12)),
            ClientBound::Error(ProtocolError::Unauthorized {
                required: PermissionScope::Control,
            }),
            ClientBound::Error(ProtocolError::LimitExceeded { limit: 64 }),
            // A host that cannot receive injected input must be able to *say* so
            // over the socket; a refusal that does not survive the wire is a
            // refusal the CLI never hears.
            ClientBound::Error(ProtocolError::InjectionUnavailable),
        ];
        for message in messages {
            let bytes = WireCodec::encode_client_bound(&message);
            assert_eq!(WireCodec::decode_client_bound(&bytes), Ok(message));
        }
    }

    #[test]
    fn scope_set_bitset_round_trips_all_combinations() {
        for bits in 0u8..16 {
            let mut set = ScopeSet::empty();
            for (bit, scope) in scope_bits() {
                if bits & bit != 0 {
                    set = set.with(scope);
                }
            }
            let response =
                ClientBound::Handshake(HandshakeResponse::accepted(ProtocolVersion::CURRENT, set));
            let bytes = WireCodec::encode_client_bound(&response);
            assert_eq!(WireCodec::decode_client_bound(&bytes), Ok(response));
        }
    }

    #[test]
    fn truncated_buffer_is_unexpected_eof() {
        let message = ServerBound::Command(CommandMessage::new(
            CommandId::new(1),
            CommandKind::Inspect(Inspectable::Scene),
        ));
        let bytes = WireCodec::encode_server_bound(&message);
        assert_eq!(
            WireCodec::decode_server_bound(&bytes[..bytes.len() - 1]),
            Err(WireError::UnexpectedEof)
        );
    }

    #[test]
    fn unknown_tag_is_rejected() {
        assert_eq!(
            WireCodec::decode_server_bound(&[9]),
            Err(WireError::UnknownTag(9))
        );
    }

    #[test]
    fn wire_error_maps_to_malformed_message() {
        assert_eq!(
            ProtocolError::from(WireError::UnexpectedEof),
            ProtocolError::MalformedMessage
        );
    }
}
