//! Typed identifier: `CorrelationId`.

#[doc = include_str!("../../docs/api/id/correlation_id.doc/CorrelationId.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CorrelationId(u64);

impl CorrelationId {
    /// Wraps a raw handle value into a `CorrelationId`.
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
