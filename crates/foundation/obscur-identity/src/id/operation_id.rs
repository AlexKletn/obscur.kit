//! Typed identifier: `OperationId`.

/// Stable, opaque identifier for a normalized operation.
///
/// Backed by an integer handle; the numeric value carries no meaning outside
/// the subsystem that mints it and must never be treated as a public schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct OperationId(u64);

impl OperationId {
    /// Wraps a raw handle value into a `OperationId`.
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
