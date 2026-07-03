//! A single cached measure/arrange result for a node.

use crate::model::{LayoutBox, LayoutResult};
use obscur_geometry::BoxConstraints;
use obscur_identity::VersionStamp;

/// One node's cached layout, stored by [`LayoutCache`](super::LayoutCache)
/// (spec §4.3).
///
/// It records the [`BoxConstraints`] the node was measured under together with
/// the resulting [`LayoutResult`], an optional arranged [`LayoutBox`], and the
/// [`VersionStamp`] at which it was produced. The stored constraints are the
/// cache key for the **early-exit on stable geometry**: a later measure request
/// hits only when its constraints equal the ones recorded here.
///
/// Because [`BoxConstraints`], [`LayoutResult`], and [`LayoutBox`] hold `f32`,
/// this type is `PartialEq` but not `Eq`.
#[derive(Debug, Clone, PartialEq)]
pub struct CacheEntry {
    /// The constraints the node was measured under.
    constraints: BoxConstraints,
    /// The measured result under those constraints.
    result: LayoutResult,
    /// The arranged box, if the node has been arranged since it was measured.
    arranged: Option<LayoutBox>,
    /// The version at which this entry was produced.
    version: VersionStamp,
}

impl CacheEntry {
    /// Creates an entry for a measure produced under `constraints` at `version`,
    /// with no arranged box yet.
    #[must_use]
    pub const fn new(
        constraints: BoxConstraints,
        result: LayoutResult,
        version: VersionStamp,
    ) -> Self {
        Self {
            constraints,
            result,
            arranged: None,
            version,
        }
    }

    /// The constraints this entry was measured under (the cache key).
    #[must_use]
    pub const fn constraints(&self) -> BoxConstraints {
        self.constraints
    }

    /// The measured result.
    #[must_use]
    pub const fn result(&self) -> LayoutResult {
        self.result
    }

    /// The arranged box, if one has been recorded.
    #[must_use]
    pub const fn arranged(&self) -> Option<LayoutBox> {
        self.arranged
    }

    /// The version at which this entry was produced.
    #[must_use]
    pub const fn version(&self) -> VersionStamp {
        self.version
    }

    /// Records the arranged box for this entry.
    pub fn set_arranged(&mut self, arranged: LayoutBox) {
        self.arranged = Some(arranged);
    }

    /// Whether this entry's stored constraints exactly equal `constraints`.
    ///
    /// Equality is the exact `f32` `PartialEq` comparison: a cache hit requires
    /// identical constraints, so a measure under any different constraints
    /// misses.
    #[must_use]
    pub fn matches_constraints(&self, constraints: BoxConstraints) -> bool {
        self.constraints == constraints
    }
}

#[cfg(test)]
mod tests {
    use super::CacheEntry;
    use crate::model::{LayoutBox, LayoutResult};
    use obscur_geometry::{BoxConstraints, Size};
    use obscur_identity::VersionStamp;

    fn constraints(w: f32, h: f32) -> BoxConstraints {
        BoxConstraints::tight(Size::new(w, h))
    }

    #[test]
    fn new_stores_fields_and_no_arrangement() {
        let e = CacheEntry::new(
            constraints(10.0, 10.0),
            LayoutResult::new(Size::new(10.0, 10.0)),
            VersionStamp::from_raw(3),
        );
        assert_eq!(e.result().size(), Size::new(10.0, 10.0));
        assert_eq!(e.version(), VersionStamp::from_raw(3));
        assert_eq!(e.arranged(), None);
    }

    #[test]
    fn matches_constraints_is_exact() {
        let e = CacheEntry::new(
            constraints(10.0, 10.0),
            LayoutResult::default(),
            VersionStamp::default(),
        );
        assert!(e.matches_constraints(constraints(10.0, 10.0)));
        assert!(!e.matches_constraints(constraints(10.0, 11.0)));
    }

    #[test]
    fn set_arranged_records_box() {
        let mut e = CacheEntry::new(
            constraints(10.0, 10.0),
            LayoutResult::default(),
            VersionStamp::default(),
        );
        let b = LayoutBox::from_size(Size::new(10.0, 10.0));
        e.set_arranged(b);
        assert_eq!(e.arranged(), Some(b));
    }
}
