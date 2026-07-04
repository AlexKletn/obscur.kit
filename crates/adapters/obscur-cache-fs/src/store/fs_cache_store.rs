//! A filesystem-backed [`CacheStore`] adapter (spec §5.20 persistent derived
//! cache, §2.7 "persistent cache adapter").
//!
//! Spec §5.20 describes a store of expensive *derived* artifacts (compiled
//! shaders, rasterised glyphs, prepared display lists) that survives across runs:
//! it is **versioned**, **content-addressed where useful**, **compatibility-aware**
//! (a stored entry is served only while its version still matches), and
//! **disposable** — every entry is derived data that can be safely deleted and
//! recomputed. The kit's core performs no filesystem access itself; an interface
//! adapter does (spec §2.7). This module is that adapter: [`FsCacheStore`]
//! implements the pure [`CacheStore`] port using only `std`, persisting each entry
//! as a single file under a cache directory.
//!
//! # On-disk format
//!
//! One file per entry. The filename is content-addressed from the key so distinct
//! keys never collide:
//!
//! ```text
//! <resource-id>-<variant>.oc
//! ```
//!
//! where `<resource-id>` is [`CacheKey::resource`]'s raw `u64` and `<variant>` is
//! [`CacheKey::variant`]. The file contents are the 8-byte little-endian version
//! stamp followed by the raw artifact bytes:
//!
//! ```text
//! [ version_stamp: u64 LE (8 bytes) ][ artifact bytes … ]
//! ```
//!
//! [`CacheStore::get`] reads the 8-byte header and returns the payload only when it
//! equals the requested version (§5.20 compatibility-awareness); a stale header, a
//! truncated file, or any I/O error is treated as a miss.
//!
//! The whole directory is safe to delete: every `.oc` file is derived data. This
//! adapter only ever touches its own `.oc` files, leaving any other files in the
//! directory untouched.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use obscur_resources::cache::CacheKey;
use obscur_resources::port::CacheStore;
use obscur_resources::state::ResourceVersion;

/// The number of leading bytes of an entry file that hold the little-endian
/// version stamp (spec §5.20 versioned / compatibility-aware).
const VERSION_HEADER_LEN: usize = 8;

/// The filename extension marking a file as one of this adapter's cache entries.
/// Only files with this extension are read, counted, or cleared, so unrelated
/// files sharing the cache directory are left alone.
const ENTRY_EXTENSION: &str = "oc";

/// A filesystem-backed [`CacheStore`] that persists each derived artifact as a
/// single file under a cache directory (spec §5.20, §2.7).
///
/// Each entry file is named from its [`CacheKey`] (`<resource>-<variant>.oc`) and
/// stores the artifact's [`ResourceVersion`] stamp inline ahead of the bytes, so a
/// [`CacheStore::get`] served by a *different* process or run still enforces the
/// §5.20 compatibility check. Because every entry is derived data, the entire
/// directory is disposable: [`CacheStore::clear`], [`CacheStore::remove`], or
/// deleting the directory outright is always safe.
///
/// Writes are best-effort. `put` swaps a fully written temp file into place with a
/// rename, so a crash never leaves a half-written entry, and any I/O failure during
/// `put` is swallowed — a failed write simply means a later `get` misses and the
/// artifact is recomputed.
#[derive(Debug, Clone)]
pub struct FsCacheStore {
    /// The cache directory holding the `.oc` entry files.
    root: PathBuf,
}

impl FsCacheStore {
    /// Creates a store rooted at `root`, creating the directory (and any missing
    /// parents) if it does not already exist.
    ///
    /// A pre-existing directory is fine; only a genuine creation failure other than
    /// "already exists" is surfaced — and even that is swallowed, since the store is
    /// best-effort and a later `get`/`put` against an absent directory simply
    /// misses.
    #[must_use]
    pub fn new(root: impl Into<PathBuf>) -> Self {
        let root = root.into();
        if let Err(err) = std::fs::create_dir_all(&root) {
            debug_assert!(
                err.kind() == std::io::ErrorKind::AlreadyExists,
                "cache dir create_dir_all failed: {err}"
            );
        }
        Self { root }
    }

    /// Returns the cache directory this store writes its entry files into.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Returns the absolute path of the entry file backing `key`.
    ///
    /// The name (`<resource>-<variant>.oc`) is derived entirely from the key, so it
    /// is stable across runs and collision-free between distinct keys.
    fn entry_path(&self, key: &CacheKey) -> PathBuf {
        self.root.join(format!(
            "{}-{}.{ENTRY_EXTENSION}",
            key.resource().raw(),
            key.variant()
        ))
    }

