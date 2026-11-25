mod pending_delivery;

use pending_delivery::PendingDelivery;

use obscur_plugin_executor_wasm::{error::WasmExecutorError, executor::ManagedWasmHandle};

use super::service_revoker::until_cancelled;
use super::{ServiceFrameCodec, ServicePacketDispatch, ServiceSession, ServiceSessionRevoker};

use super::WasmServiceDriverError;

/// Drives host calls for one managed guest and its authenticated session.
pub struct WasmServiceDriver {
    guest: ManagedWasmHandle,
    session: ServiceSession,
    codec: ServiceFrameCodec,
    pending_reply: Option<Vec<u8>>,
}

impl WasmServiceDriver {
    /// Binds the guest to the session issued by the host for its installation.
    /// The session's plugin ID must match the verified manifest ID.
    pub fn new(
        guest: ManagedWasmHandle,
        session: ServiceSession,
        codec: ServiceFrameCodec,
    ) -> Result<Self, WasmServiceDriverError> {
        if guest.id() != session.principal().plugin() {
            return Err(WasmServiceDriverError::PrincipalMismatch);
        }
        Ok(Self {
            guest,
            session,
            codec,
            pending_reply: None,
        })
    }

    /// Returns the session control for host-side revocation.
    #[must_use]
    pub fn revoker(&self) -> ServiceSessionRevoker {
        self.session.revoker()
    }

    /// Sends an application message to the guest on the manager thread.
    pub fn exchange(&mut self, request: &[u8]) -> Result<Vec<u8>, WasmServiceDriverError> {
        self.ensure_live()?;
        if self.pending_reply.is_some() {
            return Err(WasmServiceDriverError::Guest(
                WasmExecutorError::HostCallPending,
            ));
        }
        self.guest
            .exchange(request)
            .map_err(WasmServiceDriverError::Guest)
    }

    /// Dispatches one pending guest request and returns the guest's reply after
    /// delivery. Returns `None` when idle or suspended. A reply completed during
    /// suspension is delivered on the first pump after resumption.
    ///
    /// A cancelled or failed delivery revokes the session. The driver cannot
    /// know whether the service already performed side effects, so it never
    /// retries the request.
    pub async fn pump_next(
        &mut self,
        dispatcher: &impl ServicePacketDispatch,
    ) -> Result<Option<Vec<u8>>, WasmServiceDriverError> {
        self.ensure_live()?;
        if self.guest.is_suspended() {
            return Ok(None);
        }
        if let Some(reply) = self.pending_reply.take() {
            return self.deliver_reply(&reply).map(Some);
        }
        let Some(packet) = self
            .guest
            .take_host_call()
            .map_err(WasmServiceDriverError::Guest)?
        else {
            return Ok(None);
        };
        let mut delivery = PendingDelivery(Some(self.session.revoker()));
        let dispatch = async {
            dispatcher
                .dispatch_packet(self.codec, &self.session, &packet)
                .await
                .map_err(WasmServiceDriverError::Packet)
        };
        let reply = until_cancelled(dispatch, self.guest.closed(), || {
            Err(WasmServiceDriverError::Guest(WasmExecutorError::Inactive))
        })
        .await?;
        self.ensure_live()?;
        if self.guest.is_suspended() {
            self.pending_reply = Some(reply);
            delivery.0 = None;
            return Ok(None);
        }
        delivery.0 = None;
        self.deliver_reply(&reply).map(Some)
    }

    fn deliver_reply(&self, reply: &[u8]) -> Result<Vec<u8>, WasmServiceDriverError> {
        self.guest.exchange(reply).map_err(|error| {
            self.session.revoker().revoke();
            WasmServiceDriverError::Guest(error)
        })
    }

    fn ensure_live(&self) -> Result<(), WasmServiceDriverError> {
        let revoker = self.session.revoker();
        if revoker.is_revoked() {
            Err(WasmServiceDriverError::SessionRevoked)
        } else if self.guest.is_closed() {
            revoker.revoke();
            Err(WasmServiceDriverError::Guest(WasmExecutorError::Inactive))
        } else {
            Ok(())
        }
    }
}
