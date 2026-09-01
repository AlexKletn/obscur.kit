#![doc = include_str!("wlr_mapper.doc/_module.md")]

use obscur_windowing::layer::{
    LayerAnchor, LayerBand, LayerFocusPolicy, LayerReservation, LayerSettings,
};
use smithay_client_toolkit::shell::wlr_layer::{Anchor, KeyboardInteractivity, Layer};

#[doc = include_str!("wlr_mapper.doc/WlrMapper.md")]
#[derive(Debug)]
pub(crate) struct WlrMapper;

impl WlrMapper {
    #[doc = include_str!("wlr_mapper.doc/layer.md")]
    pub(crate) const fn layer(band: LayerBand) -> Layer {
        match band {
            LayerBand::Background => Layer::Background,
            LayerBand::Bottom => Layer::Bottom,
            LayerBand::Top => Layer::Top,
            LayerBand::Overlay => Layer::Overlay,
        }
    }

    #[doc = include_str!("wlr_mapper.doc/anchor.md")]
    pub(crate) fn anchor(anchor: LayerAnchor) -> Anchor {
        let mut out = Anchor::empty();
        if anchor.top {
            out |= Anchor::TOP;
        }
        if anchor.right {
            out |= Anchor::RIGHT;
        }
        if anchor.bottom {
            out |= Anchor::BOTTOM;
        }
        if anchor.left {
            out |= Anchor::LEFT;
        }
        out
    }

    #[doc = include_str!("wlr_mapper.doc/keyboard.md")]
    pub(crate) const fn keyboard(focus: LayerFocusPolicy) -> KeyboardInteractivity {
        match focus {
            LayerFocusPolicy::Never => KeyboardInteractivity::None,
            LayerFocusPolicy::OnDemand => KeyboardInteractivity::OnDemand,
            LayerFocusPolicy::Exclusive => KeyboardInteractivity::Exclusive,
        }
    }

    #[doc = include_str!("wlr_mapper.doc/exclusive_zone.md")]
    pub(crate) const fn exclusive_zone(reservation: LayerReservation) -> i32 {
        match reservation {
            LayerReservation::None => 0,
            LayerReservation::Ignore => -1,
            LayerReservation::Reserve(px) => {
                if px > i32::MAX.cast_unsigned() {
                    i32::MAX
                } else {
                    px.cast_signed()
                }
            }
        }
    }

    #[doc = include_str!("wlr_mapper.doc/requested_size.md")]
    pub(crate) const fn requested_size(settings: &LayerSettings) -> (u32, u32) {
        // Zero is the protocol's way of saying "you decide", and an axis the
        // anchor stretches is always the compositor's to decide — a width sent
        // alongside a left-and-right anchor is a contradiction the compositor
        // resolves in the anchor's favour anyway.
        let width = if settings.anchor.stretches_horizontally() {
            0
        } else {
            match settings.width {
                Some(px) => px,
                None => 0,
            }
        };
        let height = if settings.anchor.stretches_vertically() {
            0
        } else {
            match settings.height {
                Some(px) => px,
                None => 0,
            }
        };
        (width, height)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // These mappings are the one place a wrong pairing would put a panel in the
    // wrong band, or hand it the wrong strip of screen, with no error anywhere.

    #[test]
    fn every_band_maps_to_its_own_protocol_layer() {
        assert!(matches!(
            WlrMapper::layer(LayerBand::Background),
            Layer::Background
        ));
        assert!(matches!(WlrMapper::layer(LayerBand::Bottom), Layer::Bottom));
        assert!(matches!(WlrMapper::layer(LayerBand::Top), Layer::Top));
        assert!(matches!(
            WlrMapper::layer(LayerBand::Overlay),
            Layer::Overlay
        ));
    }

    #[test]
    fn a_top_bar_anchors_to_three_edges() {
        let anchor = WlrMapper::anchor(LayerAnchor::TOP_BAR);
        assert!(anchor.contains(Anchor::TOP));
        assert!(anchor.contains(Anchor::LEFT));
        assert!(anchor.contains(Anchor::RIGHT));
        assert!(!anchor.contains(Anchor::BOTTOM));
    }

    #[test]
    fn an_empty_anchor_maps_to_no_bits() {
        assert!(WlrMapper::anchor(LayerAnchor::NONE).is_empty());
        assert_eq!(WlrMapper::anchor(LayerAnchor::ALL), Anchor::all());
    }

    #[test]
    fn keyboard_modes_map_one_to_one() {
        assert!(matches!(
            WlrMapper::keyboard(LayerFocusPolicy::Never),
            KeyboardInteractivity::None
        ));
        assert!(matches!(
            WlrMapper::keyboard(LayerFocusPolicy::OnDemand),
            KeyboardInteractivity::OnDemand
        ));
        assert!(matches!(
            WlrMapper::keyboard(LayerFocusPolicy::Exclusive),
            KeyboardInteractivity::Exclusive
        ));
    }

    #[test]
    fn the_three_reservations_encode_to_the_protocols_three_signs() {
        // This encoding used to live in the domain type, where `-1` read as a
        // quantity. It is protocol, so it lives here.
        assert_eq!(WlrMapper::exclusive_zone(LayerReservation::None), 0);
        assert_eq!(WlrMapper::exclusive_zone(LayerReservation::Ignore), -1);
        assert_eq!(WlrMapper::exclusive_zone(LayerReservation::Reserve(32)), 32);
    }

    #[test]
    fn an_absurd_reservation_saturates_rather_than_becoming_ignore() {
        let huge = WlrMapper::exclusive_zone(LayerReservation::Reserve(u32::MAX));
        assert_eq!(huge, i32::MAX);
        assert_ne!(huge, -1, "a wrapped reservation must never mean ignore");
    }

    #[test]
    fn a_bar_leaves_the_stretched_axis_to_the_compositor() {
        assert_eq!(
            WlrMapper::requested_size(&LayerSettings::top_bar(36)),
            (0, 36)
        );
        assert_eq!(
            WlrMapper::requested_size(&LayerSettings::left_bar(48)),
            (48, 0)
        );
    }

    #[test]
    fn an_unset_axis_is_the_compositors_to_choose() {
        assert_eq!(WlrMapper::requested_size(&LayerSettings::new()), (0, 0));
    }

    #[test]
    fn an_explicit_size_survives_when_the_anchor_does_not_stretch() {
        let floating = LayerSettings::new().with_size(320, 200);
        assert_eq!(WlrMapper::requested_size(&floating), (320, 200));
    }

    #[test]
    fn an_anchor_that_stretches_wins_over_a_size_that_disagrees() {
        let contradiction = LayerSettings::top_bar(36).with_size(640, 36);
        assert_eq!(WlrMapper::requested_size(&contradiction), (0, 36));
    }
}
