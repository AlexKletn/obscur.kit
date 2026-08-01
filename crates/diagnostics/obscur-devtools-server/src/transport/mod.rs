#![doc = include_str!("mod.doc/_module.md")]

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
