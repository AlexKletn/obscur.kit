//! The versioned IR format version [`UiIrVersion`] (spec §2.5 versioned IR).

#[doc = include_str!("../../docs/api/version/ui_ir_version.doc/UiIrVersion.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct UiIrVersion {
    major: u16,
    minor: u16,
}

impl UiIrVersion {
    /// The current IR format version emitted by this crate.
    pub const CURRENT: UiIrVersion = UiIrVersion::new(1, 0);

    /// Creates a version from its `major` and `minor` components.
    #[must_use]
    pub const fn new(major: u16, minor: u16) -> Self {
        Self { major, minor }
    }

    /// Returns the major component (breaking-change gate).
    #[must_use]
    pub const fn major(self) -> u16 {
        self.major
    }

    /// Returns the minor component (backward-compatible-addition gate).
    #[must_use]
    pub const fn minor(self) -> u16 {
        self.minor
    }

    #[doc = include_str!("../../docs/api/version/ui_ir_version.doc/is_compatible_with.md")]
    #[must_use]
    pub const fn is_compatible_with(self, other: UiIrVersion) -> bool {
        self.major == other.major && self.minor >= other.minor
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_is_one_zero() {
        assert_eq!(UiIrVersion::CURRENT, UiIrVersion::new(1, 0));
        assert_eq!(UiIrVersion::CURRENT.major(), 1);
        assert_eq!(UiIrVersion::CURRENT.minor(), 0);
    }

    #[test]
    fn ordering_is_lexicographic() {
        assert!(UiIrVersion::new(1, 0) < UiIrVersion::new(1, 1));
        assert!(UiIrVersion::new(1, 9) < UiIrVersion::new(2, 0));
        assert!(UiIrVersion::new(2, 0) > UiIrVersion::new(1, 9));
    }

    #[test]
    fn compatible_same_major_newer_minor() {
        // A reader at 1.2 can read a document at 1.1 (older minor).
        assert!(UiIrVersion::new(1, 2).is_compatible_with(UiIrVersion::new(1, 1)));
        // And one at exactly its own version.
        assert!(UiIrVersion::new(1, 2).is_compatible_with(UiIrVersion::new(1, 2)));
    }

    #[test]
    fn incompatible_same_major_older_reader() {
        // A reader at 1.1 cannot read a document at 1.2 (newer minor).
        assert!(!UiIrVersion::new(1, 1).is_compatible_with(UiIrVersion::new(1, 2)));
    }

    #[test]
    fn incompatible_different_major() {
        assert!(!UiIrVersion::new(2, 0).is_compatible_with(UiIrVersion::new(1, 0)));
        assert!(!UiIrVersion::new(1, 0).is_compatible_with(UiIrVersion::new(2, 0)));
    }
}
