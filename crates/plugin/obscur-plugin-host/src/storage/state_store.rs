use std::io;

use obscur_plugin_model::manifest::VersionReq;

use super::{PluginStorage, StorageNamespace};

/// Host persistence port for versioned plugin state.
pub trait StateStore {
    /// Loads the latest snapshot in the given namespace, if one exists.
    fn load(&self, namespace: &StorageNamespace) -> io::Result<Option<PluginStorage>>;

    /// Atomically stores the candidate if the current snapshot matches `expected`.
    ///
    /// Use `None` to create state only if it is absent. An expected snapshot
    /// must belong to the candidate's namespace. Returns false on conflict.
    fn compare_exchange(
        &self,
        expected: Option<&PluginStorage>,
        candidate: &PluginStorage,
    ) -> io::Result<bool>;
}

/// Loads committed state and checks the returned namespace and accepted schema before exposing its bytes.
pub fn load_compatible_state(
    store: &(impl StateStore + ?Sized),
    namespace: &StorageNamespace,
    accepted: &VersionReq,
) -> io::Result<Option<PluginStorage>> {
    let snapshot = store.load(namespace)?;
    if let Some(snapshot) = &snapshot {
        if snapshot.namespace() != namespace {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "state namespace mismatch",
            ));
        }
        if !accepted.matches(snapshot.schema().version()) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "incompatible committed state schema",
            ));
        }
    }
    Ok(snapshot)
}
