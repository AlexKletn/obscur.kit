use std::{
    collections::BTreeMap,
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};

use obscur_plugin_model::identity::PluginPrincipal;

use super::{
    ServiceCallLimit, ServiceHandle, ServiceRegistry, ServiceRevoker, ServiceSessionMetrics,
    ServiceSessionObserver, ServiceSessionRevoker, ServiceWireError,
};
use super::{call_rate_limiter::CallRateLimiter, service_revoker::until_cancelled};
use crate::capability::{CapabilityIssuer, ServiceScope};

use super::{
    ServiceSessionError, call_permit::CallPermit, session_counters::increment,
    session_grant::SessionGrant,
};

static NEXT_HANDLE_ID: AtomicU64 = AtomicU64::new(1);

/// Host-owned state for one authenticated transport connection.
///
/// Dropping the session releases its grants. Transport handle IDs are scoped
/// to the session, and are never reused while this host process is running.
pub struct ServiceSession {
    principal: PluginPrincipal,
    issuer: CapabilityIssuer,
    handles: BTreeMap<u64, SessionGrant>,
    max_handles: usize,
    call_limit: Option<CallRateLimiter>,
    max_in_flight: usize,
    observer: ServiceSessionObserver,
    slow_call_threshold: Option<Duration>,
    revoker: ServiceSessionRevoker,
}

impl ServiceSession {
    /// Samples the in-flight gauge and cumulative quota counters independently.
    #[must_use]
    pub fn metrics(&self) -> ServiceSessionMetrics {
        self.observer.metrics()
    }

    /// Returns a read-only observer that can be kept while the session is moved into a transport or executor.
    #[must_use]
    pub fn observer(&self) -> ServiceSessionObserver {
        self.observer.clone()
    }

    /// Lists authorized service scopes and expiration times for this authenticated session.
    /// Revoked sessions, service registrations, and handles are excluded, as are expired grants.
    pub fn active_scopes(&self) -> impl Iterator<Item = (&ServiceScope, Option<Instant>)> {
        self.handles.values().filter_map(|grant| {
            if self.revoker.is_revoked() || grant.revoker.is_revoked() {
                return None;
            }
            grant.handle.active_scope(&self.issuer)
        })
    }

    /// Opens a session for the identity authenticated by the adapter.
    #[must_use]
    pub fn new(principal: PluginPrincipal, max_handles: usize) -> Self {
        Self {
            principal,
            issuer: CapabilityIssuer::default(),
            handles: BTreeMap::new(),
            max_handles,
            call_limit: None,
            max_in_flight: usize::MAX,
            observer: ServiceSessionObserver::new(),
            slow_call_threshold: None,
            revoker: ServiceSessionRevoker::new(),
        }
    }

    /// Sets the rolling call budget for this session.
    #[must_use]
    pub fn with_call_limit(mut self, limit: ServiceCallLimit) -> Self {
        self.call_limit = Some(CallRateLimiter::new(limit));
        self
    }

    /// Caps simultaneously active transport calls. A limit of zero denies all.
    #[must_use]
    pub fn with_in_flight_limit(mut self, max_in_flight: usize) -> Self {
        self.max_in_flight = max_in_flight;
        self
    }

    /// Enables slow-call counting for this session. Time includes asynchronous waiting after admission. A zero threshold counts every admitted call.
    #[must_use]
    pub fn with_slow_call_threshold(mut self, threshold: Duration) -> Self {
        self.slow_call_threshold = Some(threshold);
        self
    }

    /// Returns the authenticated caller bound to this session.
    #[must_use]
    pub const fn principal(&self) -> &PluginPrincipal {
        &self.principal
    }

    /// Returns a control handle for invalidating this session from trusted host code.
    #[must_use]
    pub fn revoker(&self) -> ServiceSessionRevoker {
        self.revoker.clone()
    }

    /// Returns control for revoking just the specified transport handle.
    #[must_use]
    pub fn handle_revoker(&self, id: u64) -> Option<ServiceRevoker> {
        self.handles.get(&id).map(|grant| grant.revoker.clone())
    }

    /// Issues a transport handle after host policy approves the service scope.
    pub fn issue<Req, Res>(
        &mut self,
        registry: &ServiceRegistry<Req, Res>,
        scope: ServiceScope,
        expires: Option<Instant>,
    ) -> Result<u64, ServiceSessionError> {
        if self.revoker.is_revoked() {
            return Err(ServiceSessionError::Revoked);
        }
        if self.handles.len() >= self.max_handles {
            return Err(ServiceSessionError::HandleLimit);
        }
        let id = NEXT_HANDLE_ID
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
            .map_err(|_| ServiceSessionError::HandleLimit)?;
        let handle = registry
            .issue(&mut self.issuer, self.principal.clone(), scope, expires)
            .map_err(ServiceSessionError::Dispatch)?;
        self.handles.insert(
            id,
            SessionGrant {
                handle,
                revoker: ServiceRevoker::new(),
            },
        );
        Ok(id)
    }

    /// Releases a transport handle issued in this session.
    pub fn release(&mut self, id: u64) -> bool {
        let Some(grant) = self.handles.remove(&id) else {
            return false;
        };
        grant.revoker.revoke();
        grant.handle.revoke(&mut self.issuer);
        true
    }

    pub(super) async fn dispatch<T>(
        &self,
        id: u64,
        invoke: impl AsyncFnOnce(&CapabilityIssuer, &ServiceHandle) -> Result<T, ServiceWireError>,
    ) -> Result<T, ServiceWireError> {
        let (issuer, grant) = self.resolve(id).ok_or(ServiceWireError::Denied)?;
        let _permit = self.acquire_call().ok_or(ServiceWireError::Denied)?;
        let call = until_cancelled(
            invoke(issuer, &grant.handle),
            grant.revoker.cancelled(),
            || Err(ServiceWireError::Denied),
        );
        until_cancelled(call, self.revoker.cancelled(), || {
            Err(ServiceWireError::Denied)
        })
        .await
    }

    fn acquire_call(&self) -> Option<CallPermit<'_>> {
        CallPermit::acquire(
            &self.observer.0,
            self.max_in_flight,
            self.slow_call_threshold,
        )
    }

    fn resolve(&self, id: u64) -> Option<(&CapabilityIssuer, &SessionGrant)> {
        if self.call_limit.as_ref().is_some_and(|limit| !limit.admit()) {
            increment(&self.observer.0.rate_limit_rejections);
            return None;
        }
        self.handles.get(&id).map(|grant| (&self.issuer, grant))
    }
}

impl Drop for ServiceSession {
    fn drop(&mut self) {
        self.revoker.revoke();
        for grant in self.handles.values() {
            grant.revoker.revoke();
        }
    }
}
