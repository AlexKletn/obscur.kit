//! `StateGeneration`: monotonic change token for local view state.

use obscur_identity::VersionStamp;

/// Monotonic change token stamped onto a value each time it is mutated.
///
/// A consumer records the `StateGeneration` it last observed and later compares
/// it against the source's current generation to decide whether it must react.
/// This is the mechanism behind the spec's "invalidate only consumers of
/// changed values": equal generations mean nothing changed, so no work is done.
///
/// Backed by [`VersionStamp`] so the whole workspace shares one change-versioning
/// primitive rather than re-inventing a raw counter. Generations are totally
/// ordered; a larger value is strictly newer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct StateGeneration(VersionStamp);

impl StateGeneration {
    /// The generation assigned to freshly created state before any mutation.
    pub const INITIAL: Self = Self(VersionStamp::from_raw(0));

    /// Wraps a raw generation value.
    ///
    /// Intended for reconstructing a previously observed generation (for example
    /// when rehydrating a consumer's last-seen watermark); ordinary code advances
    /// generations with [`StateGeneration::next`].
    #[must_use]
    pub const fn from_raw(raw: u64) -> Self {
        Self(VersionStamp::from_raw(raw))
    }

    /// Returns the raw underlying counter value.
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0.raw()
    }

    /// Returns the next generation, advancing the monotonic counter by one.
    ///
    /// Wraps on overflow, matching the backing [`VersionStamp`]; a `u64` counter
    /// makes real-world wraparound irrelevant.
    #[must_use]
    pub fn next(self) -> Self {
        Self(self.0.bump())
    }

    /// Returns `true` if `self` is strictly newer than `other`.
    ///
    /// Consumers use this against their last-observed generation to detect a
    /// pending change without diffing values.
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
