//! In-process loopback transport and its wire envelopes (spec §7.6/§7.7).

mod local_transport;
pub use local_transport::{ClientBound, LocalTransport, ServerBound};
