#[doc = include_str!("../../docs/api/power/power_profile.doc/ThermalState.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub enum ThermalState {
    /// No thermal pressure.
    #[default]
    Nominal,
    /// Mild thermal pressure; still comfortable.
    Fair,
    /// Serious thermal pressure; quality should be reduced.
    Serious,
    /// Critical thermal pressure; aggressive throttling is warranted.
    Critical,
}

#[cfg(test)]
mod tests {
    use super::ThermalState;

    #[test]
    fn thermal_states_are_ordered() {
        assert!(ThermalState::Nominal < ThermalState::Fair);
        assert!(ThermalState::Fair < ThermalState::Serious);
        assert!(ThermalState::Serious < ThermalState::Critical);
    }
}
