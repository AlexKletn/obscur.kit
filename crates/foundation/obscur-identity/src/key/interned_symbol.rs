//! Interned symbol handle.

/// An opaque handle into a symbol interner. The backing table lives in the
/// subsystem that owns interning; this type is only the stable handle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct InternedSymbol(u32);

impl InternedSymbol {
    /// Wraps a raw interner index.
    #[must_use]
    pub const fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    /// Returns the raw interner index.
    #[must_use]
    pub const fn raw(self) -> u32 {
        self.0
    }
}
