//! `StateGeneration`: monotonic change token for local view state.

use obscur_identity::VersionStamp;

#[doc = include_str!("../../docs/api/state/state_generation.doc/StateGeneration.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct StateGeneration(VersionStamp);

impl StateGeneration {
    /// The generation assigned to freshly created state before any mutation.
    pub const INITIAL: Self = Self(VersionStamp::from_raw(0));

    #[doc = include_str!("../../docs/api/state/state_generation.doc/from_raw.md")]
    #[must_use]
    pub const fn from_raw(raw: u64) -> Self {
        Self(VersionStamp::from_raw(raw))
    }

    /// Returns the raw underlying counter value.
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0.raw()
    }

    #[doc = include_str!("../../docs/api/state/state_generation.doc/next.md")]
    #[must_use]
    pub fn next(self) -> Self {
        Self(self.0.bump())
    }

    #[doc = include_str!("../../docs/api/state/state_generation.doc/is_newer_than.md")]
    #[must_use]
    pub fn is_newer_than(self, other: Self) -> bool {
        self > other
    }
}

#[cfg(test)]
mod tests {
    use super::StateGeneration;

    #[test]
    fn initial_is_zero() {
        assert_eq!(StateGeneration::INITIAL.raw(), 0);
        assert_eq!(StateGeneration::default(), StateGeneration::INITIAL);
    }

    #[test]
    fn next_advances_monotonically() {
        let g0 = StateGeneration::INITIAL;
        let g1 = g0.next();
        let g2 = g1.next();
        assert!(g1.is_newer_than(g0));
        assert!(g2.is_newer_than(g1));
        assert_eq!(g2.raw(), 2);
    }

    #[test]
    fn equal_generations_are_not_newer() {
        let g = StateGeneration::from_raw(7);
        assert!(!g.is_newer_than(g));
        assert!(!g.is_newer_than(StateGeneration::from_raw(8)));
    }
}
