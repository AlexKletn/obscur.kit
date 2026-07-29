//! How much screen a layer surface reserves for itself (spec §6.5).

/// The region a layer surface asks other surfaces to keep clear of.
///
/// This is what makes a panel *work*: a maximised window stops at the panel's
/// edge instead of underneath it. It is modelled as three cases rather than as a
/// signed number because the protocol's `-1` is not "minus one pixel" — it means
/// something categorically different, and a bare `i32` invites writing `-1` by
/// accident and getting a surface that ignores every other panel on the screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LayerExclusiveZone {
    /// Reserve nothing, and stay out of the way of everything that does.
    ///
    /// The surface is placed inside whatever area other exclusive zones have
    /// left over — so an overlay that appears beside an existing panel does not
    /// sit on top of it.
    #[default]
    None,
    /// Reserve `px` logical pixels along the anchored edge.
    ///
    /// Only meaningful for a surface anchored to an edge; the reserved strip is
    /// measured from that edge inward.
    Reserve(u32),
    /// Reserve nothing **and** ignore every other surface's reservation.
    ///
    /// The surface is placed against the raw output edge, overlapping any panel
    /// that reserved space there. This is what a lock screen or a full-screen
    /// overlay wants, and what almost nothing else does.
    Ignore,
}

impl LayerExclusiveZone {
    /// The protocol's signed encoding: `-1` ignore, `0` none, `n > 0` reserve.
    ///
    /// Saturates at [`i32::MAX`] rather than wrapping, because a reservation
    /// wider than the screen is nonsense either way and a wrapped negative would
    /// silently become [`Ignore`](Self::Ignore) — the one value it must never
    /// turn into by accident.
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

    /// Whether this zone takes space away from other surfaces.
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
