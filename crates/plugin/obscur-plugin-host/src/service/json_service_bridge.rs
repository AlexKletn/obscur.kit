mod bounded_response;

use bounded_response::BoundedResponse;

use std::{future::Future, io};

use obscur_plugin_model::identity::PluginPrincipal;
use serde::{Serialize, de::DeserializeOwned};

use super::service_revoker::until_cancelled;
use super::{
    ServiceCallFrame, ServiceFrameCodec, ServiceHandle, ServicePacketDispatch, ServiceRegistry,
    ServiceSession,
};
use crate::capability::CapabilityIssuer;

use super::ServiceWireError;

/// Bridges JSON payloads from a transport adapter to a typed service registry.
pub struct JsonServiceBridge<'a, Req, Res> {
    registry: &'a ServiceRegistry<Req, Res>,
    max_request_bytes: usize,
    max_response_bytes: usize,
}

impl<'a, Req: DeserializeOwned, Res: Serialize> JsonServiceBridge<'a, Req, Res> {
    /// Binds the bridge to the registry and host-defined wire byte budgets.
    #[must_use]
    pub const fn new(
        registry: &'a ServiceRegistry<Req, Res>,
        max_request_bytes: usize,
        max_response_bytes: usize,
    ) -> Self {
        Self {
            registry,
            max_request_bytes,
            max_response_bytes,
        }
    }

    /// Dispatches one packet from a message transport and encodes its reply.
    pub async fn dispatch_packet(
        &self,
        codec: ServiceFrameCodec,
        session: &ServiceSession,
        packet: &[u8],
    ) -> io::Result<Vec<u8>> {
        ServicePacketDispatch::dispatch_packet(self, codec, session, packet).await
    }

    /// Dispatches a decoded stream frame in its authenticated session.
    pub async fn dispatch_frame(
        &self,
        session: &ServiceSession,
        call: ServiceCallFrame<'_>,
    ) -> Result<Vec<u8>, ServiceWireError> {
        self.dispatch_session(session, call.handle, call.operation, call.payload)
            .await
    }

    /// Dispatches until the host-provided deadline future completes.
    ///
    /// Expiration drops a pending handler and returns the redacted `Failed`
    /// status. This cannot interrupt synchronous work inside a handler poll
    /// or undo side effects the handler already performed.
    pub async fn dispatch_frame_until(
        &self,
        session: &ServiceSession,
        call: ServiceCallFrame<'_>,
        deadline: impl Future<Output = ()>,
    ) -> Result<Vec<u8>, ServiceWireError> {
        until_cancelled(self.dispatch_frame(session, call), deadline, || {
            Err(ServiceWireError::Failed)
        })
        .await
    }

    /// Dispatches a transport call using only the session's identity and grants.
    pub async fn dispatch_session(
        &self,
        session: &ServiceSession,
        id: u64,
        operation: &str,
        payload: &[u8],
    ) -> Result<Vec<u8>, ServiceWireError> {
        session
            .dispatch(id, async |issuer, handle| {
                self.dispatch(issuer, handle, session.principal(), operation, payload)
                    .await
            })
            .await
    }

    /// Authorizes the call, decodes its request, and encodes a bounded JSON reply.
    ///
    /// The adapter resolves the handle and caller from its authenticated host
    /// context and enforces a frame size limit before buffering transport data.
    pub async fn dispatch(
        &self,
        issuer: &CapabilityIssuer,
        handle: &ServiceHandle,
        caller: &PluginPrincipal,
        operation: &str,
        payload: &[u8],
    ) -> Result<Vec<u8>, ServiceWireError> {
        let response = self
            .registry
            .dispatch_with(issuer, handle, caller, operation, || {
                if payload.len().saturating_add(operation.len()) > self.max_request_bytes {
                    return Err(ServiceWireError::RequestTooLarge);
                }
                serde_json::from_slice(payload).map_err(|_| ServiceWireError::InvalidRequest)
            })
            .await?;

        let mut output = BoundedResponse {
            bytes: Vec::new(),
            limit: self.max_response_bytes,
            exceeded: false,
        };
        serde_json::to_writer(&mut output, &response).map_err(|_| {
            if output.exceeded {
                ServiceWireError::ResponseTooLarge
            } else {
                ServiceWireError::Failed
            }
        })?;
        Ok(output.bytes)
    }
}

impl<Req: DeserializeOwned, Res: Serialize> ServicePacketDispatch
    for JsonServiceBridge<'_, Req, Res>
{
    async fn dispatch_frame(
        &self,
        session: &ServiceSession,
        call: ServiceCallFrame<'_>,
    ) -> Result<Vec<u8>, ServiceWireError> {
        Self::dispatch_frame(self, session, call).await
    }
}
