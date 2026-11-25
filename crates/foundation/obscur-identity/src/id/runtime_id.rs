//! Typed identifier: `RuntimeId`.

#[doc = include_str!("../../docs/api/id/runtime_id.doc/RuntimeId.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RuntimeId(u64);

impl RuntimeId {
    /// Wraps a raw handle value into a `RuntimeId`.
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
