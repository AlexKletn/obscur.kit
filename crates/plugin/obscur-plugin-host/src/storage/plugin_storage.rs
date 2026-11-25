//! An in-memory snapshot of opaque plugin state and its schema.

use obscur_plugin_model::state::StateSchema;

use super::StorageNamespace;

/// An in-memory snapshot of opaque plugin state and its schema.
#[derive(Clone, PartialEq, Eq)]
pub struct PluginStorage {
    namespace: StorageNamespace,
    schema: StateSchema,
    bytes: Vec<u8>,
}

impl PluginStorage {
    /// Wraps a snapshot loaded from host-managed storage.
    #[must_use]
    pub const fn new(namespace: StorageNamespace, schema: StateSchema, bytes: Vec<u8>) -> Self {
        Self {
            namespace,
            schema,
            bytes,
        }
    }

    /// Returns the namespace associated with the snapshot.
    #[must_use]
    pub const fn namespace(&self) -> &StorageNamespace {
        &self.namespace
    }

    /// Returns the schema of the stored snapshot.
    #[must_use]
    pub const fn schema(&self) -> &StateSchema {
        &self.schema
    }

    /// Returns the opaque state bytes.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

impl std::fmt::Debug for PluginStorage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PluginStorage")
            .field("namespace", &self.namespace)
            .field("schema", &self.schema)
            .field("byte_count", &self.bytes.len())
            .finish()
    }
}
