#![doc = include_str!("scene_generation.doc/_module.md")]

use obscur_identity::VersionStamp;

#[doc = include_str!("scene_generation.doc/SceneGeneration.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct SceneGeneration(VersionStamp);

impl SceneGeneration {
    #[doc = include_str!("scene_generation.doc/FIRST.md")]
    pub const FIRST: Self = Self(VersionStamp::from_raw(0));

    #[doc = include_str!("scene_generation.doc/from_raw.md")]
    #[must_use]
    pub const fn from_raw(raw: u64) -> Self {
        Self(VersionStamp::from_raw(raw))
    }

    #[doc = include_str!("scene_generation.doc/raw.md")]
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0.raw()
    }

    #[doc = include_str!("scene_generation.doc/next.md")]
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
