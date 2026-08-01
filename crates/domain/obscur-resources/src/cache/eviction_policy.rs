#![doc = include_str!("eviction_policy.doc/_module.md")]

use crate::cache::CacheEntry;

#[doc = include_str!("eviction_policy.doc/EvictionPolicy.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EvictionPolicy {
    #[doc = include_str!("eviction_policy.doc/Lru.md")]
    Lru,
    #[doc = include_str!("eviction_policy.doc/Lfu.md")]
    Lfu,
    #[doc = include_str!("eviction_policy.doc/VisibilityAware.md")]
    VisibilityAware,
}

impl EvictionPolicy {
    #[doc = include_str!("eviction_policy.doc/choose_victim.md")]
    #[must_use]
    pub fn choose_victim<'a>(&self, entries: &'a [CacheEntry]) -> Option<&'a CacheEntry> {
        entries.iter().min_by(|a, b| match self {
            Self::Lru => a
                .last_access()
                .cmp(&b.last_access())
                .then_with(|| a.key().cmp(&b.key())),
            Self::Lfu => a
                .hits()
                .cmp(&b.hits())
                .then_with(|| a.last_access().cmp(&b.last_access()))
                .then_with(|| a.key().cmp(&b.key())),
            Self::VisibilityAware => a
                .visibility()
                .cmp(&b.visibility())
                .then_with(|| a.last_access().cmp(&b.last_access()))
                .then_with(|| a.key().cmp(&b.key())),
        })
    }
}

impl Default for EvictionPolicy {
    #[doc = include_str!("eviction_policy.doc/default.md")]
    fn default() -> Self {
        Self::VisibilityAware
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::{CacheKey, VisibilityPriority};
    use crate::state::ResourceVersion;
    use obscur_identity::ResourceId;

    fn entry(raw: u64, bytes: u64) -> CacheEntry {
        CacheEntry::new(
            CacheKey::for_resource(ResourceId::from_raw(raw)),
            bytes,
            ResourceVersion::initial(),
        )
    }

    #[test]
    fn empty_slice_has_no_victim() {
        assert!(EvictionPolicy::Lru.choose_victim(&[]).is_none());
        assert!(EvictionPolicy::Lfu.choose_victim(&[]).is_none());
        assert!(EvictionPolicy::VisibilityAware.choose_victim(&[]).is_none());
    }

    #[test]
    fn lru_picks_smallest_last_access() {
        let mut a = entry(1, 0);
        a.touch(30);
        let mut b = entry(2, 0);
        b.touch(10);
        let mut c = entry(3, 0);
        c.touch(20);
        let expected = b.key();
        let entries = [a, b, c];
        let victim = EvictionPolicy::Lru.choose_victim(&entries).unwrap();
        assert_eq!(victim.key(), expected);
    }

    #[test]
    fn lru_breaks_ties_by_key() {
        // Both untouched (last_access 0); the smaller key wins deterministically,
        // regardless of slice order.
        let a = entry(5, 0);
        let b = entry(2, 0);
        let expected = b.key();
        let entries = [a, b];
        let victim = EvictionPolicy::Lru.choose_victim(&entries).unwrap();
        assert_eq!(victim.key(), expected);
    }

    #[test]
    fn lfu_picks_smallest_hits_then_last_access() {
        let mut a = entry(1, 0);
        a.touch(5); // hits = 1
        let mut b = entry(2, 0);
        b.touch(1);
        b.touch(2); // hits = 2
        let c = entry(3, 0); // hits = 0
        let expected = c.key();
        let entries = [a, b, c];
        let victim = EvictionPolicy::Lfu.choose_victim(&entries).unwrap();
        assert_eq!(victim.key(), expected);
    }

    #[test]
    fn lfu_breaks_tie_on_hits_by_last_access() {
        let mut a = entry(1, 0);
        a.touch(40); // hits 1, last_access 40
        let mut b = entry(2, 0);
        b.touch(10); // hits 1, last_access 10
        let expected = b.key();
        let entries = [a, b];
        let victim = EvictionPolicy::Lfu.choose_victim(&entries).unwrap();
        assert_eq!(victim.key(), expected);
    }

    #[test]
    fn visibility_aware_picks_least_visible() {
        let a = entry(1, 0).with_visibility(VisibilityPriority::Visible);
        let b = entry(2, 0).with_visibility(VisibilityPriority::Hidden);
        let c = entry(3, 0).with_visibility(VisibilityPriority::NearViewport);
        let expected = b.key();
        let entries = [a, b, c];
        let victim = EvictionPolicy::VisibilityAware
            .choose_victim(&entries)
            .unwrap();
        assert_eq!(victim.key(), expected);
    }

    #[test]
    fn visibility_aware_ties_fall_back_to_last_access() {
        let mut a = entry(1, 0).with_visibility(VisibilityPriority::Offscreen);
        a.touch(30);
        let mut b = entry(2, 0).with_visibility(VisibilityPriority::Offscreen);
        b.touch(5);
        let expected = b.key();
        let entries = [a, b];
        let victim = EvictionPolicy::VisibilityAware
            .choose_victim(&entries)
            .unwrap();
        assert_eq!(victim.key(), expected);
    }

    #[test]
    fn default_is_visibility_aware() {
        assert_eq!(EvictionPolicy::default(), EvictionPolicy::VisibilityAware);
    }
}
