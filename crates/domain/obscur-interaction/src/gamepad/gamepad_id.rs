#[doc = include_str!("../../docs/api/gamepad/gamepad_event.doc/GamepadId.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct GamepadId(u64);

impl GamepadId {
    /// Wraps a raw handle value into a `GamepadId`.
    #[must_use]
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    /// Returns the raw handle value backing this identifier.
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::GamepadId;

    #[test]
    fn id_round_trips_raw() {
        assert_eq!(GamepadId::from_raw(42).raw(), 42);
    }
}
