#![doc = include_str!("mod.doc/_module.md")]

mod runtime_bridge;
pub use runtime_bridge::{InspectSource, NullInspectSource, RuntimeBridge};
mod snapshot_inspect_source;
pub use snapshot_inspect_source::{SnapshotInspectSource, default_payload};
