//! Typed identifier: `ResourceId`.

#[doc = include_str!("../../docs/api/id/resource_id.doc/ResourceId.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ResourceId(u64);

impl ResourceId {
    /// Wraps a raw handle value into a `ResourceId`.
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
