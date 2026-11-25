#![doc = include_str!("../../docs/api/power/power_provider.doc/_module.md")]

use super::PowerProfile;
use obscur_capability::SupportLevel;

#[doc = include_str!("../../docs/api/power/power_provider.doc/PowerProvider.md")]
pub trait PowerProvider {
    /// Returns the current power state (spec §7.11).
    #[must_use]
    fn profile(&self) -> PowerProfile;

    /// Returns whether power information is available on this platform
    /// (spec §7.11).
    #[must_use]
    fn availability(&self) -> SupportLevel;
}

#[cfg(test)]
mod tests {
    use super::super::NullPowerProvider;
    use super::*;

    #[test]
    fn power_provider_is_object_safe() {
        let _: Box<dyn PowerProvider> = Box::new(NullPowerProvider);
    }
}
