#![doc = include_str!("../../docs/api/power/power_profile.doc/_module.md")]

use super::{PowerSource, ThermalState};

#[doc = include_str!("../../docs/api/power/power_profile.doc/PowerProfile.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PowerProfile {
    source: PowerSource,
    battery_level: Option<f32>,
    low_power_mode: bool,
    thermal: ThermalState,
}

impl PowerProfile {
    /// The battery charge below which a battery-powered device is treated as
    /// constrained (20%).
    pub const LOW_BATTERY_THRESHOLD: f32 = 0.2;

    /// Creates the plugged-in baseline: on AC, no battery reading, low-power mode
    /// off, and nominal thermal state (spec §7.11).
    #[must_use]
    pub const fn new() -> Self {
        Self {
            source: PowerSource::Ac,
            battery_level: None,
            low_power_mode: false,
            thermal: ThermalState::Nominal,
        }
    }

    /// Sets the power source (spec §7.11).
    #[must_use]
    pub const fn on_source(mut self, source: PowerSource) -> Self {
        self.source = source;
        self
    }

    /// Sets the battery charge level, clamped to `0.0..=1.0` (spec §7.11).
    #[must_use]
    pub fn with_battery_level(mut self, level: f32) -> Self {
        self.battery_level = Some(level.clamp(0.0, 1.0));
        self
    }

    /// Sets whether the OS low-power / battery-saver mode is engaged
    /// (spec §7.11).
    #[must_use]
    pub const fn low_power_mode(mut self, enabled: bool) -> Self {
        self.low_power_mode = enabled;
        self
    }

    /// Sets the thermal pressure state (spec §7.11).
    #[must_use]
    pub const fn with_thermal(mut self, thermal: ThermalState) -> Self {
        self.thermal = thermal;
        self
    }

    #[doc = include_str!("../../docs/api/power/power_profile.doc/is_constrained.md")]
    #[must_use]
    pub fn is_constrained(&self) -> bool {
        let low_battery = matches!(self.source, PowerSource::Battery)
            && matches!(self.battery_level, Some(level) if level < Self::LOW_BATTERY_THRESHOLD);
        self.low_power_mode || low_battery || self.thermal >= ThermalState::Serious
    }

    /// Returns whether the device is running on battery (spec §7.11).
    #[must_use]
    pub const fn on_battery(&self) -> bool {
        matches!(self.source, PowerSource::Battery)
    }

    /// Returns the current power source (spec §7.11).
    #[must_use]
    pub const fn source(&self) -> PowerSource {
        self.source
    }

    /// Returns the battery charge level in `0.0..=1.0`, or `None` if unknown or
    /// on AC (spec §7.11).
    #[must_use]
    pub const fn battery_level(&self) -> Option<f32> {
        self.battery_level
    }

    /// Returns whether OS low-power / battery-saver mode is engaged (spec §7.11).
    #[must_use]
    pub const fn is_low_power_mode(&self) -> bool {
        self.low_power_mode
    }

    /// Returns the current thermal pressure state (spec §7.11).
    #[must_use]
    pub const fn thermal(&self) -> ThermalState {
        self.thermal
    }
}

impl Default for PowerProfile {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plugged_in_baseline_is_not_constrained() {
        let profile = PowerProfile::default();
        assert_eq!(profile.source(), PowerSource::Ac);
        assert_eq!(profile.battery_level(), None);
        assert!(!profile.is_low_power_mode());
        assert_eq!(profile.thermal(), ThermalState::Nominal);
        assert!(!profile.is_constrained());
        assert!(!profile.on_battery());
    }

    #[test]
    fn low_power_mode_constrains() {
        let profile = PowerProfile::new().low_power_mode(true);
        assert!(profile.is_constrained());
    }

    #[test]
    fn low_battery_on_battery_constrains() {
        let profile = PowerProfile::new()
            .on_source(PowerSource::Battery)
            .with_battery_level(0.1);
        assert!(profile.on_battery());
        assert!(profile.is_constrained());
    }

    #[test]
    fn healthy_battery_is_not_constrained() {
        let profile = PowerProfile::new()
            .on_source(PowerSource::Battery)
            .with_battery_level(0.8);
        assert!(!profile.is_constrained());
    }

    #[test]
    fn low_battery_on_ac_is_not_constrained() {
        // A low reading only matters when actually on battery.
        let profile = PowerProfile::new().with_battery_level(0.05);
        assert!(!profile.is_constrained());
    }

    #[test]
    fn serious_thermal_constrains() {
        assert!(
            PowerProfile::new()
                .with_thermal(ThermalState::Serious)
                .is_constrained()
        );
        assert!(
            PowerProfile::new()
                .with_thermal(ThermalState::Critical)
                .is_constrained()
        );
        assert!(
            !PowerProfile::new()
                .with_thermal(ThermalState::Fair)
                .is_constrained()
        );
    }

    #[test]
    fn battery_level_is_clamped() {
        assert_eq!(
            PowerProfile::new().with_battery_level(1.5).battery_level(),
            Some(1.0)
        );
        assert_eq!(
            PowerProfile::new().with_battery_level(-0.3).battery_level(),
            Some(0.0)
        );
    }
}
