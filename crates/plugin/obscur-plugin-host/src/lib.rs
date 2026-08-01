#![doc = include_str!("lib.doc/_module.md")]
#![forbid(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]

#[doc = include_str!("lib.doc/activation.md")]
pub mod activation;
#[doc = include_str!("lib.doc/capability.md")]
pub mod capability;
#[doc = include_str!("lib.doc/contribution.md")]
pub mod contribution;
#[doc = include_str!("lib.doc/dependency.md")]
pub mod dependency;
#[doc = include_str!("lib.doc/discovery.md")]
pub mod discovery;
#[doc = include_str!("lib.doc/lifecycle.md")]
pub mod lifecycle;
#[doc = include_str!("lib.doc/manager.md")]
pub mod manager;
#[doc = include_str!("lib.doc/performance.md")]
pub mod performance;
#[doc = include_str!("lib.doc/policy.md")]
pub mod policy;
#[doc = include_str!("lib.doc/storage.md")]
pub mod storage;
#[doc = include_str!("lib.doc/update.md")]
pub mod update;
