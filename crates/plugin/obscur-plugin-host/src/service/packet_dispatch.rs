use std::{future::Future, io};

use super::{ServiceCallFrame, ServiceFrameCodec, ServiceSession, ServiceWireError};

/// Host dispatch contract for complete version-1 service packets.
pub trait ServicePacketDispatch {
    /// Dispatches a decoded call in its authenticated session.
    fn dispatch_frame(
        &self,
        session: &ServiceSession,
        call: ServiceCallFrame<'_>,
    ) -> impl Future<Output = Result<Vec<u8>, ServiceWireError>>;

    /// Decodes one complete packet and encodes its reply.
    fn dispatch_packet(
        &self,
        codec: ServiceFrameCodec,
        session: &ServiceSession,
        packet: &[u8],
    ) -> impl Future<Output = io::Result<Vec<u8>>> {
        async move {
            let call = ServiceFrameCodec::decode_call(codec.decode_packet(packet)?)?;
            let reply = self.dispatch_frame(session, call).await;
            let mut output = Vec::new();
            codec.write_reply(&mut output, reply.as_deref().map_err(|error| *error))?;
            Ok(output)
        }
    }
}
