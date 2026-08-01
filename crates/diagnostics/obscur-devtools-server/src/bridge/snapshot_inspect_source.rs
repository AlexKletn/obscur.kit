#![doc = include_str!("snapshot_inspect_source.doc/_module.md")]

use std::collections::BTreeMap;

use obscur_devtools_protocol::message::{Inspectable, SnapshotPayload};

use crate::bridge::InspectSource;

#[doc = include_str!("snapshot_inspect_source.doc/SnapshotInspectSource.md")]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SnapshotInspectSource {
    snapshots: BTreeMap<InspectableKey, SnapshotPayload>,
}

impl SnapshotInspectSource {
    #[doc = include_str!("snapshot_inspect_source.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        Self {
            snapshots: BTreeMap::new(),
        }
    }

    #[doc = include_str!("snapshot_inspect_source.doc/with_snapshot.md")]
    #[must_use]
    pub fn with_snapshot(mut self, target: Inspectable, payload: SnapshotPayload) -> Self {
        self.set_snapshot(target, payload);
        self
    }

    #[doc = include_str!("snapshot_inspect_source.doc/set_snapshot.md")]
    pub fn set_snapshot(&mut self, target: Inspectable, payload: SnapshotPayload) {
        self.snapshots.insert(InspectableKey::from(target), payload);
    }

    #[doc = include_str!("snapshot_inspect_source.doc/registered.md")]
    #[must_use]
    pub fn registered(&self, target: Inspectable) -> Option<&SnapshotPayload> {
        self.snapshots.get(&InspectableKey::from(target))
    }
}

impl InspectSource for SnapshotInspectSource {
    fn snapshot(&self, target: Inspectable) -> SnapshotPayload {
        self.registered(target)
            .cloned()
            .unwrap_or_else(|| default_payload(target))
    }
}

#[doc = include_str!("snapshot_inspect_source.doc/default_payload.md")]
#[must_use]
pub fn default_payload(target: Inspectable) -> SnapshotPayload {
    SnapshotPayload::summary(format!("no snapshot registered for {target:?}"))
}

#[doc = include_str!("snapshot_inspect_source.doc/InspectableKey.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct InspectableKey(u8);

impl From<Inspectable> for InspectableKey {
    fn from(target: Inspectable) -> Self {
        Self(match target {
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
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bridge::RuntimeBridge;
    use obscur_devtools_protocol::message::CommandId;

    #[test]
    fn returns_registered_snapshot_not_null() {
        let source = SnapshotInspectSource::new().with_snapshot(
            Inspectable::Layout,
            SnapshotPayload::new("layout", 4, vec![1, 2]),
        );
        let payload = source.snapshot(Inspectable::Layout);
        assert_eq!(&*payload.description, "layout");
        assert_eq!(payload.entity_count, 4);
        assert_eq!(payload.entity_ids, vec![1, 2]);
    }

    #[test]
    fn unregistered_target_uses_typed_default() {
        let source = SnapshotInspectSource::new();
        let payload = source.snapshot(Inspectable::Scene);
        assert!(payload.description.contains("Scene"));
        assert_eq!(payload.entity_count, 0);
        assert!(payload.entity_ids.is_empty());
    }

    #[test]
    fn set_snapshot_replaces_previous() {
        let mut source = SnapshotInspectSource::new();
        source.set_snapshot(Inspectable::Tree, SnapshotPayload::new("old", 1, vec![]));
        source.set_snapshot(Inspectable::Tree, SnapshotPayload::new("new", 2, vec![9]));
        assert_eq!(&*source.snapshot(Inspectable::Tree).description, "new");
    }

    #[test]
    fn serves_through_the_runtime_bridge() {
        let source = SnapshotInspectSource::new().with_snapshot(
            Inspectable::Focus,
            SnapshotPayload::new("focused", 1, vec![42]),
        );
        let bridge = RuntimeBridge::new(source);
        let message = bridge.inspect(CommandId::new(5), Inspectable::Focus);
        assert_eq!(message.command, CommandId::new(5));
        assert_eq!(message.target, Inspectable::Focus);
        assert_eq!(message.payload.entity_ids, vec![42]);
    }
}
