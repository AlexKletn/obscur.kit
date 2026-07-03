//! An opaque identifier for a compositing layer.

/// An opaque identifier for a compositing layer (spec §5.1).
///
/// A newtype over a `u32`. Its integer value is an identity token only: callers
/// mint and compare `LayerId`s but must not attribute meaning to the underlying
/// number beyond equality and a stable, deterministic ordering. It is `Ord` and
/// `Hash` so layers can be sorted and keyed in maps.
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
