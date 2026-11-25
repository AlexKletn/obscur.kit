//! Everything an application says about a desktop surface before it opens.

use crate::display::DisplayId;

use super::{
    LayerAnchor, LayerBand, LayerEdge, LayerFocusPolicy, LayerMargins, LayerPointerPolicy,
    LayerReservation,
};

#[doc = include_str!("../../docs/api/layer/layer_settings.doc/LayerSettings.md")]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LayerSettings {
    /// Which stacking band the surface sits in.
    pub band: LayerBand,
    #[doc = include_str!("../../docs/api/layer/layer_settings.doc/anchor.md")]
    pub anchor: LayerAnchor,
    #[doc = include_str!("../../docs/api/layer/layer_settings.doc/width.md")]
    pub width: Option<u32>,
    #[doc = include_str!("../../docs/api/layer/layer_settings.doc/height.md")]
    pub height: Option<u32>,
    /// The space other windows are asked to keep clear.
    pub reservation: LayerReservation,
    /// The gap from each anchored edge.
    pub margin: LayerMargins,
    /// How much of the keyboard the surface asks for.
    pub focus: LayerFocusPolicy,
    /// Whether the pointer stops at the surface or passes through it.
    pub pointer: LayerPointerPolicy,
    #[doc = include_str!("../../docs/api/layer/layer_settings.doc/display.md")]
    pub display: Option<DisplayId>,
}

impl LayerSettings {
    #[doc = include_str!("../../docs/api/layer/layer_settings.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// A bar across the top of the screen, reserving its own height.
    #[must_use]
    pub fn top_bar(height: u32) -> Self {
        Self {
            anchor: LayerAnchor::TOP_BAR,
            height: Some(height),
            reservation: LayerReservation::Reserve(height),
            ..Self::new()
        }
    }

    /// A bar across the bottom of the screen, reserving its own height.
    #[must_use]
    pub fn bottom_bar(height: u32) -> Self {
        Self {
            anchor: LayerAnchor::BOTTOM_BAR,
            height: Some(height),
            reservation: LayerReservation::Reserve(height),
            ..Self::new()
        }
    }

    /// A bar down the left of the screen, reserving its own width.
    #[must_use]
    pub fn left_bar(width: u32) -> Self {
        Self {
            anchor: LayerAnchor::LEFT_BAR,
            width: Some(width),
            reservation: LayerReservation::Reserve(width),
            ..Self::new()
        }
    }

    /// A bar down the right of the screen, reserving its own width.
    #[must_use]
    pub fn right_bar(width: u32) -> Self {
        Self {
            anchor: LayerAnchor::RIGHT_BAR,
            width: Some(width),
            reservation: LayerReservation::Reserve(width),
            ..Self::new()
        }
    }

    /// A surface covering the whole screen, above everything, reserving nothing and
    /// ignoring what everyone else reserved.
    #[must_use]
    pub fn overlay() -> Self {
        Self {
            band: LayerBand::Overlay,
            anchor: LayerAnchor::ALL,
            reservation: LayerReservation::Ignore,
            ..Self::new()
        }
    }

    /// The same settings in a different stacking band.
    #[must_use]
    pub const fn with_band(mut self, band: LayerBand) -> Self {
        self.band = band;
        self
    }

    /// The same settings anchored to different edges.
    #[must_use]
    pub const fn with_anchor(mut self, anchor: LayerAnchor) -> Self {
        self.anchor = anchor;
        self
    }

