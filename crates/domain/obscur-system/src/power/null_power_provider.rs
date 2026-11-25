use super::{PowerProfile, PowerProvider, PowerSource};
use obscur_capability::SupportLevel;

#[doc = include_str!("../../docs/api/power/power_provider.doc/NullPowerProvider.md")]
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
