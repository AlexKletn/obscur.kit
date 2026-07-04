//! Server→client snapshot response to an `Inspect` command (§7.6/§7.8).

use super::{CommandId, Inspectable};

/// A structured, typed summary of an inspected target.
///
/// This crate cannot serialize the live runtime, so the payload is a typed
/// container the server fills from its runtime bridge: a human-readable
/// `description` plus coarse counts and ids. It is deliberately typed and
/// structured rather than free-form log text (spec §7.8).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SnapshotPayload {
    /// A short, human-readable description of the snapshot.
    pub description: Box<str>,
    /// The number of entities summarized (e.g. nodes, styles, resources).
    pub entity_count: u64,
    /// Stable ids of the summarized entities, when the server enumerates them
    /// (may be empty).
    pub entity_ids: Vec<u64>,
}

impl SnapshotPayload {
    /// Creates a snapshot payload.
    #[must_use]
    pub fn new(
        description: impl Into<Box<str>>,
        entity_count: u64,
        entity_ids: Vec<u64>,
    ) -> Self {
        Self { description: description.into(), entity_count, entity_ids }
    }

    /// Creates a payload carrying only a description.
    #[must_use]
    pub fn summary(description: impl Into<Box<str>>) -> Self {
        Self { description: description.into(), entity_count: 0, entity_ids: Vec::new() }
    }
}

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
        Self { command, target, payload }
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
