#![doc = include_str!("runtime_bridge.doc/_module.md")]

use obscur_devtools_protocol::message::{CommandId, Inspectable, SnapshotMessage, SnapshotPayload};

#[doc = include_str!("runtime_bridge.doc/InspectSource.md")]
pub trait InspectSource {
    #[doc = include_str!("runtime_bridge.doc/snapshot.md")]
    fn snapshot(&self, target: Inspectable) -> SnapshotPayload;
}

#[doc = include_str!("runtime_bridge.doc/NullInspectSource.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct NullInspectSource;

impl InspectSource for NullInspectSource {
    fn snapshot(&self, target: Inspectable) -> SnapshotPayload {
        SnapshotPayload::summary(format!("no live inspect source for {target:?}"))
    }
}

#[doc = include_str!("runtime_bridge.doc/RuntimeBridge.md")]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RuntimeBridge<S> {
    source: S,
}

impl<S: InspectSource> RuntimeBridge<S> {
    #[doc = include_str!("runtime_bridge.doc/new.md")]
    pub const fn new(source: S) -> Self {
        Self { source }
    }

    #[doc = include_str!("runtime_bridge.doc/inspect.md")]
    pub fn inspect(&self, command: CommandId, target: Inspectable) -> SnapshotMessage {
        SnapshotMessage::new(command, target, self.source.snapshot(target))
    }

    #[doc = include_str!("runtime_bridge.doc/source.md")]
    pub const fn source(&self) -> &S {
        &self.source
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
