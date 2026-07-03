//! The hexagonal port for the persistent derived-artifact cache (spec §5.20).
//!
//! This module defines [`CacheStore`], the port for the §5.20 persistent derived
//! cache: a store of derived artifacts (compiled shaders, rasterised glyphs,
//! prepared display lists) that survives across runs. Per spec §5.20 the store is
//! **versioned** and **compatibility-aware** (a stored entry is only served when
//! its version still matches), **content-addressed where useful** (via the
//! [`CacheKey`] variant), and **disposable** — every entry is derived data that
//! can be safely deleted and recomputed. Adapters back it with a real on-disk
//! store; the colocated [`InMemoryCacheStore`] is a volatile reference
//! implementation.

use std::collections::HashMap;
use std::sync::Arc;

use crate::cache::CacheKey;
use crate::state::ResourceVersion;

/// The hexagonal port for the persistent derived-artifact cache (spec §5.20).
///
/// An adapter implements this over a real persistent backend, and the domain
/// stores it as `Box<dyn CacheStore>`; the trait is therefore deliberately
/// **object-safe** — no generic methods and no `Self`-returning methods. Reads
/// take `&self` and writes take `&mut self`. Because every entry is derived data,
/// the store is disposable: dropping or [`Self::clear`]ing it is always safe
/// (spec §5.20).
pub trait CacheStore {
    /// Returns the bytes stored under `key`, but **only if** the stored entry's
    /// version equals `version` (spec §5.20 compatibility-awareness).
    ///
    /// A stored entry whose version differs from `version` is stale — derived
    /// under an incompatible input or format revision — so it yields `None`
    /// rather than incompatible bytes. `None` is also returned when no entry is
    /// stored under `key` at all.
    #[must_use]
    fn get(&self, key: &CacheKey, version: ResourceVersion) -> Option<Arc<[u8]>>;

    /// Stores `bytes` under `key`, tagged with `version` (spec §5.20 versioned).
    ///
    /// Any existing entry for `key` is replaced, so a re-derivation at a newer
    /// version supersedes the stale one.
    fn put(&mut self, key: CacheKey, version: ResourceVersion, bytes: Arc<[u8]>);

    /// Returns whether an entry is stored under `key`, regardless of its version.
    #[must_use]
    fn contains(&self, key: &CacheKey) -> bool;

    /// Removes the entry stored under `key`, returning whether one was present
    /// (spec §5.20 disposable).
    fn remove(&mut self, key: &CacheKey) -> bool;

    /// Removes every entry (spec §5.20 disposable / safe-to-delete).
    fn clear(&mut self);

    /// Returns the number of stored entries.
    #[must_use]
    fn len(&self) -> usize;

    /// Returns whether the store holds no entries.
    #[must_use]
    fn is_empty(&self) -> bool;
}

/// A volatile in-memory reference [`CacheStore`] (spec §5.20).
///
/// `InMemoryCacheStore` keeps entries in a [`HashMap`] keyed by [`CacheKey`],
/// storing each artifact's [`ResourceVersion`] alongside its bytes so
/// [`CacheStore::get`] can enforce the §5.20 compatibility check. It is a
/// reference implementation for tests and headless use: because it holds
/// everything in memory, nothing survives across runs — a real *persistent*
/// derived cache lives in an adapter. Its entries are derived data, so clearing
/// or dropping it is always safe.
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

    /// Compile-time proof the port is object-safe: adapters must be storable as
    /// `Box<dyn CacheStore>`.
    #[test]
    fn cache_store_is_object_safe() {
        let store: Box<dyn CacheStore> = Box::new(InMemoryCacheStore::new());
        assert!(store.is_empty());
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
        assert_eq!(&*store.get(&key(8), v2).expect("newest version hits"), b"new");
    }
}
