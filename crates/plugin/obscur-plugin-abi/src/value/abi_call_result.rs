//! Result of a native host call using a borrowed reply buffer.

use super::AbiStatus;

/// Carries an ABI status and the number of initialized reply bytes.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct AbiCallResult {
    /// Status of the host callback.
    pub status: AbiStatus,
    /// Bytes written on success. On failure, this is zero.
    pub written: usize,
}
