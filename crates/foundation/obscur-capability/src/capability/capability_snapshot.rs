#![doc = include_str!("capability_snapshot.doc/_module.md")]

use super::Capability;

#[doc = include_str!("capability_snapshot.doc/CapabilitySnapshot.md")]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CapabilitySnapshot {
    entries: Vec<Capability>,
}

impl CapabilitySnapshot {
    #[doc = include_str!("capability_snapshot.doc/new.md")]
    #[must_use]
    pub fn new(entries: Vec<Capability>) -> Self {
        Self { entries }
    }

    #[doc = include_str!("capability_snapshot.doc/entries.md")]
    #[must_use]
    pub fn entries(&self) -> &[Capability] {
        &self.entries
    }
}
