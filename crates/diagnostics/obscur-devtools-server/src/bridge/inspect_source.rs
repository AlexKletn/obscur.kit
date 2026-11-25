use obscur_devtools_protocol::message::{Inspectable, SnapshotPayload, StableSelector};

#[doc = include_str!("../../docs/api/bridge/runtime_bridge.doc/InspectSource.md")]
pub trait InspectSource {
    /// Returns matching node IDs from a published retained tree.
    fn select_stable(&self, _selector: &StableSelector) -> SnapshotPayload {
        SnapshotPayload::summary("no stable selector source")
    }
    /// Returns a typed snapshot payload for `target`.
    fn snapshot(&self, target: Inspectable) -> SnapshotPayload;

    /// Returns matching node IDs from a published semantic projection.
    fn select_semantic(&self, _role: &str, _name: Option<&str>) -> SnapshotPayload {
        SnapshotPayload::summary("no semantic selector source")
    }
}
