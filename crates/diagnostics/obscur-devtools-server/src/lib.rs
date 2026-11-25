#![doc = include_str!("../docs/api/lib.doc/_module.md")]
#![forbid(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]

mod audit_entry;
pub mod auth;
pub mod bridge;
mod command_outcome;
pub mod endpoint;
pub mod subscription;
pub mod transport;
