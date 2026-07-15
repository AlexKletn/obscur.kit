//! `SourceVersion`: which revision of a CSS source a parse was of.

/// The revision of a [`CssSource`](crate::source::CssSource)'s text (spec §4.12).
///
/// §4.12 requires CSS parsing to be "cached per unique source/version and
/// invalidated explicitly". A [`SourceId`](crate::source::SourceId) answers *which
/// stylesheet*; this answers *which revision of it*, so a cache can tell a hit
/// from a stale entry without comparing the text.
///
/// # Why not hash the text
///
/// Because that is a different guarantee. Hashing says "the bytes match"; a
/// version says "the owner declares this is the same revision". §4.12 asks for
/// *explicit* invalidation, and an explicit version is what makes it explicit: an
/// author who edits a sheet bumps it, and nothing silently decides on their behalf
/// by re-reading and re-hashing the very text the cache exists to avoid touching.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct SourceVersion(u64);

impl SourceVersion {
    /// The revision a freshly authored source starts at.
    pub const INITIAL: Self = Self(0);

    /// Wraps a raw revision value.
    #[must_use]
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    /// The raw revision value.
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }

    /// The next revision — what an owner stamps a source with after editing it.
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
