//! A capability paired with its current support level.

use super::{CapabilityId, CapabilityScope};
use crate::support::SupportLevel;

/// A capability entry: its identity, the scope it applies to and how well it is
/// currently supported.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Capability {
    /// Stable capability identity.
    pub id: CapabilityId,
    /// Scope the capability is issued at.
    pub scope: CapabilityScope,
    /// Current support level.
    pub support: SupportLevel,
}
