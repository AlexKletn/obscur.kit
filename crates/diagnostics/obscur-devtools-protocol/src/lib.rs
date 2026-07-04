//! Stable `DevTools` protocol: versioning, handshake/auth, capability-scoped
//! commands, snapshots, events, and subscriptions (spec §7.6/§7.7).
//!
//! This is a pure type + small-logic layer: version negotiation, scope checks,
//! and limit constants. It carries no transport, auth store, or runtime access —
//! those live in the server crate that consumes these types. The wire types are
//! modeled as ordinary Rust values; a transport serializes them elsewhere.
#![forbid(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]

pub mod error;
pub mod message;
