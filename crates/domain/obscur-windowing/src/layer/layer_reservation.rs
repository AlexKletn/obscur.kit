//! The screen space a layer surface asks other windows to keep clear of.

#[doc = include_str!("../../docs/api/layer/layer_reservation.doc/LayerReservation.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LayerReservation {
    /// Reserve nothing: other windows may sit underneath this surface.
    #[default]
    None,
    #[doc = include_str!("../../docs/api/layer/layer_reservation.doc/Reserve.md")]
    Reserve(u32),
    #[doc = include_str!("../../docs/api/layer/layer_reservation.doc/Ignore.md")]
    Ignore,
}

impl LayerReservation {
    #[doc = include_str!("../../docs/api/layer/layer_reservation.doc/reserves.md")]
    #[must_use]
    pub const fn reserves(self) -> bool {
        matches!(self, Self::Reserve(px) if px > 0)
    }

    #[doc = include_str!("../../docs/api/layer/layer_reservation.doc/reserved_pixels.md")]
    #[must_use]
    pub const fn reserved_pixels(self) -> Option<u32> {
        match self {
            Self::Reserve(px) => Some(px),
            Self::None | Self::Ignore => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_a_positive_reservation_takes_space() {
        assert!(LayerReservation::Reserve(1).reserves());
        assert!(!LayerReservation::Reserve(0).reserves());
        assert!(!LayerReservation::None.reserves());
        assert!(!LayerReservation::Ignore.reserves());
    }

    #[test]
    fn the_default_reserves_nothing_but_respects_everyone_else() {
        assert_eq!(LayerReservation::default(), LayerReservation::None);
        assert!(!LayerReservation::default().reserves());
    }

    #[test]
    fn ignoring_is_not_a_quantity() {
        // The distinction the three cases exist to preserve: `Ignore` carries
        // no pixel count, so a backend cannot mistake it for a measurement.
        assert_eq!(LayerReservation::Ignore.reserved_pixels(), None);
        assert_eq!(LayerReservation::None.reserved_pixels(), None);
        assert_eq!(LayerReservation::Reserve(32).reserved_pixels(), Some(32));
    }

    #[test]
    fn an_absurd_reservation_stays_a_reservation() {
        // Nothing here narrows the value, so no backend's saturation can turn
        // a huge reservation into the sentinel that means the opposite.
        let huge = LayerReservation::Reserve(u32::MAX);
        assert_eq!(huge.reserved_pixels(), Some(u32::MAX));
        assert_ne!(huge, LayerReservation::Ignore);
    }
}
