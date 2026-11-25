use obscur_devtools_protocol::message::{Inspectable, SnapshotPayload};

use super::InspectSource;

#[doc = include_str!("../../docs/api/bridge/runtime_bridge.doc/NullInspectSource.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct NullInspectSource;

impl InspectSource for NullInspectSource {
    fn snapshot(&self, target: Inspectable) -> SnapshotPayload {
        SnapshotPayload::summary(format!("no live inspect source for {target:?}"))
    }
}
