//! Power module.
mod power_profile;
pub use power_profile::PowerProfile;
mod power_source;
pub use power_source::PowerSource;
mod thermal_state;
pub use thermal_state::ThermalState;
mod power_provider;
pub use power_provider::PowerProvider;
mod null_power_provider;
pub use null_power_provider::NullPowerProvider;
mod fixed_power_provider;
pub use fixed_power_provider::FixedPowerProvider;
