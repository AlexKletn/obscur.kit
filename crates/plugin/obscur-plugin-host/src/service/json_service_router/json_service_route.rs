use std::{future::Future, pin::Pin};

use obscur_plugin_model::identity::PluginPrincipal;
use serde::{Serialize, de::DeserializeOwned};

use super::{
    CapabilityIssuer, JsonServiceBridge, ServiceCallFrame, ServiceHandle, ServiceWireError,
};

type JsonReply<'a> = Pin<Box<dyn Future<Output = Result<Vec<u8>, ServiceWireError>> + 'a>>;

pub(super) trait JsonServiceRoute {
    fn dispatch<'a>(
        &'a self,
        issuer: &'a CapabilityIssuer,
        handle: &'a ServiceHandle,
        caller: &'a PluginPrincipal,
        call: ServiceCallFrame<'a>,
    ) -> JsonReply<'a>;
}

impl<Req: DeserializeOwned, Res: Serialize> JsonServiceRoute for JsonServiceBridge<'_, Req, Res> {
    fn dispatch<'a>(
        &'a self,
        issuer: &'a CapabilityIssuer,
        handle: &'a ServiceHandle,
        caller: &'a PluginPrincipal,
        call: ServiceCallFrame<'a>,
    ) -> JsonReply<'a> {
        Box::pin(self.dispatch(issuer, handle, caller, call.operation, call.payload))
    }
}