    /// Returns whether `path`'s name marks it as one of this adapter's entry files.
    fn is_entry(path: &Path) -> bool {
        path.extension().is_some_and(|ext| ext == ENTRY_EXTENSION)
    }
}

impl CacheStore for FsCacheStore {
    fn get(&self, key: &CacheKey, version: ResourceVersion) -> Option<Arc<[u8]>> {
        let bytes = std::fs::read(self.entry_path(key)).ok()?;
        // A well-formed entry is at least the version header; a shorter file is
        // corrupt and treated as a miss.
        let (header, payload) = bytes.split_at_checked(VERSION_HEADER_LEN)?;
        let stored = u64::from_le_bytes(header.try_into().ok()?);
        // Compatibility-aware (spec §5.20): a version mismatch is stale, so it
        // yields None rather than incompatible bytes.
        if stored != version.stamp().raw() {
            return None;
        }
        Some(Arc::from(payload))
    }

    fn put(&mut self, key: CacheKey, version: ResourceVersion, bytes: Arc<[u8]>) {
        let final_path = self.entry_path(&key);
        let tmp_path = final_path.with_extension("tmp");

        // Assemble the header + payload, then write it whole to a temp file and
        // rename it over the destination. The rename is atomic on the same
        // filesystem, so a crash mid-write leaves either the old entry or the new
        // one — never a torn file. Every failure is swallowed: a cache miss and
        // recompute is the correct fallback (spec §5.20 disposable / best-effort).
        let mut buffer = Vec::with_capacity(VERSION_HEADER_LEN + bytes.len());
        buffer.extend_from_slice(&version.stamp().raw().to_le_bytes());
        buffer.extend_from_slice(&bytes);

        if std::fs::write(&tmp_path, &buffer).is_err() {
            let _ = std::fs::remove_file(&tmp_path);
            return;
        }
        if std::fs::rename(&tmp_path, &final_path).is_err() {
            let _ = std::fs::remove_file(&tmp_path);
        }
    }

    fn contains(&self, key: &CacheKey) -> bool {
        self.entry_path(key).try_exists().unwrap_or(false)
    }

    fn remove(&mut self, key: &CacheKey) -> bool {
        // Ok => the entry existed and was deleted; NotFound (or any other error)
        // => nothing removed.
        std::fs::remove_file(self.entry_path(key)).is_ok()
    }

