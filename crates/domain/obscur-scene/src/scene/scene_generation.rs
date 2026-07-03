//! A monotonic generation stamping a scene's content.

use obscur_identity::VersionStamp;

/// A monotonic generation counter stamping the content of a
/// [`Scene`](crate::scene::Scene) (spec §1.6).
///
/// Each content change advances the generation via [`SceneGeneration::next`],
/// so a published [`SceneSnapshot`](crate::scene::SceneSnapshot) can be checked
/// for staleness against the live scene: a **newer** generation (a greater
/// value) means the scene changed after the snapshot was taken, and the
/// snapshot may be discarded as stale.
///
/// This is a newtype over [`VersionStamp`]; ordering follows the underlying
/// stamp, so `a < b` means generation `a` is older than `b`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct SceneGeneration(VersionStamp);

impl SceneGeneration {
    /// The first generation of any scene, with raw value `0`.
    ///
    /// This is also the [`Default`] value.
    pub const FIRST: Self = Self(VersionStamp::from_raw(0));

    /// Wraps a raw generation value.
    #[must_use]
    pub const fn from_raw(raw: u64) -> Self {
        Self(VersionStamp::from_raw(raw))
    }

    /// Returns the raw generation value.
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0.raw()
    }

    /// Returns the next generation, advancing past this one.
    ///
    /// The returned generation compares strictly greater than `self` (barring
    /// the extreme wraparound of the underlying [`VersionStamp::bump`]), marking
    /// that the scene's content changed.
    #[must_use]
    pub const fn next(self) -> Self {
        Self(self.0.bump())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_equals_default_and_is_zero() {
        assert_eq!(SceneGeneration::FIRST, SceneGeneration::default());
        assert_eq!(SceneGeneration::FIRST.raw(), 0);
    }

    #[test]
    fn from_raw_round_trips() {
        assert_eq!(SceneGeneration::from_raw(42).raw(), 42);
    }

    #[test]
    fn next_is_monotonic() {
        let g0 = SceneGeneration::FIRST;
        let g1 = g0.next();
        let g2 = g1.next();
        assert!(g1 > g0);
        assert!(g2 > g1);
        assert_eq!(g1.raw(), 1);
        assert_eq!(g2.raw(), 2);
    }
}
