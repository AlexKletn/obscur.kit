#![doc = include_str!("../../docs/api/store/fs_cache_store.doc/_module.md")]

use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use obscur_resources::cache::CacheKey;
use obscur_resources::port::CacheStore;
use obscur_resources::state::ResourceVersion;

/// The filename extension marking a file as one of this adapter's cache entries.
const ENTRY_EXTENSION: &str = "oc";

#[doc = include_str!("../../docs/api/store/fs_cache_store.doc/FsCacheStore.md")]
#[derive(Debug, Clone)]
pub struct FsCacheStore {
    /// The cache directory holding the `.oc` entry files.
    root: PathBuf,
}

impl FsCacheStore {
    #[doc = include_str!("../../docs/api/store/fs_cache_store.doc/new.md")]
    #[must_use]
    pub fn new(root: impl Into<PathBuf>) -> Self {
        let root = root.into();
        // An unavailable cache directory is equivalent to a cache miss.
        let _ = std::fs::create_dir_all(&root);
        Self { root }
    }

    /// Returns the cache directory this store writes its entry files into.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    #[doc = include_str!("../../docs/api/store/fs_cache_store.doc/entry_path.md")]
    fn entry_path(&self, key: &CacheKey) -> PathBuf {
        self.root.join(format!(
            "{}-{}.{ENTRY_EXTENSION}",
            key.resource().raw(),
            key.variant()
        ))
    }

    fn entry_paths(&self) -> impl Iterator<Item = PathBuf> {
        std::fs::read_dir(&self.root)
            .into_iter()
            .flatten()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_file()))
            .map(|entry| entry.path())
            .filter(|path| {
                path.extension().is_some_and(|ext| ext == ENTRY_EXTENSION)
                    && path
                        .file_stem()
                        .and_then(|stem| stem.to_str())
                        .and_then(|stem| stem.split_once('-'))
                        .is_some_and(|(resource, variant)| {
                            canonical_number(resource) && canonical_number(variant)
                        })
            })
    }

    fn publish(&self, key: &CacheKey, version: ResourceVersion, bytes: &[u8]) -> io::Result<()> {
        // Staging beside the destination gives each writer its own file on the same filesystem.
        let mut staged = match tempfile::NamedTempFile::new_in(&self.root) {
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                // A cache directory removed between writes can be recreated.
                std::fs::create_dir_all(&self.root)?;
                tempfile::NamedTempFile::new_in(&self.root)?
            }
            creation => creation?,
        };
        staged.write_all(&version.stamp().raw().to_le_bytes())?;
        staged.write_all(bytes)?;
        staged
            .persist(self.entry_path(key))
            .map_err(|error| error.error)?;
        Ok(())
    }
}

impl CacheStore for FsCacheStore {
    fn get(&self, key: &CacheKey, version: ResourceVersion) -> Option<Arc<[u8]>> {
        let path = self.entry_path(key);
        if !regular_file(&path) {
            return None;
        }
        // Read the header and payload from the same open file because a writer
        // may atomically replace the path between reads.
        let mut entry = std::fs::File::open(path).ok()?;
        let expected_header = version.stamp().raw().to_le_bytes();
        let mut header = [0; size_of::<u64>()];
        entry.read_exact(&mut header).ok()?;
        if header != expected_header {
            return None;
        }
        let mut payload = Vec::new();
        entry.read_to_end(&mut payload).ok()?;
        Some(Arc::from(payload))
    }

    fn put(&mut self, key: CacheKey, version: ResourceVersion, bytes: Arc<[u8]>) {
        // Cache entries are disposable, so a failed publish can be retried by recomputation.
        let _ = self.publish(&key, version, &bytes);
    }

    fn contains(&self, key: &CacheKey) -> bool {
        regular_file(&self.entry_path(key))
    }

    fn remove(&mut self, key: &CacheKey) -> bool {
        let path = self.entry_path(key);
        regular_file(&path) && std::fs::remove_file(path).is_ok()
    }

    fn clear(&mut self) {
        for path in self.entry_paths() {
            let _ = std::fs::remove_file(path);
        }
    }

    fn len(&self) -> usize {
        self.entry_paths().count()
    }

    fn is_empty(&self) -> bool {
        self.entry_paths().next().is_none()
    }
}

fn canonical_number(value: &str) -> bool {
    !value.is_empty()
        && (value.len() == 1 || !value.starts_with('0'))
        && value.bytes().all(|byte| byte.is_ascii_digit())
        && value.parse::<u64>().is_ok()
}

