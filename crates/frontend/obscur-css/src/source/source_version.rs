#![doc = include_str!("source_version.doc/_module.md")]

#[doc = include_str!("source_version.doc/SourceVersion.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct SourceVersion(u64);

impl SourceVersion {
    #[doc = include_str!("source_version.doc/INITIAL.md")]
    pub const INITIAL: Self = Self(0);

    #[doc = include_str!("source_version.doc/from_raw.md")]
    #[must_use]
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    #[doc = include_str!("source_version.doc/raw.md")]
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }

    #[doc = include_str!("source_version.doc/next.md")]
    #[must_use]
    pub const fn next(self) -> Self {
        Self(self.0.wrapping_add(1))
    }
}

#[cfg(test)]
mod tests {
    use super::SourceVersion;

    #[test]
    fn a_fresh_source_starts_at_the_initial_revision() {
        assert_eq!(SourceVersion::default(), SourceVersion::INITIAL);
        assert_eq!(SourceVersion::INITIAL.raw(), 0);
    }

    #[test]
    fn each_edit_yields_a_distinct_revision() {
        let first = SourceVersion::INITIAL;
        let second = first.next();
        assert_ne!(first, second);
        assert!(second > first);
    }
}
