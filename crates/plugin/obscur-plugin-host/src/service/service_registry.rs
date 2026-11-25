mod service_registration;

use service_registration::ServiceRegistration;

use std::{
    collections::BTreeMap,
    future::{Future, poll_fn},
    panic::{AssertUnwindSafe, catch_unwind},
    task::Poll,
    time::Instant,
};

use obscur_plugin_model::{identity::PluginPrincipal, manifest::Version};

use super::{
    ServiceDispatchError, ServiceHandle, ServiceRequest, ServiceRevoker,
    service_revoker::until_cancelled,
};
use crate::capability::{CapabilityAccessError, CapabilityIssuer, ServiceScope};

pub(super) type ServiceKey = (Box<str>, Version);
/// In-process registry for one typed request and response contract.
pub struct ServiceRegistry<Req, Res> {
    services: BTreeMap<ServiceKey, ServiceRegistration<Req, Res>>,
}

impl<Req, Res> Default for ServiceRegistry<Req, Res> {
    fn default() -> Self {
        Self {
            services: BTreeMap::new(),
        }
    }
}

impl<Req, Res> ServiceRegistry<Req, Res> {
    /// Lists live service registrations and their providers for host inspection. Revoked registrations disappear immediately.
    pub fn active_services(&self) -> impl Iterator<Item = (&PluginPrincipal, &ServiceScope)> {
        self.services
            .values()
            .filter(|registration| !registration.lifecycle.is_revoked())
            .map(|registration| (&registration.provider, &registration.scope))
    }

    /// Registers a provider and the operations available on one service version.
    pub fn register<Fut>(
        &mut self,
        provider: PluginPrincipal,
        scope: ServiceScope,
        handler: impl Fn(ServiceRequest<Req>) -> Fut + 'static,
    ) -> Result<(), ServiceDispatchError>
    where
        Fut: Future<Output = Result<Res, ServiceDispatchError>> + 'static,
    {
        let (id, version) = scope.key();
        let std::collections::btree_map::Entry::Vacant(slot) =
            self.services.entry((id.into(), version.clone()))
        else {
            return Err(ServiceDispatchError::AlreadyRegistered);
        };
        slot.insert(ServiceRegistration {
            provider,
            scope,
            lifecycle: ServiceRevoker::new(),
            handler: Box::new(move |request| Box::pin(handler(request))),
        });
        Ok(())
    }

    /// Issues a service handle after the host has approved the consumer's request.
    pub fn issue(
        &self,
        issuer: &mut CapabilityIssuer,
        caller: PluginPrincipal,
        scope: ServiceScope,
        expires: Option<Instant>,
    ) -> Result<ServiceHandle, ServiceDispatchError> {
        let (id, version) = scope.key();
        let key = (id.into(), version.clone());
        let service = self
            .services
            .get(&key)
            .filter(|service| !service.lifecycle.is_revoked())
            .ok_or(ServiceDispatchError::Unsupported)?;
        if !service.scope.includes(&scope) {
            return Err(CapabilityAccessError::Denied.into());
        }
        let capability = issuer.issue(caller, scope, expires)?;
        Ok(ServiceHandle {
            key,
            lifecycle: service.lifecycle.clone(),
            capability,
        })
    }

    /// Issues a narrower handle for a host-approved delegate.
    pub fn delegate(
        &self,
        issuer: &mut CapabilityIssuer,
        source: &ServiceHandle,
        caller: PluginPrincipal,
        scope: ServiceScope,
        expires: Option<Instant>,
    ) -> Result<ServiceHandle, ServiceDispatchError> {
        self.registration(source)?;
        let capability = issuer.delegate(&source.capability, caller, scope, expires)?;
        Ok(ServiceHandle {
            key: source.key.clone(),
            lifecycle: source.lifecycle.clone(),
            capability,
        })
    }

    /// Checks the grant before calling the registered provider.
    pub async fn dispatch(
        &self,
        issuer: &CapabilityIssuer,
        handle: &ServiceHandle,
        caller: &PluginPrincipal,
        operation: &str,
        payload: Req,
    ) -> Result<Res, ServiceDispatchError> {
        self.dispatch_with(issuer, handle, caller, operation, || Ok(payload))
            .await
    }

    pub(super) async fn dispatch_with<E>(
        &self,
        issuer: &CapabilityIssuer,
        handle: &ServiceHandle,
        caller: &PluginPrincipal,
        operation: &str,
        decode: impl FnOnce() -> Result<Req, E>,
    ) -> Result<Res, E>
    where
        E: From<ServiceDispatchError>,
    {
        let service = self.registration(handle)?;
        issuer
            .authorize(
                &handle.capability,
                caller,
                &handle.key.0,
                &handle.key.1,
                operation,
            )
            .map_err(ServiceDispatchError::from)?;
        let request = ServiceRequest {
            caller: caller.clone(),
            operation: operation.into(),
            payload: decode()?,
        };
        let call = async {
            let mut response = catch_unwind(AssertUnwindSafe(|| (service.handler)(request)))
                .map_err(|_| ServiceDispatchError::Panicked)?;
            poll_fn(|context| {
                catch_unwind(AssertUnwindSafe(|| response.as_mut().poll(context)))
                    .unwrap_or(Poll::Ready(Err(ServiceDispatchError::Panicked)))
            })
            .await
        };
        until_cancelled(call, service.lifecycle.cancelled(), || {
            Err(ServiceDispatchError::Unsupported)
        })
        .await
        .map_err(E::from)
    }

    /// Returns host control to withdraw one service version while calls are pending.
    /// Revocation denies future calls and wakes the pending handlers.
    #[must_use]
    pub fn registration_revoker(&self, service: &str, version: &Version) -> Option<ServiceRevoker> {
        self.services
            .get(&(service.into(), version.clone()))
            .map(|registration| registration.lifecycle.clone())
    }

    /// Withdraws the provider's registrations and invalidates their handles.
    pub fn unregister_provider(&mut self, provider: &PluginPrincipal) -> usize {
        let previous = self.services.len();
        self.services
            .retain(|_, registration| &registration.provider != provider);
        previous - self.services.len()
    }

    fn registration(
        &self,
        handle: &ServiceHandle,
    ) -> Result<&ServiceRegistration<Req, Res>, ServiceDispatchError> {
        self.services
            .get(&handle.key)
            .filter(|service| {
                !service.lifecycle.is_revoked() && service.lifecycle.same_target(&handle.lifecycle)
            })
            .ok_or(ServiceDispatchError::Unsupported)
    }
}
