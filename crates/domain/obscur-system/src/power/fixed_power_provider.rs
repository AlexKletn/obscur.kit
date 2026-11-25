use super::{PowerProfile, PowerProvider};
use obscur_capability::SupportLevel;

#[doc = include_str!("../../docs/api/power/power_provider.doc/FixedPowerProvider.md")]
#[derive(Debug, Default, Clone)]
pub struct FixedPowerProvider {
    profile: PowerProfile,
}

impl FixedPowerProvider {
    /// Creates a provider that always reports `profile` (spec §7.11).
    #[must_use]
    pub const fn new(profile: PowerProfile) -> Self {
        Self { profile }
    }
}

impl PowerProvider for FixedPowerProvider {
    fn profile(&self) -> PowerProfile {
        self.profile
    }

    fn availability(&self) -> SupportLevel {
        SupportLevel::Native
    }
}

#[cfg(test)]
mod tests {
    use super::super::{PowerSource, ThermalState};
    use super::*;

    #[test]
    fn fixed_provider_returns_its_profile_and_is_native() {
        let profile = PowerProfile::new()
            .on_source(PowerSource::Battery)
            .with_battery_level(0.5)
            .with_thermal(ThermalState::Fair);
        let provider = FixedPowerProvider::new(profile);
        assert_eq!(provider.availability(), SupportLevel::Native);
        assert_eq!(provider.profile(), profile);
    }

    #[test]
    fn fixed_provider_default_is_baseline() {
        let provider = FixedPowerProvider::default();
        assert_eq!(provider.profile(), PowerProfile::default());
    }
}
