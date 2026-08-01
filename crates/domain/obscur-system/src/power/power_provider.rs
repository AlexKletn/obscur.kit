#![doc = include_str!("power_provider.doc/_module.md")]

use obscur_capability::SupportLevel;

use crate::power::{PowerProfile, PowerSource};

#[doc = include_str!("power_provider.doc/PowerProvider.md")]
pub trait PowerProvider {
    #[doc = include_str!("power_provider.doc/profile.md")]
    #[must_use]
    fn profile(&self) -> PowerProfile;

    #[doc = include_str!("power_provider.doc/availability.md")]
    #[must_use]
    fn availability(&self) -> SupportLevel;
}

#[doc = include_str!("power_provider.doc/NullPowerProvider.md")]
#[derive(Debug, Default, Clone)]
pub struct NullPowerProvider;

impl PowerProvider for NullPowerProvider {
    fn profile(&self) -> PowerProfile {
        // Honest: with no telemetry, the source is unknown — never a fabricated
        // plugged-in (`Ac`) reading. The rest stays at the unconstrained baseline
        // so an absent provider does not spuriously lower rendering quality.
        PowerProfile::new().on_source(PowerSource::Unknown)
    }

    fn availability(&self) -> SupportLevel {
        SupportLevel::Unavailable
    }
}

#[doc = include_str!("power_provider.doc/FixedPowerProvider.md")]
#[derive(Debug, Default, Clone)]
pub struct FixedPowerProvider {
    profile: PowerProfile,
}

impl FixedPowerProvider {
    #[doc = include_str!("power_provider.doc/new.md")]
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
    use super::*;
    use crate::power::{PowerSource, ThermalState};

    #[test]
    fn power_provider_is_object_safe() {
        let _: Box<dyn PowerProvider> = Box::new(NullPowerProvider);
    }

    #[test]
    fn null_provider_is_unavailable_and_reports_typed_unknown() {
        // Rewritten 2026-07-20 (owner-approved): the §5.18-style "return the
        // plugged-in baseline" decision was retracted for power. An absent
        // provider must report typed absence (§7.11 / README principle 7), so
        // `profile()` no longer equals `PowerProfile::default()` (which is `Ac`).
        let provider = NullPowerProvider;
        assert_eq!(provider.availability(), SupportLevel::Unavailable);
        // The source is the typed Unknown, distinguishable from a fabricated Ac.
        assert_eq!(provider.profile().source(), PowerSource::Unknown);
        assert_ne!(
            provider.profile(),
            PowerProfile::new().on_source(PowerSource::Ac),
            "absence must not be indistinguishable from a genuine plugged-in reading"
        );
        // But absence stays unconstrained, so it never spuriously lowers quality.
        assert!(!provider.profile().is_constrained());
    }

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
