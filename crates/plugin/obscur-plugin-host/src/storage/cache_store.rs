use std::io;

use super::{StateStore, StorageNamespace};

/// Host port for evicting the rebuildable cache of a plugin installation.
pub trait CacheStore: StateStore {
    /// Clears only the cache of the identified installation, regardless of
    /// the scope of the supplied namespace. Returns false when it was absent.
    fn clear_cache(&self, namespace: &StorageNamespace) -> io::Result<bool>;
}
