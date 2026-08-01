#![doc = include_str!("power_profile.doc/_module.md")]

#[doc = include_str!("power_profile.doc/PowerSource.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PowerSource {
    #[doc = include_str!("power_profile.doc/Ac.md")]
    #[default]
    Ac,
    #[doc = include_str!("power_profile.doc/Battery.md")]
    Battery,
    #[doc = include_str!("power_profile.doc/Unknown.md")]
    Unknown,
}

#[doc = include_str!("power_profile.doc/ThermalState.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub enum ThermalState {
    #[doc = include_str!("power_profile.doc/Nominal.md")]
    #[default]
    Nominal,
    #[doc = include_str!("power_profile.doc/Fair.md")]
    Fair,
    #[doc = include_str!("power_profile.doc/Serious.md")]
    Serious,
    #[doc = include_str!("power_profile.doc/Critical.md")]
    Critical,
}

#[doc = include_str!("power_profile.doc/PowerProfile.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PowerProfile {
    source: PowerSource,
    battery_level: Option<f32>,
    low_power_mode: bool,
    thermal: ThermalState,
}

impl PowerProfile {
    #[doc = include_str!("power_profile.doc/LOW_BATTERY_THRESHOLD.md")]
    pub const LOW_BATTERY_THRESHOLD: f32 = 0.2;

    #[doc = include_str!("power_profile.doc/new.md")]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            source: PowerSource::Ac,
            battery_level: None,
            low_power_mode: false,
            thermal: ThermalState::Nominal,
        }
    }

    #[doc = include_str!("power_profile.doc/on_source.md")]
    #[must_use]
    pub const fn on_source(mut self, source: PowerSource) -> Self {
        self.source = source;
        self
    }

    #[doc = include_str!("power_profile.doc/with_battery_level.md")]
    #[must_use]
    pub fn with_battery_level(mut self, level: f32) -> Self {
        self.battery_level = Some(level.clamp(0.0, 1.0));
        self
    }

    #[doc = include_str!("power_profile.doc/low_power_mode.md")]
    #[must_use]
    pub const fn low_power_mode(mut self, enabled: bool) -> Self {
        self.low_power_mode = enabled;
        self
    }

    #[doc = include_str!("power_profile.doc/with_thermal.md")]
    #[must_use]
    pub const fn with_thermal(mut self, thermal: ThermalState) -> Self {
        self.thermal = thermal;
        self
    }

    #[doc = include_str!("power_profile.doc/is_constrained.md")]
    #[must_use]
    pub fn is_constrained(&self) -> bool {
        let low_battery = matches!(self.source, PowerSource::Battery)
            && matches!(self.battery_level, Some(level) if level < Self::LOW_BATTERY_THRESHOLD);
        self.low_power_mode || low_battery || self.thermal >= ThermalState::Serious
    }

    #[doc = include_str!("power_profile.doc/on_battery.md")]
    #[must_use]
    pub const fn on_battery(&self) -> bool {
        matches!(self.source, PowerSource::Battery)
    }

    #[doc = include_str!("power_profile.doc/source.md")]
    #[must_use]
    pub const fn source(&self) -> PowerSource {
        self.source
    }

    #[doc = include_str!("power_profile.doc/battery_level.md")]
    #[must_use]
    pub const fn battery_level(&self) -> Option<f32> {
        self.battery_level
    }

    #[doc = include_str!("power_profile.doc/is_low_power_mode.md")]
    #[must_use]
    pub const fn is_low_power_mode(&self) -> bool {
        self.low_power_mode
    }

    #[doc = include_str!("power_profile.doc/thermal.md")]
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

    #[test]
    fn thermal_states_are_ordered() {
        assert!(ThermalState::Nominal < ThermalState::Fair);
        assert!(ThermalState::Fair < ThermalState::Serious);
        assert!(ThermalState::Serious < ThermalState::Critical);
    }
}
