//! Pure versioned plugin DTO and contracts.
#![forbid(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]

/// What a plugin may ask the host for, as data.
pub mod capability;
/// What a plugin adds to the host: the declared extension points.
pub mod contribution;
/// A plugin's stable name and version.
pub mod identity;
/// The states a plugin moves through, from discovered to unloaded.
pub mod lifecycle;
/// The manifest a plugin ships: identity, capabilities, contributions.
pub mod manifest;
/// The typed messages that cross the host/plugin boundary.
pub mod message;
/// Services a host exposes to plugins, described as data.
pub mod service;
/// A plugin's declared settings and their values.
pub mod settings;
/// A plugin's persisted state.
pub mod state;
