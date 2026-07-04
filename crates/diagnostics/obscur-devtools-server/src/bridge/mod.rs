//! Runtime bridge serving inspect commands into snapshots (spec §7.6/§7.8).

mod runtime_bridge;
pub use runtime_bridge::{InspectSource, NullInspectSource, RuntimeBridge};
