#![doc = include_str!("mod.doc/_module.md")]
mod capability_changed;
pub use capability_changed::CapabilityChanged;
mod capability_revoked;
pub use capability_revoked::CapabilityRevoked;
