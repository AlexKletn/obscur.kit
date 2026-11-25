use std::collections::HashMap;
use std::sync::Arc;

use crate::cache::CacheKey;
use crate::state::ResourceVersion;

use super::CacheStore;

#[doc = include_str!("../../docs/api/port/in_memory_cache_store.doc/InMemoryCacheStore.md")]
#[derive(Debug, Default, Clone)]
pub struct InMemoryCacheStore {
    entries: HashMap<CacheKey, (ResourceVersion, Arc<[u8]>)>,
}

impl InMemoryCacheStore {
    /// Creates an empty store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl CacheStore for InMemoryCacheStore {
    fn get(&self, key: &CacheKey, version: ResourceVersion) -> Option<Arc<[u8]>> {
        match self.entries.get(key) {
            Some((stored_version, bytes)) if *stored_version == version => Some(Arc::clone(bytes)),
            // Compatibility-aware (spec §5.20): a version mismatch or a missing
            // entry both yield None.
            _ => None,
        }
    }

    fn put(&mut self, key: CacheKey, version: ResourceVersion, bytes: Arc<[u8]>) {
        self.entries.insert(key, (version, bytes));
    }

    fn contains(&self, key: &CacheKey) -> bool {
        self.entries.contains_key(key)
    }

    fn remove(&mut self, key: &CacheKey) -> bool {
        self.entries.remove(key).is_some()
    }

    fn clear(&mut self) {
        self.entries.clear();
    }

    fn len(&self) -> usize {
        self.entries.len()
    }

    fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_identity::ResourceId;

    fn key(raw: u64) -> CacheKey {
        CacheKey::for_resource(ResourceId::from_raw(raw))
    }

    #[test]
    fn get_returns_bytes_for_matching_version() {
        let mut store = InMemoryCacheStore::new();
        let version = ResourceVersion::initial();
        store.put(key(1), version, Arc::from(&b"artifact"[..]));

        let fetched = store.get(&key(1), version).expect("matching version hits");
        assert_eq!(&*fetched, b"artifact");
    }

    #[test]
    fn get_returns_none_for_mismatched_version() {
        let mut store = InMemoryCacheStore::new();
        let version = ResourceVersion::initial();
        store.put(key(1), version, Arc::from(&b"artifact"[..]));

        // A newer version supersedes the stored one: compatibility-aware miss.
        assert_eq!(store.get(&key(1), version.next()), None);
    }

    #[test]
    fn get_returns_none_for_absent_key() {
        let store = InMemoryCacheStore::new();
        assert_eq!(store.get(&key(9), ResourceVersion::initial()), None);
    }

    #[test]
    fn contains_ignores_version() {
        let mut store = InMemoryCacheStore::new();
        store.put(key(2), ResourceVersion::initial(), Arc::from(&[1u8][..]));
        assert!(store.contains(&key(2)));
        assert!(!store.contains(&key(3)));
    }

    #[test]
    fn remove_reports_whether_present() {
        let mut store = InMemoryCacheStore::new();
        store.put(key(4), ResourceVersion::initial(), Arc::from(&[1u8][..]));
        assert!(store.remove(&key(4)));
        assert!(!store.remove(&key(4)));
        assert!(!store.contains(&key(4)));
    }

    #[test]
    fn clear_empties_the_store() {
        let mut store = InMemoryCacheStore::new();
        store.put(key(5), ResourceVersion::initial(), Arc::from(&[1u8][..]));
        store.put(key(6), ResourceVersion::initial(), Arc::from(&[2u8][..]));
        assert_eq!(store.len(), 2);
        store.clear();
        assert!(store.is_empty());
        assert_eq!(store.len(), 0);
    }

    #[test]
    fn len_and_is_empty_track_entries() {
        let mut store = InMemoryCacheStore::new();
        assert!(store.is_empty());
        assert_eq!(store.len(), 0);
        store.put(key(7), ResourceVersion::initial(), Arc::from(&[1u8][..]));
        assert!(!store.is_empty());
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn put_replaces_existing_entry() {
        let mut store = InMemoryCacheStore::new();
        let v1 = ResourceVersion::initial();
        let v2 = v1.next();
        store.put(key(8), v1, Arc::from(&b"old"[..]));
        store.put(key(8), v2, Arc::from(&b"new"[..]));
        assert_eq!(store.len(), 1);
        assert_eq!(store.get(&key(8), v1), None);
        assert_eq!(
            &*store.get(&key(8), v2).expect("newest version hits"),
            b"new"
        );
    }
}
