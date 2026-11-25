use std::{future::Future, pin::Pin};

use obscur_plugin_model::identity::PluginPrincipal;

use super::{ServiceDispatchError, ServiceRequest, ServiceRevoker};
use crate::capability::ServiceScope;

type Reply<T> = Pin<Box<dyn Future<Output = Result<T, ServiceDispatchError>>>>;
type Handler<Req, Res> = Box<dyn Fn(ServiceRequest<Req>) -> Reply<Res>>;

pub(super) struct ServiceRegistration<Req, Res> {
    pub(super) provider: PluginPrincipal,
    pub(super) lifecycle: ServiceRevoker,
    pub(super) scope: ServiceScope,
    pub(super) handler: Handler<Req, Res>,
}

impl<Req, Res> Drop for ServiceRegistration<Req, Res> {
    fn drop(&mut self) {
        self.lifecycle.revoke();
    }
}
