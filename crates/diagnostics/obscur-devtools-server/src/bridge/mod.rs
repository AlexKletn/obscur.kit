//! Runtime bridge serving inspect commands into snapshots (spec §7.6/§7.8).

mod runtime_bridge;
pub use runtime_bridge::{InspectSource, NullInspectSource, RuntimeBridge};
mod snapshot_inspect_source;
pub use snapshot_inspect_source::{SnapshotInspectSource, default_payload};
