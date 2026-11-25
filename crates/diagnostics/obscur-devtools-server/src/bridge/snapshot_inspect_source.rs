#![doc = include_str!("../../docs/api/bridge/snapshot_inspect_source.doc/_module.md")]

mod inspectable_key;

use inspectable_key::InspectableKey;

use std::collections::BTreeMap;

use obscur_devtools_protocol::message::{Inspectable, SnapshotPayload, StableSelector};

use crate::bridge::InspectSource;

type SemanticIdentity = (u64, Box<str>, Option<Box<str>>);

/// An [`InspectSource`] serving host-registered snapshots per target.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SnapshotInspectSource {
    snapshots: BTreeMap<InspectableKey, SnapshotPayload>,
    semantic: Option<Vec<SemanticIdentity>>,
    stable: Option<BTreeMap<StableSelector, Vec<u64>>>,
}

impl SnapshotInspectSource {
    #[doc = include_str!("../../docs/api/bridge/snapshot_inspect_source.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        Self {
            snapshots: BTreeMap::new(),
            semantic: None,
            stable: None,
        }
    }

    /// Publishes the stable identifiers of the current retained tree.
    #[must_use]
    pub fn with_stable_nodes(
        mut self,
        nodes: impl IntoIterator<Item = (StableSelector, u64)>,
    ) -> Self {
        let mut index = BTreeMap::<_, Vec<_>>::new();
        for (selector, node) in nodes {
            index.entry(selector).or_default().push(node);
        }
        for nodes in index.values_mut() {
            nodes.sort_unstable();
            nodes.dedup();
        }
        self.stable = Some(index);
        self
    }

    /// Publishes visible semantic IDs, role labels, and accessible names.
    #[must_use]
    pub fn with_semantic_nodes(
        mut self,
        nodes: impl IntoIterator<Item = SemanticIdentity>,
    ) -> Self {
        self.semantic = Some(nodes.into_iter().collect());
        self
    }

    /// Registers `payload` for `target`, returning the updated source (builder
    /// form).
    #[must_use]
    pub fn with_snapshot(mut self, target: Inspectable, payload: SnapshotPayload) -> Self {
        self.set_snapshot(target, payload);
        self
    }

    /// Registers or replaces the payload the host reports for `target`.
    pub fn set_snapshot(&mut self, target: Inspectable, payload: SnapshotPayload) {
        self.snapshots.insert(InspectableKey::from(target), payload);
    }

    /// Returns the registered payload for `target`, if the host described it.
    #[must_use]
    pub fn registered(&self, target: Inspectable) -> Option<&SnapshotPayload> {
        self.snapshots.get(&InspectableKey::from(target))
    }
}

impl InspectSource for SnapshotInspectSource {
    fn select_stable(&self, selector: &StableSelector) -> SnapshotPayload {
        let Some(index) = &self.stable else {
            return SnapshotPayload::summary("no stable selector source");
        };
        let nodes = index.get(selector).cloned().unwrap_or_default();
        SnapshotPayload::new("stable selector matches", nodes.len() as u64, nodes)
    }
    fn select_semantic(&self, role: &str, name: Option<&str>) -> SnapshotPayload {
        let Some(nodes) = &self.semantic else {
            return SnapshotPayload::summary("no semantic selector source");
        };
        let ids: Vec<_> = nodes
            .iter()
            .filter(|(_, node_role, node_name)| {
                node_role.as_ref() == role
                    && name.is_none_or(|name| node_name.as_deref() == Some(name))
            })
            .map(|(id, _, _)| *id)
            .collect();
        SnapshotPayload::new("semantic selector matches", ids.len() as u64, ids)
    }
    fn snapshot(&self, target: Inspectable) -> SnapshotPayload {
        self.registered(target)
            .cloned()
            .unwrap_or_else(|| default_payload(target))
    }
}

#[doc = include_str!("../../docs/api/bridge/snapshot_inspect_source.doc/default_payload.md")]
#[must_use]
pub fn default_payload(target: Inspectable) -> SnapshotPayload {
    SnapshotPayload::summary(format!("no snapshot registered for {target:?}"))
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