    /// The same settings at an explicit size.
    #[must_use]
    pub const fn with_size(mut self, width: u32, height: u32) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }

    /// The same settings reserving differently.
    #[must_use]
    pub const fn with_reservation(mut self, reservation: LayerReservation) -> Self {
        self.reservation = reservation;
        self
    }

    /// The same settings with a different gap from the anchored edges.
    #[must_use]
    pub const fn with_margin(mut self, margin: LayerMargins) -> Self {
        self.margin = margin;
        self
    }

    /// The same settings with a different keyboard policy.
    #[must_use]
    pub const fn with_focus(mut self, focus: LayerFocusPolicy) -> Self {
        self.focus = focus;
        self
    }

    /// The same settings with a different pointer policy.
    #[must_use]
    pub const fn with_pointer(mut self, pointer: LayerPointerPolicy) -> Self {
        self.pointer = pointer;
        self
    }

    /// The same settings pinned to one display.
    #[must_use]
    pub const fn with_display(mut self, display: DisplayId) -> Self {
        self.display = Some(display);
        self
    }

    #[doc = include_str!("../../docs/api/layer/layer_settings.doc/docked_edge.md")]
    #[must_use]
    pub const fn docked_edge(&self) -> Option<LayerEdge> {
        self.anchor.docked_edge()
    }

    #[doc = include_str!("../../docs/api/layer/layer_settings.doc/needs_redraw_from.md")]
    #[must_use]
    pub fn needs_redraw_from(&self, previous: &Self) -> bool {
        // Only what changes the pixels. Moving a surface — a margin, a band, a
        // reservation, a keyboard policy — changes where it is, not what it
        // shows, and a drag is made of margins: answering each one with a frame
        // re-renders identical content sixty times a gesture.
        //
        // Extent is the exception, and the anchor is part of it: anchoring both
        // edges of an axis stretches the surface, so the extent follows from the
        // anchor as much as from the stated size. A different display may come
        // with a different scale, which is the same question one level up.
        self.width != previous.width
            || self.height != previous.height
            || self.anchor != previous.anchor
            || self.display != previous.display
    }

    #[doc = include_str!("../../docs/api/layer/layer_settings.doc/reserves_space.md")]
    #[must_use]
    pub const fn reserves_space(&self) -> bool {
        self.reservation.reserves() && self.anchor.docked_edge().is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_default_floats_in_the_panel_band_and_asks_for_nothing() {
        let settings = LayerSettings::new();
        assert_eq!(settings.band, LayerBand::Top);
        assert_eq!(settings.anchor, LayerAnchor::NONE);
        assert_eq!(settings.reservation, LayerReservation::None);
        assert_eq!(settings.focus, LayerFocusPolicy::Never);
        assert_eq!(settings.pointer, LayerPointerPolicy::Interactive);
        assert_eq!(settings.display, None);
        assert_eq!((settings.width, settings.height), (None, None));
    }

    #[test]
    fn an_axis_the_platform_should_decide_is_absent_not_zero() {
        // The sentinel this type exists to remove: a bar states its thickness
        // and says nothing at all about the axis it stretches along.
        let bar = LayerSettings::top_bar(36);
        assert_eq!(bar.height, Some(36));
        assert_eq!(bar.width, None);
        assert_ne!(bar.width, Some(0));
    }

    #[test]
    fn every_bar_reserves_its_own_thickness_along_its_own_edge() {
        for (settings, edge, thickness) in [
            (LayerSettings::top_bar(30), LayerEdge::Top, 30),
            (LayerSettings::bottom_bar(24), LayerEdge::Bottom, 24),
            (LayerSettings::left_bar(48), LayerEdge::Left, 48),
            (LayerSettings::right_bar(64), LayerEdge::Right, 64),
        ] {
            assert_eq!(settings.docked_edge(), Some(edge));
            assert_eq!(settings.reservation.reserved_pixels(), Some(thickness));
            assert!(settings.reserves_space());
        }
    }

    #[test]
    fn a_bar_stretches_along_the_axis_of_the_edge_it_docks_to() {
        let top = LayerSettings::top_bar(30);
        assert!(top.docked_edge().is_some_and(LayerEdge::is_horizontal));
        assert!(top.anchor.stretches_horizontally());

        let left = LayerSettings::left_bar(30);
        assert!(!left.docked_edge().is_some_and(LayerEdge::is_horizontal));
        assert!(left.anchor.stretches_vertically());
    }

    #[test]
    fn an_overlay_covers_everything_and_defers_to_no_one() {
        let overlay = LayerSettings::overlay();
        assert_eq!(overlay.band, LayerBand::Overlay);
        assert_eq!(overlay.anchor, LayerAnchor::ALL);
        assert_eq!(overlay.reservation, LayerReservation::Ignore);
        assert!(!overlay.reserves_space());
    }

    #[test]
    fn a_reservation_with_no_edge_to_measure_from_reserves_nothing() {
        // Both halves have to hold, and this is the half a caller gets wrong:
        // a number with no anchor has nothing to be perpendicular to.
        let floating = LayerSettings::new().with_reservation(LayerReservation::Reserve(40));
        assert!(floating.reservation.reserves());
        assert_eq!(floating.docked_edge(), None);
        assert!(!floating.reserves_space());
    }

    #[test]
    fn the_builders_change_one_thing_each() {
        let base = LayerSettings::top_bar(30);
        let moved = base
            .clone()
            .with_band(LayerBand::Overlay)
            .with_focus(LayerFocusPolicy::OnDemand)
            .with_pointer(LayerPointerPolicy::Transparent)
            .with_display(DisplayId::from_raw(7))
            .with_margin(LayerMargins::uniform(6));

        assert_eq!(moved.band, LayerBand::Overlay);
        assert_eq!(moved.focus, LayerFocusPolicy::OnDemand);
        assert_eq!(moved.pointer, LayerPointerPolicy::Transparent);
        assert_eq!(moved.display, Some(DisplayId::from_raw(7)));
        assert_eq!(moved.margin, LayerMargins::uniform(6));
        // Untouched by any of them.
        assert_eq!(moved.anchor, base.anchor);
        assert_eq!(moved.reservation, base.reservation);
        assert_eq!(moved.height, base.height);
    }

    #[test]
    fn moving_a_surface_does_not_change_what_it_draws() {
        // The whole of 2-H: a drag is a stream of margin changes, and none of
        // them alters a pixel of content.
        let docked = LayerSettings::top_bar(36);
        let nudged = docked.clone().with_margin(LayerMargins::uniform(12));
        assert!(!nudged.needs_redraw_from(&docked));

        for moved in [
            docked.clone().with_band(LayerBand::Background),
            docked.clone().with_reservation(LayerReservation::None),
            docked.clone().with_focus(LayerFocusPolicy::Exclusive),
            docked.clone().with_pointer(LayerPointerPolicy::Transparent),
        ] {
            assert!(
                !moved.needs_redraw_from(&docked),
                "{moved:?} changes placement or policy, not pixels"
            );
        }
    }

    #[test]
    fn changing_the_extent_does_need_a_frame() {
        let docked = LayerSettings::top_bar(36);
        assert!(LayerSettings::top_bar(48).needs_redraw_from(&docked));
        assert!(
            docked
                .clone()
                .with_size(320, 200)
                .needs_redraw_from(&docked),
            "an explicit size is an extent"
        );
    }

    #[test]
    fn the_anchor_counts_as_extent_because_it_stretches() {
        // `top_bar` and `left_bar` state the same 36 on different axes, and the
        // anchor is what decides which one the compositor stretches.
        let top = LayerSettings::top_bar(36);
        let left = LayerSettings::left_bar(36);
        assert_ne!(top.anchor, left.anchor);
        assert!(left.needs_redraw_from(&top));
    }

    #[test]
    fn a_different_display_needs_a_frame_because_it_may_scale_differently() {
        let docked = LayerSettings::top_bar(36);
        let moved = docked.clone().with_display(DisplayId::from_raw(2));
        assert!(moved.needs_redraw_from(&docked));
    }

    #[test]
    fn settings_that_did_not_change_need_nothing() {
        let docked = LayerSettings::top_bar(36);
        assert!(!docked.needs_redraw_from(&docked));
    }

    #[test]
    fn a_display_is_chosen_by_the_id_the_backend_already_publishes() {
        // Not a connector name: the same id a `WindowBackend` hands out.
        let pinned = LayerSettings::top_bar(30).with_display(DisplayId::from_raw(2));
        assert_eq!(pinned.display.map(DisplayId::raw), Some(2));
    }
}
