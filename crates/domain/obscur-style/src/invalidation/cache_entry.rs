#![doc = include_str!("cache_entry.doc/_module.md")]

use crate::computed::ComputedStyle;
use obscur_identity::VersionStamp;

#[doc = include_str!("cache_entry.doc/CacheEntry.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct CacheEntry {
    version: VersionStamp,
    style: ComputedStyle,
}

impl CacheEntry {
    #[doc = include_str!("cache_entry.doc/new.md")]
    #[must_use]
    pub fn new(version: VersionStamp, style: ComputedStyle) -> Self {
        Self { version, style }
    }

    #[doc = include_str!("cache_entry.doc/version.md")]
    #[must_use]
    pub fn version(&self) -> VersionStamp {
        self.version
    }

    #[doc = include_str!("cache_entry.doc/style.md")]
    #[must_use]
    pub fn style(&self) -> &ComputedStyle {
        &self.style
    }

    #[doc = include_str!("cache_entry.doc/is_current.md")]
    #[must_use]
    pub fn is_current(&self, version: VersionStamp) -> bool {
        self.version == version
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stores_version_and_style() {
        let style = ComputedStyle::new();
        let entry = CacheEntry::new(VersionStamp::from_raw(3), style);
        assert_eq!(entry.version(), VersionStamp::from_raw(3));
        assert!(entry.style().is_empty());
    }

    #[test]
    fn is_current_matches_exact_version() {
        let entry = CacheEntry::new(VersionStamp::from_raw(3), ComputedStyle::new());
        assert!(entry.is_current(VersionStamp::from_raw(3)));
        assert!(!entry.is_current(VersionStamp::from_raw(4)));
    }
}
