#[doc = include_str!("../../docs/api/focus/focus_scope.doc/FocusScopeId.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FocusScopeId(u64);

impl FocusScopeId {
    /// Wraps a raw handle value into a `FocusScopeId`.
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
    use super::*;

    #[test]
    fn scope_id_roundtrips() {
        let id = FocusScopeId::from_raw(7);
        assert_eq!(id.raw(), 7);
    }
}
