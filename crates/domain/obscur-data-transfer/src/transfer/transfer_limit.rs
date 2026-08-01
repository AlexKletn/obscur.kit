#![doc = include_str!("transfer_limit.doc/_module.md")]

#[doc = include_str!("transfer_limit.doc/TransferLimit.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TransferLimit {
    max_bytes: Option<u64>,
    max_items: Option<usize>,
}

impl TransferLimit {
    #[doc = include_str!("transfer_limit.doc/unlimited.md")]
    #[must_use]
    pub const fn unlimited() -> Self {
        Self {
            max_bytes: None,
            max_items: None,
        }
    }

    #[doc = include_str!("transfer_limit.doc/bytes.md")]
    #[must_use]
    pub const fn bytes(max_bytes: u64) -> Self {
        Self {
            max_bytes: Some(max_bytes),
            max_items: None,
        }
    }

    #[doc = include_str!("transfer_limit.doc/items.md")]
    #[must_use]
    pub const fn items(max_items: usize) -> Self {
        Self {
            max_bytes: None,
            max_items: Some(max_items),
        }
    }

    #[doc = include_str!("transfer_limit.doc/with_max_bytes.md")]
    #[must_use]
    pub const fn with_max_bytes(mut self, max_bytes: u64) -> Self {
        self.max_bytes = Some(max_bytes);
        self
    }

    #[doc = include_str!("transfer_limit.doc/with_max_items.md")]
    #[must_use]
    pub const fn with_max_items(mut self, max_items: usize) -> Self {
        self.max_items = Some(max_items);
        self
    }

    #[doc = include_str!("transfer_limit.doc/max_bytes.md")]
    #[must_use]
    pub const fn max_bytes(&self) -> Option<u64> {
        self.max_bytes
    }

    #[doc = include_str!("transfer_limit.doc/max_items.md")]
    #[must_use]
    pub const fn max_items(&self) -> Option<usize> {
        self.max_items
    }

    #[doc = include_str!("transfer_limit.doc/admits.md")]
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

    #[doc = include_str!("transfer_limit.doc/exceeds_bytes.md")]
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
