#![doc = include_str!("lib.doc/_module.md")]
#![forbid(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]

pub mod auth;
pub mod bridge;
pub mod endpoint;
pub mod subscription;
pub mod transport;
