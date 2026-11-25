use std::{fmt, time::Instant};

use obscur_plugin_model::manifest::Version;

use crate::capability::{CapabilityHandle, CapabilityIssuer, CapabilityRevoker, ServiceScope};

use super::{ServiceRevoker, service_registry::ServiceKey};

/// Opaque handle tied to a service registration and a caller grant.
pub struct ServiceHandle {
    pub(super) key: ServiceKey,
    pub(super) lifecycle: ServiceRevoker,
    pub(super) capability: CapabilityHandle,
}

impl ServiceHandle {
    pub(super) fn active_scope<'a>(
        &self,
        issuer: &'a CapabilityIssuer,
    ) -> Option<(&'a ServiceScope, Option<Instant>)> {
        if self.lifecycle.is_revoked() {
            return None;
        }
        issuer.grant_scope(&self.capability)
    }

    pub(super) fn service_key(&self) -> (&str, &Version) {
        (&self.key.0, &self.key.1)
    }

    /// Revokes this handle and its delegates through the issuing host.
    pub fn revoke(&self, issuer: &mut CapabilityIssuer) -> usize {
        CapabilityRevoker::revoke(issuer, &self.capability)
    }
}

impl fmt::Debug for ServiceHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ServiceHandle").finish_non_exhaustive()
    }
}
