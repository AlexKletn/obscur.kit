#![doc = include_str!("lib.doc/_module.md")]
#![forbid(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]

#[doc = include_str!("lib.doc/executor.md")]
pub mod executor;
#[doc = include_str!("lib.doc/handle.md")]
pub mod handle;
#[doc = include_str!("lib.doc/host.md")]
pub mod host;
#[doc = include_str!("lib.doc/plugin.md")]
pub mod plugin;
#[doc = include_str!("lib.doc/service.md")]
pub mod service;
#[doc = include_str!("lib.doc/status.md")]
pub mod status;
