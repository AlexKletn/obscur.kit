//! Immutable snapshot of available capabilities.

use super::Capability;

/// An immutable snapshot of the capabilities available to a principal at
/// activation time (spec §8.58).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CapabilitySnapshot {
    entries: Vec<Capability>,
}

impl CapabilitySnapshot {
    /// Builds a snapshot from a set of capability entries.
    #[must_use]
    pub fn new(entries: Vec<Capability>) -> Self {
        Self { entries }
    }

    /// Returns the capabilities in this snapshot.
    #[must_use]
    pub fn entries(&self) -> &[Capability] {
        &self.entries
    }
}
