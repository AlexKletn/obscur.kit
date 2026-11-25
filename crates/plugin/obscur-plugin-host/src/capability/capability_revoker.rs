//! Revokes grants and their descendants within a host registry.

use obscur_plugin_model::identity::PluginPrincipal;

use super::{CapabilityHandle, CapabilityIssuer};

/// Revokes grants and their descendants within a host registry.
#[derive(Debug)]
pub struct CapabilityRevoker;

impl CapabilityRevoker {
    /// Revokes the handle and all derived grants, returning the number removed.
    pub fn revoke(issuer: &mut CapabilityIssuer, handle: &CapabilityHandle) -> usize {
        if !issuer.owns(handle) {
            return 0;
        }
        issuer.grants.revoke_where(|id, _| id == handle.id)
    }

    /// Revokes all grants for a principal and their descendants.
    /// Call this when unloading the corresponding plugin instance.
    pub fn revoke_principal(issuer: &mut CapabilityIssuer, principal: &PluginPrincipal) -> usize {
        issuer
            .grants
            .revoke_where(|_, grant| &grant.principal == principal)
    }
}
