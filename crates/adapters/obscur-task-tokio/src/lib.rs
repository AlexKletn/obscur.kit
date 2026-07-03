//! Tokio-based task port adapter (spec §2.7 — "Опциональные реализации task
//! ports. Ни одна не является core dependency.").
//!
//! Tokio-backed implementations of the `obscur-task` ports. Every implementation
//! here is optional: the kit's core never depends on Tokio, and a host that
//! wires this adapter chooses to bring the Tokio runtime into its reach.

#![forbid(unsafe_code)]

pub mod spawner;
pub mod timer;
