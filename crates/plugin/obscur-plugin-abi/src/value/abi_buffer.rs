//! Non-owning storage passed across the native ABI.

/// Non-owning storage passed across the native ABI.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct AbiBuffer {
    /// Pointer to caller-provided writable storage.
    pub data: *mut u8,
    /// Number of writable bytes.
    pub capacity: usize,
}

impl AbiBuffer {
    /// Borrows writable storage for the duration of an ABI call.
    #[must_use]
    pub fn from_bytes(bytes: &mut [u8]) -> Self {
        Self {
            data: bytes.as_mut_ptr(),
            capacity: bytes.len(),
        }
    }

    /// Copies a complete reply into caller-owned storage.
    ///
    /// Returns false if the buffer is too small or has an invalid shape.
    ///
    /// # Safety
    ///
    /// When the checks pass for nonempty input, the destination must be
    /// exclusively writable for at least `bytes.len()` bytes in one
    /// allocation and must not overlap the source.
    pub unsafe fn write(&self, bytes: &[u8]) -> bool {
        if bytes.is_empty() {
            return true;
        }
        if bytes.len() > self.capacity || self.capacity > isize::MAX as usize || self.data.is_null()
        {
            return false;
        }
        // SAFETY: capacity was checked above and the caller guarantees writable, disjoint storage.
        unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), self.data, bytes.len()) };
        true
    }
}
