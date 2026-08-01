#![doc = include_str!("layer_exclusive_zone.doc/_module.md")]

#[doc = include_str!("layer_exclusive_zone.doc/LayerExclusiveZone.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LayerExclusiveZone {
    #[doc = include_str!("layer_exclusive_zone.doc/None.md")]
    #[default]
    None,
    #[doc = include_str!("layer_exclusive_zone.doc/Reserve.md")]
    Reserve(u32),
    #[doc = include_str!("layer_exclusive_zone.doc/Ignore.md")]
    Ignore,
}

impl LayerExclusiveZone {
    #[doc = include_str!("layer_exclusive_zone.doc/to_raw.md")]
    #[must_use]
    pub const fn to_raw(self) -> i32 {
        match self {
            Self::None => 0,
            Self::Ignore => -1,
            // The guard is what keeps a huge reservation from wrapping to `-1`,
            // which is the one value it must never become by accident.
            Self::Reserve(px) => {
                if px > i32::MAX.cast_unsigned() {
                    i32::MAX
                } else {
                    px.cast_signed()
                }
            }
        }
    }

    #[doc = include_str!("layer_exclusive_zone.doc/reserves.md")]
    #[must_use]
    pub const fn reserves(self) -> bool {
        matches!(self, Self::Reserve(px) if px > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_three_cases_encode_to_the_protocols_three_signs() {
        assert_eq!(LayerExclusiveZone::None.to_raw(), 0);
        assert_eq!(LayerExclusiveZone::Ignore.to_raw(), -1);
        assert_eq!(LayerExclusiveZone::Reserve(32).to_raw(), 32);
    }

    /// The reason this is an enum: a reservation must never wrap into `-1`,
    /// which would turn "reserve a lot" into "ignore everyone".
    #[test]
    fn an_absurd_reservation_saturates_rather_than_becoming_ignore() {
        let huge = LayerExclusiveZone::Reserve(u32::MAX);
        assert_eq!(huge.to_raw(), i32::MAX);
        assert_ne!(huge.to_raw(), -1);
    }

    #[test]
    fn only_a_positive_reservation_takes_space() {
        assert!(LayerExclusiveZone::Reserve(1).reserves());
        assert!(!LayerExclusiveZone::Reserve(0).reserves());
        assert!(!LayerExclusiveZone::None.reserves());
        assert!(!LayerExclusiveZone::Ignore.reserves());
    }
}
