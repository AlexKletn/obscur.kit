//! Runtime bridge: serves `Inspect` commands into snapshots (spec §7.6/§7.8).

use obscur_devtools_protocol::message::{
    CommandId, Inspectable, SnapshotMessage, SnapshotPayload,
};

/// A source of live inspectable state the bridge queries on the host's behalf.
///
/// The bridge never reaches into the runtime directly: the host (a
/// `UiRuntime`, in future) implements this trait and hands it to the bridge,
/// mirroring the *host-provides-device* pattern used elsewhere in the kit. Each
/// call maps an [`Inspectable`] target to a typed [`SnapshotPayload`].
pub trait InspectSource {
    /// Returns a typed snapshot payload for `target`.
    fn snapshot(&self, target: Inspectable) -> SnapshotPayload;
}

/// An [`InspectSource`] with no live runtime behind it.
///
/// Every target resolves to an empty, descriptive payload. Useful for tests and
/// for standing an endpoint up before a real runtime is attached.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct NullInspectSource;

impl InspectSource for NullInspectSource {
    fn snapshot(&self, target: Inspectable) -> SnapshotPayload {
        SnapshotPayload::summary(format!("no live inspect source for {target:?}"))
    }
}

/// Serves `Inspect` commands by turning a target into a [`SnapshotMessage`].
///
/// The bridge holds an [`InspectSource`] `S` and does nothing but correlate the
/// command id with the payload the source produces — it can never execute
/// arbitrary code against the runtime.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RuntimeBridge<S> {
    source: S,
}

impl<S: InspectSource> RuntimeBridge<S> {
    /// Creates a bridge over the given inspect `source`.
    pub const fn new(source: S) -> Self {
        Self { source }
    }

    /// Builds the snapshot for `target`, correlated with `command`.
    pub fn inspect(&self, command: CommandId, target: Inspectable) -> SnapshotMessage {
        SnapshotMessage::new(command, target, self.source.snapshot(target))
    }

    /// Returns a reference to the underlying inspect source.
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
