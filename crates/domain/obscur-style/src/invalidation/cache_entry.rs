//! A single cached computed style with its version: [`CacheEntry`] (§4.3).
//!
//! [`CacheEntry`] is the value stored per node in a [`crate::invalidation::StyleCache`]:
//! an owned [`ComputedStyle`] tagged with the [`VersionStamp`] it was computed
//! at. The stamp lets the cache reject stale hits — a lookup that asks for a newer
//! version than the entry holds misses, so recomputation is confined to genuinely
//! out-of-date nodes (§4.3).

use crate::computed::ComputedStyle;
use obscur_identity::VersionStamp;

/// A cached computed style tagged with the version it was produced at (§4.3).
///
/// Because [`ComputedStyle`] carries `f32` data, this derives [`PartialEq`] but
/// not [`Eq`].
#[derive(Debug, Clone, PartialEq)]
pub struct CacheEntry {
    version: VersionStamp,
    style: ComputedStyle,
}

impl CacheEntry {
    /// Creates an entry pairing `style` with the `version` it was computed at.
    #[must_use]
    pub fn new(version: VersionStamp, style: ComputedStyle) -> Self {
        Self { version, style }
    }

    /// Returns the version this entry was computed at.
    #[must_use]
    pub fn version(&self) -> VersionStamp {
        self.version
    }

    /// Returns the cached computed style.
    #[must_use]
    pub fn style(&self) -> &ComputedStyle {
        &self.style
    }

    /// Returns whether this entry was computed at exactly `version`.
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
