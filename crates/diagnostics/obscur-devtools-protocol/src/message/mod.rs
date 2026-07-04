//! Protocol messages: versioning, handshake, commands, snapshots, events, and
//! subscriptions (spec §7.6/§7.7).

mod command_message;
pub use command_message::{CommandId, CommandKind, CommandMessage, ControlAction, InjectedEvent};
mod handshake;
pub use handshake::{HandshakeRejection, HandshakeRequest, HandshakeResponse};
mod inspectable;
pub use inspectable::Inspectable;
mod permission_scope;
pub use permission_scope::{PermissionScope, ScopeSet};
mod protocol_event;
pub use protocol_event::{EventKind, EventSeverity, ProtocolEvent};
mod protocol_version;
pub use protocol_version::{ProtocolVersion, VersionRange};
mod snapshot_message;
pub use snapshot_message::{SnapshotMessage, SnapshotPayload};
mod subscription_message;
pub use subscription_message::{
    MESSAGE_LIMIT, SUBSCRIPTION_LIMIT, SubscribeRequest, SubscribeResponse, Subscription,
    SubscriptionId,
};
mod topic;
pub use topic::Topic;
