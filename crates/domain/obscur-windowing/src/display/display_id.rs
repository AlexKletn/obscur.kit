#[doc = include_str!("../../docs/api/display/display.doc/DisplayId.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DisplayId(u64);

impl DisplayId {
    /// Wraps a raw handle value into a `DisplayId`.
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
    use super::DisplayId;

    #[test]
    fn display_id_round_trips() {
        assert_eq!(DisplayId::from_raw(42).raw(), 42);
    }
}
