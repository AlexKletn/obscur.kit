//! An opaque identifier for a compositing layer.

#[doc = include_str!("../../docs/api/layer/layer_id.doc/LayerId.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LayerId(u32);

impl LayerId {
    /// Creates a layer id from a raw integer value.
    #[must_use]
    pub const fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    /// Returns the raw integer value backing this id.
    #[must_use]
    pub const fn raw(self) -> u32 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_roundtrips() {
        assert_eq!(LayerId::from_raw(9).raw(), 9);
    }

    #[test]
    fn equality_and_ordering() {
        assert_eq!(LayerId::from_raw(1), LayerId::from_raw(1));
        assert_ne!(LayerId::from_raw(1), LayerId::from_raw(2));
        assert!(LayerId::from_raw(1) < LayerId::from_raw(2));
    }
}
