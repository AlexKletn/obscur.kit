//! Typed identifier: `NamespaceId`.

/// Stable, opaque identifier for a identifier namespace.
///
/// Backed by an integer handle; the numeric value carries no meaning outside
/// the subsystem that mints it and must never be treated as a public schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NamespaceId(u64);

impl NamespaceId {
    /// Wraps a raw handle value into a `NamespaceId`.
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
