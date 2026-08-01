#![doc = include_str!("snapshot_message.doc/_module.md")]

use super::{CommandId, Inspectable};

#[doc = include_str!("snapshot_message.doc/SnapshotPayload.md")]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SnapshotPayload {
    #[doc = include_str!("snapshot_message.doc/description.md")]
    pub description: Box<str>,
    #[doc = include_str!("snapshot_message.doc/entity_count.md")]
    pub entity_count: u64,
    #[doc = include_str!("snapshot_message.doc/entity_ids.md")]
    pub entity_ids: Vec<u64>,
    #[doc = include_str!("snapshot_message.doc/detail.md")]
    pub detail: Box<str>,
}

impl SnapshotPayload {
    #[doc = include_str!("snapshot_message.doc/new.md")]
    #[must_use]
    pub fn new(description: impl Into<Box<str>>, entity_count: u64, entity_ids: Vec<u64>) -> Self {
        Self {
            description: description.into(),
            entity_count,
            entity_ids,
            detail: Box::default(),
        }
    }

    #[doc = include_str!("snapshot_message.doc/summary.md")]
    #[must_use]
    pub fn summary(description: impl Into<Box<str>>) -> Self {
        Self {
            description: description.into(),
            entity_count: 0,
            entity_ids: Vec::new(),
            detail: Box::default(),
        }
    }

    #[doc = include_str!("snapshot_message.doc/with_detail.md")]
    #[must_use]
    pub fn with_detail(mut self, detail: impl Into<Box<str>>) -> Self {
        self.detail = detail.into();
        self
    }
}

#[doc = include_str!("snapshot_message.doc/SnapshotMessage.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotMessage {
    #[doc = include_str!("snapshot_message.doc/command.md")]
    pub command: CommandId,
    #[doc = include_str!("snapshot_message.doc/target.md")]
    pub target: Inspectable,
    #[doc = include_str!("snapshot_message.doc/payload.md")]
    pub payload: SnapshotPayload,
}

impl SnapshotMessage {
    #[doc = include_str!("snapshot_message.doc/new_2.md")]
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
