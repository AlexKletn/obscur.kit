//! Server→client snapshot response to an `Inspect` command (§7.6/§7.8).

use super::{CommandId, Inspectable, SnapshotPayload};

/// A server→client response carrying a snapshot of an inspected target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotMessage {
    /// The [`CommandId`] of the `Inspect` command this answers.
    pub command: CommandId,
    /// Which target was inspected.
    pub target: Inspectable,
    /// The structured snapshot payload.
    pub payload: SnapshotPayload,
}

impl SnapshotMessage {
    /// Creates a snapshot message.
    #[must_use]
    pub fn new(command: CommandId, target: Inspectable, payload: SnapshotPayload) -> Self {
        Self {
            command,
            target,
            payload,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn payload_summary_is_empty_counts() {
        let p = SnapshotPayload::summary("tree with 3 roots");
        assert_eq!(&*p.description, "tree with 3 roots");
        assert_eq!(p.entity_count, 0);
        assert!(p.entity_ids.is_empty());
    }

    #[test]
    fn message_round_trips_fields() {
        let payload = SnapshotPayload::new("layout", 2, vec![10, 20]);
        let msg = SnapshotMessage::new(CommandId::new(9), Inspectable::Layout, payload.clone());
        assert_eq!(msg.command, CommandId::new(9));
        assert_eq!(msg.target, Inspectable::Layout);
        assert_eq!(msg.payload, payload);
        assert_eq!(msg.payload.entity_ids, vec![10, 20]);
    }
}
