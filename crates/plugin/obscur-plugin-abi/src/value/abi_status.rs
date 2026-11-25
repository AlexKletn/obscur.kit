//! A fixed-width status returned across the native ABI.

/// A fixed-width status returned across the native ABI.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AbiStatus(i32);

impl AbiStatus {
    /// The callback completed successfully.
    pub const OK: Self = Self(0);
    /// The callback rejected the request.
    pub const REJECTED: Self = Self(1);
    /// A panic was caught on the plugin side of the boundary.
    pub const PANICKED: Self = Self(2);

    /// Returns the raw C ABI status code.
    #[must_use]
    pub const fn raw(self) -> i32 {
        self.0
    }
}
