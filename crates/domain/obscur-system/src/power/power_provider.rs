//! The replaceable port for reading platform power state (spec §7.11).
//!
//! [`PowerProvider`] is the hexagonal port through which the domain asks the host
//! for the current [`PowerProfile`] and learns, via
//! [`availability`](PowerProvider::availability), whether the platform reports
//! power information at all. Capability is honest: a platform without power
//! telemetry answers [`SupportLevel::Unavailable`] and returns the plugged-in
//! baseline rather than fabricating a state. The colocated [`NullPowerProvider`]
//! and [`FixedPowerProvider`] are reference implementations of that contract.

use obscur_capability::SupportLevel;

use crate::power::PowerProfile;

/// The replaceable port for reading the current power state (spec §7.11).
///
/// An adapter implements this trait once per platform and the domain holds it as
/// `Box<dyn PowerProvider>`. The trait is deliberately **object-safe** — no
/// generic or `Self`-returning methods — so providers can be stored
/// polymorphically.
///
/// Contract: a provider reports an honest [`availability`](Self::availability)
/// and never panics — a platform without power telemetry returns
/// [`SupportLevel::Unavailable`] and the plugged-in baseline profile.
pub trait PowerProvider {
    /// Returns the current power state (spec §7.11).
    #[must_use]
    fn profile(&self) -> PowerProfile;

    /// Returns whether power information is available on this platform
    /// (spec §7.11).
    #[must_use]
    fn availability(&self) -> SupportLevel;
}

/// A reference [`PowerProvider`] for platforms without power telemetry
/// (spec §7.11).
///
/// `NullPowerProvider` honestly reports [`SupportLevel::Unavailable`] and returns
/// the plugged-in baseline [`PowerProfile::default`]. Its answers are correct
/// behavior — a documented reference for the honest-capability contract — not a
/// stub waiting to be filled in.
#[derive(Debug, Default, Clone)]
pub struct NullPowerProvider;

impl PowerProvider for NullPowerProvider {
    fn profile(&self) -> PowerProfile {
        // Honest: with no telemetry, assume the plugged-in baseline.
        PowerProfile::default()
    }

    fn availability(&self) -> SupportLevel {
        SupportLevel::Unavailable
    }
}

/// A deterministic [`PowerProvider`] that returns a fixed profile, for tests
/// (spec §7.11).
///
/// `FixedPowerProvider` holds a [`PowerProfile`] supplied at construction and
/// always returns it from [`profile`](PowerProvider::profile), reporting
/// [`SupportLevel::Native`]. Tests use it to drive the domain with an exact,
/// reproducible power state.
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
    use super::*;
    use crate::power::{PowerSource, ThermalState};

    #[test]
    fn power_provider_is_object_safe() {
        let _: Box<dyn PowerProvider> = Box::new(NullPowerProvider);
    }

    #[test]
    fn null_provider_is_unavailable_and_baseline() {
        let provider = NullPowerProvider;
        assert_eq!(provider.availability(), SupportLevel::Unavailable);
        assert_eq!(provider.profile(), PowerProfile::default());
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
