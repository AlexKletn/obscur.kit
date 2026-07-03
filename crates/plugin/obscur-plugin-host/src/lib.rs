//! Plugin orchestration: discovery, policy, activation and lifecycle.
#![forbid(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]

pub mod activation;
pub mod capability;
pub mod contribution;
pub mod dependency;
pub mod discovery;
pub mod lifecycle;
pub mod manager;
pub mod performance;
pub mod policy;
pub mod storage;
pub mod update;
