//! Capability limits applied to a data transfer (spec §6.20 limits).

/// The size and count limits a capability places on a transfer (spec §6.20).
///
/// Clipboard and drag-and-drop integrate capability limits: a policy may cap the
/// total byte volume a transfer may carry, the number of items, or both. Each
/// bound is optional — `None` means "no limit on this axis" — so the default is
/// entirely [`TransferLimit::unlimited`]. The predicates report whether a
/// proposed transfer fits, without performing the transfer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TransferLimit {
    max_bytes: Option<u64>,
    max_items: Option<usize>,
}

impl TransferLimit {
    /// Creates a limit that constrains nothing (spec §6.20).
    #[must_use]
    pub const fn unlimited() -> Self {
        Self {
            max_bytes: None,
            max_items: None,
        }
    }

    /// Creates a limit that caps only the total byte volume (spec §6.20).
    #[must_use]
    pub const fn bytes(max_bytes: u64) -> Self {
        Self {
            max_bytes: Some(max_bytes),
            max_items: None,
        }
    }

    /// Creates a limit that caps only the item count (spec §6.20).
    #[must_use]
    pub const fn items(max_items: usize) -> Self {
        Self {
            max_bytes: None,
            max_items: Some(max_items),
        }
    }

    /// Returns this limit with its maximum byte volume set (spec §6.20).
    #[must_use]
    pub const fn with_max_bytes(mut self, max_bytes: u64) -> Self {
        self.max_bytes = Some(max_bytes);
        self
    }

    /// Returns this limit with its maximum item count set (spec §6.20).
    #[must_use]
    pub const fn with_max_items(mut self, max_items: usize) -> Self {
        self.max_items = Some(max_items);
        self
    }

    /// Returns the maximum byte volume, if bounded (spec §6.20).
    #[must_use]
    pub const fn max_bytes(&self) -> Option<u64> {
        self.max_bytes
    }

    /// Returns the maximum item count, if bounded (spec §6.20).
    #[must_use]
    pub const fn max_items(&self) -> Option<usize> {
        self.max_items
    }

    /// Returns whether a transfer of `total_bytes` across `item_count` items is
    /// admitted (spec §6.20).
    ///
    /// A bounded axis rejects any value strictly greater than its cap;
    /// unbounded axes admit everything. Both axes must pass.
    #[must_use]
    pub const fn admits(&self, total_bytes: u64, item_count: usize) -> bool {
        let bytes_ok = match self.max_bytes {
            Some(max) => total_bytes <= max,
            None => true,
        };
        let items_ok = match self.max_items {
            Some(max) => item_count <= max,
            None => true,
        };
        bytes_ok && items_ok
    }

    /// Returns whether `total_bytes` exceeds the byte cap (always `false` when
    /// unbounded) (spec §6.20).
    #[must_use]
    pub const fn exceeds_bytes(&self, total_bytes: u64) -> bool {
        match self.max_bytes {
            Some(max) => total_bytes > max,
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unlimited_admits_everything() {
        let limit = TransferLimit::unlimited();
        assert!(limit.admits(u64::MAX, usize::MAX));
        assert!(!limit.exceeds_bytes(u64::MAX));
        assert_eq!(limit, TransferLimit::default());
    }

    #[test]
    fn byte_limit_admits_at_boundary_rejects_over() {
        let limit = TransferLimit::bytes(10);
        assert!(limit.admits(10, 1000));
        assert!(!limit.admits(11, 0));
        assert!(!limit.exceeds_bytes(10));
        assert!(limit.exceeds_bytes(11));
    }

    #[test]
    fn item_limit_admits_at_boundary_rejects_over() {
        let limit = TransferLimit::items(3);
        assert!(limit.admits(u64::MAX, 3));
        assert!(!limit.admits(0, 4));
    }

    #[test]
    fn both_axes_must_pass() {
        let limit = TransferLimit::unlimited()
            .with_max_bytes(100)
            .with_max_items(2);
        assert!(limit.admits(100, 2));
        assert!(!limit.admits(101, 2));
        assert!(!limit.admits(100, 3));
    }
}
