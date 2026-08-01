#![doc = include_str!("mod.doc/_module.md")]
mod power_profile;
pub use power_profile::{PowerProfile, PowerSource, ThermalState};
mod power_provider;
pub use power_provider::{FixedPowerProvider, NullPowerProvider, PowerProvider};
