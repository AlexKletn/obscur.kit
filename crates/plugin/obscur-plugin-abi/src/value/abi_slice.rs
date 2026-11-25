//! Non-owning storage passed across the native ABI.

/// Non-owning storage passed across the native ABI.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct AbiSlice {
    /// Pointer to the first byte, or null for an empty input.
    pub data: *const u8,
    /// Number of readable bytes.
    pub len: usize,
}

impl AbiSlice {
    /// Borrows a Rust byte slice for the duration of a call.
    #[must_use]
    pub const fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            data: bytes.as_ptr(),
            len: bytes.len(),
        }
    }

    /// Returns the input after checking its shape and the caller's byte limit.
    ///
    /// # Safety
    ///
    /// A nonempty input that passes the checks must point to initialized bytes
    /// in one allocation. The bytes must remain readable and immutable for
    /// the returned lifetime.
    pub unsafe fn as_bytes(&self, max_len: usize) -> Option<&[u8]> {
        if self.len == 0 {
            return Some(&[]);
        }
        if self.len > max_len || self.len > isize::MAX as usize || self.data.is_null() {
            return None;
        }
        // SAFETY: the caller guarantees initialized storage and a valid borrow.
        Some(unsafe { std::slice::from_raw_parts(self.data, self.len) })
    }
}
