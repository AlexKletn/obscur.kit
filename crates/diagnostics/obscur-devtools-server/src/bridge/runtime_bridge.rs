//! Runtime bridge: serves `Inspect` commands into snapshots (spec §7.6/§7.8).

use obscur_devtools_protocol::message::{CommandId, Inspectable, SnapshotMessage, StableSelector};

use super::InspectSource;

#[doc = include_str!("../../docs/api/bridge/runtime_bridge.doc/RuntimeBridge.md")]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RuntimeBridge<S> {
    source: S,
}

impl<S: InspectSource> RuntimeBridge<S> {
    /// Returns stable ID matches through the tree snapshot response.
    pub fn select_stable(&self, command: CommandId, selector: &StableSelector) -> SnapshotMessage {
        SnapshotMessage::new(
            command,
            Inspectable::Tree,
            self.source.select_stable(selector),
        )
    }
    /// Creates a bridge over the given inspect `source`.
    pub const fn new(source: S) -> Self {
        Self { source }
    }

    /// Builds the snapshot for `target`, correlated with `command`.
    pub fn inspect(&self, command: CommandId, target: Inspectable) -> SnapshotMessage {
        SnapshotMessage::new(command, target, self.source.snapshot(target))
    }

    /// Returns semantic matches through the accessibility snapshot response.
    pub fn select_semantic(
        &self,
        command: CommandId,
        role: &str,
        name: Option<&str>,
    ) -> SnapshotMessage {
        SnapshotMessage::new(
            command,
            Inspectable::Accessibility,
            self.source.select_semantic(role, name),
        )
    }

    /// Returns a reference to the underlying inspect source.
    pub const fn source(&self) -> &S {
        &self.source
    }
}

#[cfg(test)]
mod tests {
    use super::super::NullInspectSource;
    use super::*;
    use obscur_devtools_protocol::message::SnapshotPayload;

    /// A test source returning a fixed payload tagged with the requested target.
    #[derive(Debug, Default)]
    struct FixedSource;

    impl InspectSource for FixedSource {
        fn snapshot(&self, target: Inspectable) -> SnapshotPayload {
            SnapshotPayload::new(format!("fixed:{target:?}"), 3, vec![1, 2, 3])
        }
    }

    #[test]
    fn inspect_returns_source_payload_for_target() {
        let bridge = RuntimeBridge::new(FixedSource);
        let msg = bridge.inspect(CommandId::new(7), Inspectable::Layout);
        assert_eq!(msg.command, CommandId::new(7));
        assert_eq!(msg.target, Inspectable::Layout);
        assert_eq!(&*msg.payload.description, "fixed:Layout");
        assert_eq!(msg.payload.entity_count, 3);
        assert_eq!(msg.payload.entity_ids, vec![1, 2, 3]);
    }

    #[test]
    fn null_source_yields_empty_payload() {
        let bridge = RuntimeBridge::new(NullInspectSource);
        let msg = bridge.inspect(CommandId::new(1), Inspectable::Scene);
        assert_eq!(msg.payload.entity_count, 0);
        assert!(msg.payload.entity_ids.is_empty());
        assert_eq!(*bridge.source(), NullInspectSource);
    }
}
