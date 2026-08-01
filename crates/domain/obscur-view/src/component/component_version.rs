#![doc = include_str!("component_version.doc/_module.md")]

#[doc = include_str!("component_version.doc/ComponentVersion.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ComponentVersion {
    #[doc = include_str!("component_version.doc/major.md")]
    pub major: u16,
    #[doc = include_str!("component_version.doc/minor.md")]
    pub minor: u16,
}

impl ComponentVersion {
    #[doc = include_str!("component_version.doc/INITIAL.md")]
    pub const INITIAL: Self = Self { major: 0, minor: 1 };

    #[doc = include_str!("component_version.doc/new.md")]
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
