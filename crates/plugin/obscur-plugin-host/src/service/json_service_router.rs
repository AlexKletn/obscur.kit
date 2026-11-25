mod json_service_route;

use json_service_route::JsonServiceRoute;

use std::collections::{BTreeMap, btree_map::Entry};

use obscur_plugin_model::manifest::Version;
use serde::{Serialize, de::DeserializeOwned};

use super::{
    JsonServiceBridge, ServiceCallFrame, ServiceDispatchError, ServiceHandle,
    ServicePacketDispatch, ServiceSession, ServiceWireError,
};
use crate::capability::CapabilityIssuer;

/// Host router for independently typed JSON services in the same session.
#[derive(Default)]
pub struct JsonServiceRouter<'a> {
    routes: BTreeMap<Box<str>, BTreeMap<Version, Box<dyn JsonServiceRoute + 'a>>>,
}

impl<'a> JsonServiceRouter<'a> {
    /// Mounts one service version with its own typed registry and byte budgets.
    pub fn mount<Req: DeserializeOwned + 'a, Res: Serialize + 'a>(
        &mut self,
        service: impl Into<Box<str>>,
        version: Version,
        bridge: JsonServiceBridge<'a, Req, Res>,
    ) -> Result<(), ServiceDispatchError> {
        let versions = self.routes.entry(service.into()).or_default();
        let Entry::Vacant(route) = versions.entry(version) else {
            return Err(ServiceDispatchError::AlreadyRegistered);
        };
        route.insert(Box::new(bridge));
        Ok(())
    }
}

impl ServicePacketDispatch for JsonServiceRouter<'_> {
    async fn dispatch_frame(
        &self,
        session: &ServiceSession,
        call: ServiceCallFrame<'_>,
    ) -> Result<Vec<u8>, ServiceWireError> {
        session
            .dispatch(call.handle, async |issuer, handle| {
                let (service, version) = handle.service_key();
                let route = self
                    .routes
                    .get(service)
                    .and_then(|versions| versions.get(version))
                    .ok_or(ServiceWireError::Unsupported)?;
                route
                    .dispatch(issuer, handle, session.principal(), call)
                    .await
            })
            .await
    }
}
