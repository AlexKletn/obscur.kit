use std::{io, sync::Arc, time::Instant};

use obscur_plugin_model::identity::PluginPrincipal;

use super::{CredentialLease, CredentialScope, SecureStorageBackend};

/// Host-owned service for issuing and resolving credential leases.
///
/// The host retains this service and makes only opaque leases available to
/// plugins. Only trusted host code may write or resolve secret bytes.
pub struct SecureStorageService<B> {
    backend: B,
    issuer: Arc<()>,
}

impl<B: SecureStorageBackend> SecureStorageService<B> {
    /// Creates an independent issuer backed by a protected store.
    pub fn new(backend: B) -> Self {
        Self {
            backend,
            issuer: Arc::new(()),
        }
    }

    /// Stores the result of a trusted host operation such as secret entry.
    pub fn store(&self, principal: &PluginPrincipal, key: &str, value: &[u8]) -> io::Result<()> {
        self.backend.store(principal, key, value)
    }

    /// Removes a stored credential.
    pub fn remove(&self, principal: &PluginPrincipal, key: &str) -> io::Result<bool> {
        self.backend.remove(principal, key)
    }

    /// Issues a lease after the host has authorized the principal and scope.
    #[must_use]
    pub fn issue(
        &self,
        principal: PluginPrincipal,
        key: impl Into<Box<str>>,
        scope: CredentialScope,
        expires: Instant,
    ) -> CredentialLease {
        CredentialLease::new(
            Arc::clone(&self.issuer),
            principal,
            key.into(),
            scope,
            expires,
        )
    }

    /// Invalidates this lease and all of its copies if this service issued it.
    pub fn revoke(&self, lease: &CredentialLease) -> bool {
        lease.revoke_from(&self.issuer)
    }

    /// Supplies secret bytes to a trusted host operation if the lease matches.
    ///
    /// The caller and scope must come from the host's authenticated context.
    /// Returns `None` when the authorized credential is absent.
    pub fn with_secret<T>(
        &self,
        lease: &CredentialLease,
        caller: &PluginPrincipal,
        scope: &CredentialScope,
        operation: impl FnOnce(&[u8]) -> T,
    ) -> io::Result<Option<T>> {
        let denied = || io::Error::new(io::ErrorKind::PermissionDenied, "credential lease denied");
        let key = lease
            .key_for(&self.issuer, caller, scope)
            .ok_or_else(denied)?;
        let secret = self.backend.load(caller, key)?;
        if !lease.is_active() {
            return Err(denied());
        }
        Ok(secret.as_deref().map(operation))
    }
}
