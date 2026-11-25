//! `ComponentVersion`: the schema version of a registered component (spec §2.14).

#[doc = include_str!("../../docs/api/component/component_version.doc/ComponentVersion.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ComponentVersion {
    /// Major version. Bumped for incompatible contract changes.
    pub major: u16,
    /// Minor version. Bumped for backward-compatible contract additions.
    pub minor: u16,
}

impl ComponentVersion {
    /// The initial `0.1` version, a sensible default for a first registration.
    pub const INITIAL: Self = Self { major: 0, minor: 1 };

    /// Creates a version from its `major` and `minor` parts.
    #[must_use]
    pub const fn new(major: u16, minor: u16) -> Self {
        Self { major, minor }
    }
}

impl Default for ComponentVersion {
    fn default() -> Self {
        Self::INITIAL
    }
}

#[cfg(test)]
mod tests {
    use super::ComponentVersion;

    #[test]
    fn new_sets_parts() {
        let v = ComponentVersion::new(2, 7);
        assert_eq!(v.major, 2);
        assert_eq!(v.minor, 7);
    }

    #[test]
    fn ordering_is_major_then_minor() {
        assert!(ComponentVersion::new(1, 0) < ComponentVersion::new(1, 5));
        assert!(ComponentVersion::new(1, 5) < ComponentVersion::new(2, 0));
    }

    #[test]
    fn default_is_initial() {
        assert_eq!(ComponentVersion::default(), ComponentVersion::INITIAL);
        assert_eq!(ComponentVersion::INITIAL, ComponentVersion::new(0, 1));
    }
}
