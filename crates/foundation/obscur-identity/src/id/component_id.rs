//! Typed identifier: `ComponentId`.

#[doc = include_str!("../../docs/api/id/component_id.doc/ComponentId.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ComponentId(u64);

impl ComponentId {
    /// Wraps a raw handle value into a `ComponentId`.
    #[must_use]
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    /// Returns the raw handle value backing this identifier.
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }
}
