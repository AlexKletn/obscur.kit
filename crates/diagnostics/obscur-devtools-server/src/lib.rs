//! DevTools endpoint server: transport, auth, subscriptions, runtime bridge,
//! and the security-enforcing [`endpoint::DevToolsEndpoint`] (spec §7.6/§7.7).
//!
//! This crate turns the pure [`obscur_devtools_protocol`] wire types into a
//! working, in-process endpoint. It is disabled by default, authenticates every
//! session against a local token registry, negotiates the protocol version,
//! separates the four capability scopes, enforces subscription/message limits,
//! audits dangerous operations, and never executes arbitrary code. No real IO
//! is performed: the [`transport::LocalTransport`] models the local socket in
//! process, and a real socket transport would implement the same shape.
#![forbid(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]

pub mod auth;
pub mod bridge;
pub mod endpoint;
pub mod subscription;
pub mod transport;