    fn clear(&mut self) {
        // Only this adapter's own `.oc` files are removed; unrelated files sharing
        // the directory are left alone (spec §5.20 disposable). Errors are
        // swallowed — a residual file just means a later `get` may still hit.
        let Ok(entries) = std::fs::read_dir(&self.root) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if Self::is_entry(&path) {
                let _ = std::fs::remove_file(&path);
            }
        }
    }

    fn len(&self) -> usize {
        // Count only our own `.oc` entry files; an unreadable directory reads as
        // empty.
        let Ok(entries) = std::fs::read_dir(&self.root) else {
            return 0;
        };
        entries
            .flatten()
            .filter(|entry| Self::is_entry(&entry.path()))
            .count()
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    use obscur_identity::ResourceId;

    /// Process-unique counter so concurrent tests never collide on a temp name.
    static COUNTER: AtomicU32 = AtomicU32::new(0);

    /// Creates a fresh, empty temp directory unique to this process and test.
    fn unique_temp_dir() -> PathBuf {
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("obscur-cache-fs-{}-{n}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    /// Removes a temp directory tree, ignoring absence.
    fn cleanup(dir: &Path) {
        let _ = std::fs::remove_dir_all(dir);
    }

    fn key(raw: u64) -> CacheKey {
        CacheKey::for_resource(ResourceId::from_raw(raw))
    }

    #[test]
    fn put_then_get_matching_version_returns_bytes() {
        let dir = unique_temp_dir();
        let mut store = FsCacheStore::new(&dir);
        let version = ResourceVersion::initial();

        store.put(key(1), version, Arc::from(&b"artifact bytes"[..]));
        let fetched = store.get(&key(1), version).expect("matching version hits");
        assert_eq!(&*fetched, b"artifact bytes");

        cleanup(&dir);
    }

    #[test]
    fn get_with_different_version_is_a_miss() {
        let dir = unique_temp_dir();
        let mut store = FsCacheStore::new(&dir);
        let version = ResourceVersion::initial();

        store.put(key(2), version, Arc::from(&b"artifact"[..]));
        // Compatibility-aware (spec §5.20): a newer version supersedes the stored
        // one, so the stale bytes are withheld.
        assert_eq!(store.get(&key(2), version.next()), None);

        cleanup(&dir);
    }

    #[test]
    fn contains_true_after_put() {
        let dir = unique_temp_dir();
        let mut store = FsCacheStore::new(&dir);

        assert!(!store.contains(&key(3)));
        store.put(key(3), ResourceVersion::initial(), Arc::from(&[7u8][..]));
        assert!(store.contains(&key(3)));

        cleanup(&dir);
    }

    #[test]
    fn remove_reports_whether_present() {
        let dir = unique_temp_dir();
        let mut store = FsCacheStore::new(&dir);

        store.put(key(4), ResourceVersion::initial(), Arc::from(&[1u8][..]));
        assert!(store.remove(&key(4)));
        assert!(!store.remove(&key(4)));
        assert!(!store.contains(&key(4)));

        cleanup(&dir);
    }

    #[test]
    fn second_put_at_new_version_replaces_entry() {
        let dir = unique_temp_dir();
        let mut store = FsCacheStore::new(&dir);
        let v1 = ResourceVersion::initial();
        let v2 = v1.next();

        store.put(key(5), v1, Arc::from(&b"old"[..]));
        store.put(key(5), v2, Arc::from(&b"new"[..]));

        // The stale version no longer hits; the new one does.
        assert_eq!(store.get(&key(5), v1), None);
        assert_eq!(&*store.get(&key(5), v2).expect("new version hits"), b"new");
        // Replacement, not accumulation.
        assert_eq!(store.len(), 1);

        cleanup(&dir);
    }

    #[test]
    fn clear_empties_the_store() {
        let dir = unique_temp_dir();
        let mut store = FsCacheStore::new(&dir);
        store.put(key(6), ResourceVersion::initial(), Arc::from(&[1u8][..]));
        store.put(key(7), ResourceVersion::initial(), Arc::from(&[2u8][..]));
        assert_eq!(store.len(), 2);

        store.clear();
        assert!(store.is_empty());
        assert_eq!(store.len(), 0);

        cleanup(&dir);
    }

    #[test]
    fn persists_across_two_store_instances_on_same_dir() {
        let dir = unique_temp_dir();
        let version = ResourceVersion::initial();

        // First instance writes; it is then dropped.
        {
            let mut writer = FsCacheStore::new(&dir);
            writer.put(key(8), version, Arc::from(&b"persisted"[..]));
        }

        // A fresh instance over the SAME directory reads what the first wrote —
        // the artifact survives across "runs" (spec §5.20 persistent).
        let reader = FsCacheStore::new(&dir);
        assert!(reader.contains(&key(8)));
        let fetched = reader.get(&key(8), version).expect("persisted entry hits");
        assert_eq!(&*fetched, b"persisted");
        // Version-awareness still holds against the persisted header.
        assert_eq!(reader.get(&key(8), version.next()), None);

        cleanup(&dir);
    }

    #[test]
    fn len_counts_only_entry_files() {
        let dir = unique_temp_dir();
        let mut store = FsCacheStore::new(&dir);
        store.put(key(9), ResourceVersion::initial(), Arc::from(&[1u8][..]));
        assert_eq!(store.len(), 1);

        // A stray non-entry file must not be counted or cleared.
        std::fs::write(dir.join("stray.txt"), b"not ours").expect("write stray file");
        assert_eq!(store.len(), 1);

        store.clear();
        assert_eq!(store.len(), 0);
        assert!(
            dir.join("stray.txt").exists(),
            "clear left foreign file alone"
        );

        cleanup(&dir);
    }

    #[test]
    fn truncated_entry_file_is_a_miss() {
        let dir = unique_temp_dir();
        let store = FsCacheStore::new(&dir);
        // A file shorter than the 8-byte version header is corrupt → miss, no panic.
        std::fs::write(store.entry_path(&key(10)), b"abc").expect("write short file");
        assert_eq!(store.get(&key(10), ResourceVersion::initial()), None);

        cleanup(&dir);
    }
}
