#![doc = include_str!("layer_id.doc/_module.md")]

#[doc = include_str!("layer_id.doc/LayerId.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LayerId(u32);

impl LayerId {
    #[doc = include_str!("layer_id.doc/from_raw.md")]
    #[must_use]
    pub const fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    #[doc = include_str!("layer_id.doc/raw.md")]
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
