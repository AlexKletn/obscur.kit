//! An inclusive range of supported IR versions [`UiIrVersionRange`] (spec §2.5
//! migrations).

use crate::version::UiIrVersion;

#[doc = include_str!("../../docs/api/version/ui_ir_version_range.doc/UiIrVersionRange.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiIrVersionRange {
    min: UiIrVersion,
    max: UiIrVersion,
}

impl UiIrVersionRange {
    #[doc = include_str!("../../docs/api/version/ui_ir_version_range.doc/new.md")]
    #[must_use]
    pub const fn new(min: UiIrVersion, max: UiIrVersion) -> Self {
        // `UiIrVersion` derives `Ord`, but trait methods are not usable in
        // `const fn`, so compare components directly. Clamp `max` up to `min`
        // when the input is inverted.
        let inverted =
            min.major() > max.major() || (min.major() == max.major() && min.minor() > max.minor());
        let max = if inverted { min } else { max };
        Self { min, max }
    }

    /// Creates a range matching exactly one version: `[v, v]`.
    #[must_use]
    pub const fn exact(v: UiIrVersion) -> Self {
        Self { min: v, max: v }
    }

    /// Creates an open-ended range from `v` up to the maximum representable
    /// version (`[v, (u16::MAX, u16::MAX)]`).
    #[must_use]
    pub const fn at_least(v: UiIrVersion) -> Self {
        Self {
            min: v,
            max: UiIrVersion::new(u16::MAX, u16::MAX),
        }
    }

    /// Returns the inclusive lower bound.
    #[must_use]
    pub const fn min(self) -> UiIrVersion {
        self.min
    }

    /// Returns the inclusive upper bound.
    #[must_use]
    pub const fn max(self) -> UiIrVersion {
        self.max
    }

    /// Returns whether `v` lies within the range: `min <= v <= max`.
    #[must_use]
    pub fn contains(self, v: UiIrVersion) -> bool {
        self.min <= v && v <= self.max
    }

    #[doc = include_str!("../../docs/api/version/ui_ir_version_range.doc/overlaps.md")]
    #[must_use]
    pub fn overlaps(&self, other: &UiIrVersionRange) -> bool {
        self.min <= other.max && other.min <= self.max
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contains_within_bounds() {
        let range = UiIrVersionRange::new(UiIrVersion::new(1, 0), UiIrVersion::new(2, 5));
        assert!(range.contains(UiIrVersion::new(1, 0)));
        assert!(range.contains(UiIrVersion::new(1, 9)));
        assert!(range.contains(UiIrVersion::new(2, 5)));
        assert!(!range.contains(UiIrVersion::new(0, 9)));
        assert!(!range.contains(UiIrVersion::new(2, 6)));
    }

    #[test]
    fn exact_matches_single_version() {
        let range = UiIrVersionRange::exact(UiIrVersion::new(1, 2));
        assert_eq!(range.min(), UiIrVersion::new(1, 2));
        assert_eq!(range.max(), UiIrVersion::new(1, 2));
        assert!(range.contains(UiIrVersion::new(1, 2)));
        assert!(!range.contains(UiIrVersion::new(1, 3)));
    }

    #[test]
    fn at_least_is_open_ended() {
        let range = UiIrVersionRange::at_least(UiIrVersion::new(1, 0));
        assert_eq!(range.max(), UiIrVersion::new(u16::MAX, u16::MAX));
        assert!(range.contains(UiIrVersion::new(1, 0)));
        assert!(range.contains(UiIrVersion::new(9, 9)));
        assert!(!range.contains(UiIrVersion::new(0, 9)));
    }

    #[test]
    fn inverted_input_clamps_max_to_min() {
        let range = UiIrVersionRange::new(UiIrVersion::new(2, 0), UiIrVersion::new(1, 0));
        assert_eq!(range.min(), UiIrVersion::new(2, 0));
        assert_eq!(range.max(), UiIrVersion::new(2, 0));
    }

    #[test]
    fn overlaps_detects_shared_versions() {
        let a = UiIrVersionRange::new(UiIrVersion::new(1, 0), UiIrVersion::new(2, 0));
        let b = UiIrVersionRange::new(UiIrVersion::new(1, 5), UiIrVersion::new(3, 0));
        let c = UiIrVersionRange::new(UiIrVersion::new(2, 1), UiIrVersion::new(4, 0));
        assert!(a.overlaps(&b));
        assert!(b.overlaps(&a));
        assert!(!a.overlaps(&c));
        assert!(!c.overlaps(&a));
    }
}
