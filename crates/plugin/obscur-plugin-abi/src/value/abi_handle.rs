//! An opaque, host-issued 64-bit identifier.

/// An opaque, host-issued 64-bit identifier.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AbiHandle(u64);

impl AbiHandle {
    /// Wraps an opaque identifier allocated by the host.
    #[must_use]
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    /// Returns the identifier for lookup in the allocating host.
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }
}
