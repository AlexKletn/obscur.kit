//! Authenticated request and response transport over separate byte streams.

mod framed_connection;

use framed_connection::FramedConnection;

use std::{
    future::{Future, pending},
    io::{self, Read, Write},
};

use obscur_plugin_host::service::{JsonServiceBridge, ServiceFrameCodec, ServiceSession};
use serde::{Serialize, de::DeserializeOwned};

/// Owns the input, output and session for one sequential plugin connection.
pub struct FramedTransport<R, W> {
    codec: ServiceFrameCodec,
    connection: Option<FramedConnection<R, W>>,
}

impl<R: Read, W: Write> FramedTransport<R, W> {
    /// Attaches the already authenticated session to its paired streams.
    #[must_use]
    pub fn new(codec: ServiceFrameCodec, input: R, output: W, session: ServiceSession) -> Self {
        Self {
            codec,
            connection: Some(FramedConnection {
                input,
                output,
                session,
            }),
        }
    }

    /// Reports whether the transport still owns its streams and grants.
    #[must_use]
    pub const fn is_open(&self) -> bool {
        self.connection.is_some()
    }

    /// Drops both streams and the session's grants.
    pub fn close(&mut self) {
        self.connection = None;
    }

    /// Serves and flushes one reply. Returns false on EOF or a closed transport.
    ///
    /// Stream reads and writes are blocking. Run this on a transport worker.
    /// A framing or stream error, or cancellation of an in-flight call, closes
    /// the connection so a later request cannot consume the wrong reply.
    ///
    /// # Errors
    ///
    /// Returns the framing or stream error if reading, decoding, writing, or
    /// flushing fails. The transport is closed in that case.
    pub async fn serve_next<Req: DeserializeOwned, Res: Serialize>(
        &mut self,
        bridge: &JsonServiceBridge<'_, Req, Res>,
    ) -> io::Result<bool> {
        self.serve_next_until(bridge, pending).await
    }

    /// Creates a host deadline after reading a frame and uses it for dispatch.
    /// A timed-out handler produces an error reply without closing the stream.
    ///
    /// # Errors
    ///
    /// Returns the same framing and stream errors as [`Self::serve_next`].
    pub async fn serve_next_until<
        Req: DeserializeOwned,
        Res: Serialize,
        Timer: Future<Output = ()>,
    >(
        &mut self,
        bridge: &JsonServiceBridge<'_, Req, Res>,
        deadline: impl FnOnce() -> Timer,
    ) -> io::Result<bool> {
        let Some(mut connection) = self.connection.take() else {
            return Ok(false);
        };
        let Some(frame) = self.codec.read_frame(&mut connection.input)? else {
            return Ok(false);
        };
        let call = ServiceFrameCodec::decode_call(&frame)?;
        let reply = bridge
            .dispatch_frame_until(&connection.session, call, deadline())
            .await;
        self.codec.write_reply(
            &mut connection.output,
            reply.as_deref().map_err(|error| *error),
        )?;
        connection.output.flush()?;
        self.connection = Some(connection);
        Ok(true)
    }
}
