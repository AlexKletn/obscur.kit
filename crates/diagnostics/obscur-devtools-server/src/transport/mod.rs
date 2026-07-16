//! DevTools transports: the in-process loopback and a real local-socket
//! transport, plus their shared wire codec (spec §7.6/§7.7).
//!
//! Two queues carry what does not fit request/reply, one per direction, and they
//! are deliberately the same shape: [`EventQueue`] is the host stating what
//! happened for the serving loop to fan out to subscribers, and [`InjectionQueue`]
//! is the serving loop delivering what an authorized client injected for the host
//! to apply to its runtime. Neither lets one side reach into the other's thread —
//! the endpoint keeps every §7.7 decision, the host keeps the runtime.

mod event_queue;
pub use event_queue::EventQueue;
mod injection_queue;
pub use injection_queue::{InjectionQueue, InjectionSink};
mod local_transport;
pub use local_transport::{ClientBound, LocalTransport, ServerBound};
mod wire_codec;
pub use wire_codec::{WireCodec, WireError};
mod socket_stream;
pub use socket_stream::{MAX_FRAME_LEN, SocketStream};
mod socket_listener;
pub use socket_listener::SocketListener;
mod socket_transport;
pub use socket_transport::SocketTransport;
mod socket_client;
pub use socket_client::SocketClient;
