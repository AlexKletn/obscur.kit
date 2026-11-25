use std::io;

use obscur_plugin_model::identity::PluginPrincipal;

/// Port for an OS keyring or another protected secret store.
///
/// Backends must key records by the full principal and key, and must redact
/// secret bytes from backend error messages and diagnostics.
pub trait SecureStorageBackend {
    /// Reads a secret for a trusted host service.
    fn load(&self, principal: &PluginPrincipal, key: &str) -> io::Result<Option<Vec<u8>>>;

    /// Stores a secret supplied by the host.
    fn store(&self, principal: &PluginPrincipal, key: &str, value: &[u8]) -> io::Result<()>;

    /// Removes a secret if it is present.
    fn remove(&self, principal: &PluginPrincipal, key: &str) -> io::Result<bool>;
}
