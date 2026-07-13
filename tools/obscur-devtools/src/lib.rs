//! The external DevTools CLI (spec §7.6): a client that connects to a running
//! runtime's diagnostics endpoint over the local socket transport and inspects
//! live state, plus a self-contained demo host to talk to.
//!
//! The binary in [`main`](../main.rs) is a thin argument parser over this
//! library; the client-driving and formatting logic lives here so it can be
//! exercised end-to-end over a real loopback socket in tests.
#![forbid(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]

pub mod cli;
pub mod host;
