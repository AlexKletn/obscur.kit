//! Host registry of scoped service grants and their lifetimes.

use std::{sync::Arc, time::Instant};

use obscur_plugin_model::{identity::PluginPrincipal, manifest::Version};

use super::{
    CapabilityAccessError, CapabilityHandle, ServiceScope, delegation_graph::DelegationGraph,
    grant::Grant,
};

/// Host registry of scoped service grants and their lifetimes.
#[derive(Default)]
pub struct CapabilityIssuer {
    identity: Arc<()>,
    next: u64,
    pub(super) grants: DelegationGraph,
}

impl CapabilityIssuer {
    /// Lists unexpired grants still held by this issuer, including delegated grants.
    /// This view describes the issuer's grants. Service and session availability are checked separately during dispatch.
    pub fn active_grants(
        &self,
    ) -> impl Iterator<Item = (&PluginPrincipal, &ServiceScope, Option<Instant>)> {
        let now = Instant::now();
        self.grants
            .grants()
            .filter(move |grant| grant.expires.is_none_or(|deadline| now < deadline))
            .map(|grant| (&grant.principal, &grant.scope, grant.expires))
    }

    /// Issues a handle after the trusted host has authorized the service scope.
    pub fn issue(
        &mut self,
        principal: PluginPrincipal,
        scope: ServiceScope,
        expires: Option<Instant>,
    ) -> Result<CapabilityHandle, CapabilityAccessError> {
        self.insert(Grant {
            principal,
            scope,
            expires,
            parent: None,
        })
    }

    /// Issues a host-authorized derived handle for the designated principal.
    ///
    /// The parent must be live and the new operation set cannot grow. With
    /// no explicit deadline the child inherits the parent's deadline.
    pub fn delegate(
        &mut self,
        parent: &CapabilityHandle,
        principal: PluginPrincipal,
        scope: ServiceScope,
        expires: Option<Instant>,
    ) -> Result<CapabilityHandle, CapabilityAccessError> {
        let original = self.live_grant(parent)?;
        if !original.scope.includes(&scope)
            || expires
                .zip(original.expires)
                .is_some_and(|(child, parent)| child > parent)
        {
            return Err(CapabilityAccessError::Denied);
        }
        let expires = expires.or(original.expires);
        self.insert(Grant {
            principal,
            scope,
            expires,
            parent: Some(parent.id),
        })
    }

    /// Checks a call against the authenticated principal and actual service operation.
    pub fn authorize(
        &self,
        handle: &CapabilityHandle,
        caller: &PluginPrincipal,
        service: &str,
        version: &Version,
        operation: &str,
    ) -> Result<(), CapabilityAccessError> {
        let grant = self.live_grant(handle)?;
        if &grant.principal == caller && grant.scope.permits(service, version, operation) {
            Ok(())
        } else {
            Err(CapabilityAccessError::Denied)
        }
    }

    pub(super) fn owns(&self, handle: &CapabilityHandle) -> bool {
        handle.belongs_to(&self.identity)
    }

    fn live_grant(&self, handle: &CapabilityHandle) -> Result<&Grant, CapabilityAccessError> {
        self.grants
            .get(handle.id)
            .filter(|grant| {
                self.owns(handle)
                    && grant
                        .expires
                        .is_none_or(|deadline| Instant::now() < deadline)
            })
            .ok_or(CapabilityAccessError::Denied)
    }

    pub(crate) fn grant_scope(
        &self,
        handle: &CapabilityHandle,
    ) -> Option<(&ServiceScope, Option<Instant>)> {
        self.live_grant(handle)
            .ok()
            .map(|grant| (&grant.scope, grant.expires))
    }

    fn insert(&mut self, grant: Grant) -> Result<CapabilityHandle, CapabilityAccessError> {
        if grant
            .expires
            .is_some_and(|deadline| Instant::now() >= deadline)
        {
            return Err(CapabilityAccessError::Denied);
        }
        let id = self.next;
        self.next = id.checked_add(1).ok_or(CapabilityAccessError::Exhausted)?;
        self.grants.insert(id, grant);
        Ok(CapabilityHandle::new(Arc::clone(&self.identity), id))
    }
}
