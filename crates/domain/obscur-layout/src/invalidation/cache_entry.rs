#![doc = include_str!("cache_entry.doc/_module.md")]

use crate::model::{LayoutBox, LayoutResult};
use obscur_geometry::BoxConstraints;
use obscur_identity::VersionStamp;

#[doc = include_str!("cache_entry.doc/CacheEntry.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct CacheEntry {
    #[doc = include_str!("cache_entry.doc/constraints.md")]
    constraints: BoxConstraints,
    #[doc = include_str!("cache_entry.doc/result.md")]
    result: LayoutResult,
    #[doc = include_str!("cache_entry.doc/arranged.md")]
    arranged: Option<LayoutBox>,
    #[doc = include_str!("cache_entry.doc/version.md")]
    version: VersionStamp,
}

impl CacheEntry {
    #[doc = include_str!("cache_entry.doc/new.md")]
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

    #[doc = include_str!("cache_entry.doc/constraints_2.md")]
    #[must_use]
    pub const fn constraints(&self) -> BoxConstraints {
        self.constraints
    }

    #[doc = include_str!("cache_entry.doc/result_2.md")]
    #[must_use]
    pub const fn result(&self) -> LayoutResult {
        self.result
    }

    #[doc = include_str!("cache_entry.doc/arranged_2.md")]
    #[must_use]
    pub const fn arranged(&self) -> Option<LayoutBox> {
        self.arranged
    }

    #[doc = include_str!("cache_entry.doc/version_2.md")]
    #[must_use]
    pub const fn version(&self) -> VersionStamp {
        self.version
    }

    #[doc = include_str!("cache_entry.doc/set_arranged.md")]
    pub fn set_arranged(&mut self, arranged: LayoutBox) {
        self.arranged = Some(arranged);
    }

    #[doc = include_str!("cache_entry.doc/matches_constraints.md")]
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