fn regular_file(path: &Path) -> bool {
    std::fs::symlink_metadata(path).is_ok_and(|metadata| metadata.is_file())
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_identity::{ResourceId, VersionStamp};

    fn key(raw: u64) -> CacheKey {
        CacheKey::for_resource(ResourceId::from_raw(raw))
    }

    #[test]
    fn clearing_ignores_other_oc_files_and_non_file_entries() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = FsCacheStore::new(dir.path());
        let version = ResourceVersion::initial();
        let foreign = dir.path().join("notes.oc");
        std::fs::write(&foreign, b"unrelated").unwrap();
        let noncanonical_entries = ["01-0.oc", "1-+0.oc", "18446744073709551616-0.oc"];
        for name in noncanonical_entries {
            std::fs::write(dir.path().join(name), b"unrelated").unwrap();
        }
        let directory_key = key(2);
        let directory = store.entry_path(&directory_key);
        std::fs::create_dir(&directory).unwrap();
        store.put(key(1), version, Arc::from(&b"entry"[..]));

        assert_eq!(store.len(), 1);
        assert!(!store.contains(&directory_key));
        assert!(!store.remove(&directory_key));
        assert!(store.get(&directory_key, version).is_none());
        store.clear();
        assert!(store.is_empty());
        assert_eq!(std::fs::read(foreign).unwrap(), b"unrelated");
        for name in noncanonical_entries {
            assert_eq!(std::fs::read(dir.path().join(name)).unwrap(), b"unrelated");
        }
        assert!(directory.is_dir());
    }

    #[test]
    fn failed_publication_cleans_staging_and_leaves_other_files_alone() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = FsCacheStore::new(dir.path());
        let key = key(1);
        let destination = store.entry_path(&key);
        std::fs::create_dir(&destination).unwrap();
        let other = destination.with_extension("tmp");
        std::fs::write(&other, b"another writer").unwrap();

        store.put(key, ResourceVersion::initial(), Arc::from(&b"entry"[..]));

        assert!(destination.is_dir());
        assert_eq!(std::fs::read(other).unwrap(), b"another writer");
        assert_eq!(dir.path().read_dir().unwrap().count(), 2);
    }

    #[test]
    fn unavailable_cache_directory_degrades_to_a_miss() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("blocked");
        std::fs::write(&path, b"not a directory").unwrap();
        let mut store = FsCacheStore::new(&path);
        let key = key(1);
        let version = ResourceVersion::initial();
        store.put(key, version, Arc::from(&b"entry"[..]));
        assert!(store.get(&key, version).is_none());
        assert_eq!(std::fs::read(path).unwrap(), b"not a directory");
    }

    #[test]
    fn publishing_recovers_if_the_cache_directory_was_removed() {
        let temporary = tempfile::tempdir().unwrap();
        let root = temporary.path().join("cache");
        let mut store = FsCacheStore::new(&root);
        std::fs::remove_dir(&root).unwrap();

        let version = ResourceVersion::initial();
        store.put(key(1), version, Arc::from(&b"recomputed"[..]));

        assert_eq!(
            store.get(&key(1), version).as_deref(),
            Some(&b"recomputed"[..])
        );
        assert_eq!(root.read_dir().unwrap().count(), 1);
    }

    #[test]
    fn concurrent_writers_publish_complete_entries_and_leave_no_staging_files() {
        let directory = tempfile::tempdir().unwrap();
        let store = FsCacheStore::new(directory.path());
        let start = std::sync::Barrier::new(2);
        let version = ResourceVersion::initial();
        let first = vec![0x5a; 8192];
        let second = vec![0x7b; 16384];

        std::thread::scope(|threads| {
            for payload in [&first, &second] {
                let mut writer = store.clone();
                let start = &start;
                threads.spawn(move || {
                    start.wait();
                    writer.put(key(1), version, Arc::from(payload.as_slice()));
                });
            }
        });

        let stored = store.get(&key(1), version).unwrap();
        assert!(stored.as_ref() == first || stored.as_ref() == second);
        assert_eq!(directory.path().read_dir().unwrap().count(), 1);
    }

    #[test]
    fn reads_and_writes_the_little_endian_cache_format() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = FsCacheStore::new(dir.path());
        let key = key(1);
        let version = ResourceVersion {
            stamp: VersionStamp::from_raw(0x0807_0605_0403_0201),
        };
        let entry = store.entry_path(&key);
        let encoded = b"\x01\x02\x03\x04\x05\x06\x07\x08entry";
        std::fs::write(&entry, encoded).unwrap();
        assert_eq!(store.get(&key, version).as_deref(), Some(&b"entry"[..]));
        store.put(key, version, Arc::from(&b"entry"[..]));
        assert_eq!(std::fs::read(entry).unwrap(), encoded);
    }

    #[test]
    fn put_then_get_matching_version_returns_bytes() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = FsCacheStore::new(dir.path());
        let version = ResourceVersion::initial();

        store.put(key(1), version, Arc::from(&b"artifact bytes"[..]));
        let fetched = store.get(&key(1), version).expect("matching version hits");
        assert_eq!(&*fetched, b"artifact bytes");
    }

    #[test]
    fn get_with_different_version_is_a_miss() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = FsCacheStore::new(dir.path());
        let version = ResourceVersion::initial();

        store.put(key(2), version, Arc::from(&b"artifact"[..]));
        // Compatibility-aware (spec §5.20): a newer version supersedes the stored
        // one, so the stale bytes are withheld.
        assert_eq!(store.get(&key(2), version.next()), None);
    }

    #[test]
    fn contains_true_after_put() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = FsCacheStore::new(dir.path());

        assert!(!store.contains(&key(3)));
        store.put(key(3), ResourceVersion::initial(), Arc::from(&[7u8][..]));
        assert!(store.contains(&key(3)));
    }

    #[test]
    fn remove_reports_whether_present() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = FsCacheStore::new(dir.path());

        store.put(key(4), ResourceVersion::initial(), Arc::from(&[1u8][..]));
        assert!(store.remove(&key(4)));
        assert!(!store.remove(&key(4)));
        assert!(!store.contains(&key(4)));
    }

    #[test]
    fn second_put_at_new_version_replaces_entry() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = FsCacheStore::new(dir.path());
        let v1 = ResourceVersion::initial();
        let v2 = v1.next();

        store.put(key(5), v1, Arc::from(&b"old"[..]));
        store.put(key(5), v2, Arc::from(&b"new"[..]));

        // The stale version no longer hits; the new one does.
        assert_eq!(store.get(&key(5), v1), None);
        assert_eq!(&*store.get(&key(5), v2).expect("new version hits"), b"new");
        // Replacement, not accumulation.
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn clear_empties_the_store() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = FsCacheStore::new(dir.path());
        store.put(key(6), ResourceVersion::initial(), Arc::from(&[1u8][..]));
        store.put(key(7), ResourceVersion::initial(), Arc::from(&[2u8][..]));
        assert_eq!(store.len(), 2);

        store.clear();
        assert!(store.is_empty());
        assert_eq!(store.len(), 0);
    }

    #[test]
    fn persists_across_two_store_instances_on_same_dir() {
        let dir = tempfile::tempdir().unwrap();
        let version = ResourceVersion::initial();

        // First instance writes; it is then dropped.
        {
            let mut writer = FsCacheStore::new(dir.path());
            writer.put(key(8), version, Arc::from(&b"persisted"[..]));
        }

        // A fresh instance over the SAME directory reads what the first wrote —
        // the artifact survives across "runs" (spec §5.20 persistent).
        let reader = FsCacheStore::new(dir.path());
        assert!(reader.contains(&key(8)));
        let fetched = reader.get(&key(8), version).expect("persisted entry hits");
        assert_eq!(&*fetched, b"persisted");
        // Version-awareness still holds against the persisted header.
        assert_eq!(reader.get(&key(8), version.next()), None);
    }

    #[test]
    fn len_counts_only_entry_files() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = FsCacheStore::new(dir.path());
        store.put(key(9), ResourceVersion::initial(), Arc::from(&[1u8][..]));
        assert_eq!(store.len(), 1);

        // A stray non-entry file must not be counted or cleared.
        std::fs::write(dir.path().join("stray.txt"), b"not ours").expect("write stray file");
        assert_eq!(store.len(), 1);

        store.clear();
        assert_eq!(store.len(), 0);
        assert!(
            dir.path().join("stray.txt").exists(),
            "clear left foreign file alone"
        );
    }

    #[test]
    fn truncated_entry_file_is_a_miss() {
        let dir = tempfile::tempdir().unwrap();
        let store = FsCacheStore::new(dir.path());
        // A file shorter than the 8-byte version header is corrupt → miss, no panic.
        std::fs::write(store.entry_path(&key(10)), b"abc").expect("write short file");
        assert_eq!(store.get(&key(10), ResourceVersion::initial()), None);
    }
}
