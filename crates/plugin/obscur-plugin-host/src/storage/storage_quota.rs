//! Plugin state storage quotas.

/// Per-snapshot payload budget for plugin state storage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StorageQuota {
    max_bytes: usize,
}

impl StorageQuota {
    /// Bounds the payload size of each stored snapshot.
    #[must_use]
    pub const fn new(max_bytes: usize) -> Self {
        Self { max_bytes }
    }

    /// Returns the maximum payload size in bytes.
    #[must_use]
    pub const fn max_bytes(self) -> usize {
        self.max_bytes
    }
}
