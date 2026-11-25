//! Capability and permission model used by platform and plugin layers.
#![forbid(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]

mod capability;
mod permission;
mod support;

pub use capability::{Capability, CapabilityId, CapabilityScope, CapabilitySnapshot};
pub use permission::{PermissionDuration, PermissionState};
pub use support::SupportLevel;

pub mod event;
