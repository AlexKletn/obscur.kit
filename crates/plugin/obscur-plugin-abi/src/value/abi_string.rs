//! Non-owning storage passed across the native ABI.

use super::AbiSlice;

/// Non-owning storage passed across the native ABI.
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct AbiString(pub AbiSlice);

impl AbiString {
    /// Borrows a Rust string for the duration of a call.
    #[must_use]
    pub const fn from_text(text: &str) -> Self {
        Self(AbiSlice::from_bytes(text.as_bytes()))
    }

    /// Checks the byte limit and UTF-8 encoding before returning text.
    ///
    /// # Safety
    ///
    /// The input must satisfy the requirements of [AbiSlice::as_bytes].
    pub unsafe fn as_str(&self, max_len: usize) -> Option<&str> {
        // SAFETY: the caller upholds the validity and lifetime of the backing slice.
        std::str::from_utf8(unsafe { self.0.as_bytes(max_len)? }).ok()
    }
}
