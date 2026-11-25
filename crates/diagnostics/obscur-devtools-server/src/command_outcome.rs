use obscur_devtools_protocol::message::{CommandId, SnapshotMessage, Subscription};

#[doc = include_str!("../docs/api/endpoint.doc/CommandOutcome.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandOutcome {
    /// An `Inspect` produced a snapshot of live state.
    Snapshot(SnapshotMessage),
    /// A `Subscribe` produced a subscription record.
    Subscribed(Subscription),
    /// An `Unsubscribe` reported whether a subscription was active.
    Unsubscribed(bool),
    /// A `Control` or `Inject` command was applied, keyed by its id.
    Acknowledged(CommandId),
}
