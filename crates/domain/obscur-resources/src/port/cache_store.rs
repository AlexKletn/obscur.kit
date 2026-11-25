#![doc = include_str!("../../docs/api/port/cache_store.doc/_module.md")]

use std::sync::Arc;

use crate::cache::CacheKey;
use crate::state::ResourceVersion;

#[doc = include_str!("../../docs/api/port/cache_store.doc/CacheStore.md")]
pub trait CacheStore {
    #[doc = include_str!("../../docs/api/port/cache_store.doc/get.md")]
    #[must_use]
    fn get(&self, key: &CacheKey, version: ResourceVersion) -> Option<Arc<[u8]>>;

    #[doc = include_str!("../../docs/api/port/cache_store.doc/put.md")]
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

#[cfg(test)]
mod tests {
    use super::super::InMemoryCacheStore;
    use super::*;

    /// Compile-time proof the port is object-safe: adapters must be storable as
    /// `Box<dyn CacheStore>`.
    #[test]
    fn cache_store_is_object_safe() {
        let store: Box<dyn CacheStore> = Box::new(InMemoryCacheStore::new());
        assert!(store.is_empty());
    }
}
