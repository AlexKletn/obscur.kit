use std::fs::{self, File, OpenOptions};
use std::io::{self, BufReader};
use std::path::{Path, PathBuf};

use obscur_plugin_model::state::StateScope;
use sha2::{Digest, Sha256};

use super::{CacheStore, PluginStorage, StateStore, StorageNamespace, StorageQuota};

mod state_record;
use state_record::StateRecord;

/// Atomic snapshot persistence for a dedicated host-managed directory.
#[derive(Debug)]
pub struct FilesystemStateStore {
    root: PathBuf,
    quota: StorageQuota,
}

impl FilesystemStateStore {
    /// Opens an existing directory protected from untrusted users and plugins.
    /// All writers must use this adapter and a filesystem supporting locks and
    /// atomic replacement.
    pub fn open(root: &Path, quota: StorageQuota) -> io::Result<Self> {
        let root = root.canonicalize()?;
        if !root.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::NotADirectory,
                "plugin state store",
            ));
        }
        Ok(Self { root, quota })
    }

    fn snapshot_path(&self, namespace: &StorageNamespace) -> PathBuf {
        let mut hash = Sha256::new();
        hash.update(namespace.plugin().as_str());
        // Plugin IDs cannot contain zero bytes, so the two parts are unambiguous.
        hash.update([0]);
        hash.update(namespace.installation());
        let installation = hash.finalize();
        match namespace.scope() {
            // Preserve the existing on-disk location for global snapshots.
            StateScope::Global => self.root.join(format!("{installation:x}.state")),
            StateScope::Instance(id) => {
                let instance = Sha256::digest(id.as_bytes());
                self.root
                    .join(format!("{installation:x}-{instance:x}.instance"))
            }
            StateScope::Cache => self.root.join(format!("{installation:x}.cache")),
        }
    }

    fn lock(&self) -> io::Result<File> {
        let lock = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(self.root.join(".state-lock"))?;
        lock.lock()?;
        Ok(lock)
    }
}

impl StateStore for FilesystemStateStore {
    fn load(&self, namespace: &StorageNamespace) -> io::Result<Option<PluginStorage>> {
        let file = match File::open(self.snapshot_path(namespace)) {
            Ok(file) => file,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error),
        };
        StateRecord::read(BufReader::new(file), namespace, self.quota).map(Some)
    }

    fn compare_exchange(
        &self,
        expected: Option<&PluginStorage>,
        candidate: &PluginStorage,
    ) -> io::Result<bool> {
        let namespace = candidate.namespace();
        if expected.is_some_and(|snapshot| snapshot.namespace() != namespace) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "snapshot namespace mismatch",
            ));
        }
        let record = StateRecord::prepare(candidate, self.quota)?;

        let _lock = self.lock()?;
        if self.load(namespace)?.as_ref() != expected {
            return Ok(false);
        }

        let mut file = tempfile::NamedTempFile::new_in(&self.root)?;
        record.write_to(&mut file)?;
        file.as_file().sync_all()?;
        file.persist(self.snapshot_path(namespace))
            .map_err(|error| error.error)?;
        Ok(true)
    }
}

impl CacheStore for FilesystemStateStore {
    fn clear_cache(&self, namespace: &StorageNamespace) -> io::Result<bool> {
        let cache = namespace.clone().with_scope(StateScope::Cache);
        let _lock = self.lock()?;
        match fs::remove_file(self.snapshot_path(&cache)) {
            Ok(()) => Ok(true),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
            Err(error) => Err(error),
        }
    }
}
