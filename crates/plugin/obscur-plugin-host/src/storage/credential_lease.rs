use std::fmt;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::Instant;

use obscur_plugin_model::identity::PluginPrincipal;

use super::CredentialScope;

/// Opaque authorization to use a stored credential until a deadline.
#[derive(Clone)]
pub struct CredentialLease {
    issuer: Arc<()>,
    principal: PluginPrincipal,
    key: Box<str>,
    scope: CredentialScope,
    expires: Instant,
    revoked: Arc<AtomicBool>,
}

impl fmt::Debug for CredentialLease {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CredentialLease").finish_non_exhaustive()
    }
}

impl CredentialLease {
    pub(super) fn new(
        issuer: Arc<()>,
        principal: PluginPrincipal,
        key: Box<str>,
        scope: CredentialScope,
        expires: Instant,
    ) -> Self {
        Self {
            issuer,
            principal,
            key,
            scope,
            expires,
            revoked: Arc::new(AtomicBool::new(false)),
        }
    }

    pub(super) fn revoke_from(&self, issuer: &Arc<()>) -> bool {
        if !Arc::ptr_eq(issuer, &self.issuer) {
            return false;
        }
        self.revoked.store(true, Ordering::Release);
        true
    }

    pub(super) fn key_for(
        &self,
        issuer: &Arc<()>,
        caller: &PluginPrincipal,
        scope: &CredentialScope,
    ) -> Option<&str> {
        (Arc::ptr_eq(issuer, &self.issuer)
            && &self.principal == caller
            && &self.scope == scope
            && self.is_active())
        .then_some(&self.key)
    }

    pub(super) fn is_active(&self) -> bool {
        !self.revoked.load(Ordering::Acquire) && Instant::now() < self.expires
    }
}
