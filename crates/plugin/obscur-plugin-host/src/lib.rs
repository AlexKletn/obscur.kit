//! Plugin orchestration: discovery, policy, activation and lifecycle.
#![forbid(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]

/// When and why a plugin is activated.
pub mod activation;
/// What a plugin may ask the host for.
pub mod capability;
/// What a plugin adds to the host.
pub mod contribution;
/// Dependencies between plugins.
pub mod dependency;
/// Finding plugins on disk.
pub mod discovery;
/// The states a plugin moves through.
pub mod lifecycle;
/// The host object that owns loaded plugins.
pub mod manager;
/// What the host measures about plugin cost.
pub mod performance;
/// The rules the host applies to plugins.
pub mod policy;
/// Where plugin state is kept.
pub mod storage;
/// Replacing a loaded plugin with a newer one.
pub mod update;
