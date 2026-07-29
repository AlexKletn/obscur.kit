//! The complete description of a layer surface (spec §6.5).

use obscur_geometry::Size;

use super::{LayerAnchor, LayerExclusiveZone, LayerKeyboardInteractivity, LayerKind, LayerOutput};

/// Everything a compositor needs to place a layer surface, as plain data.
///
/// This is the whole request. It carries no connection, no protocol object and
/// no platform type, which is what lets it be built, inspected and tested
/// without a Wayland session — and what lets a non-Wayland backend refuse it
/// honestly rather than approximate it.
///
/// # Size and anchoring interact
///
/// The requested [`size`](Self::size) is only honoured on an axis the surface is
/// **not** stretched along. A top bar anchored left+right gets the output's
/// width whatever it asked for; only its height comes from here. A surface must
/// therefore take its real extent from the configure event, never from this
/// request. See [`LayerAnchor::stretches_horizontally`].
///
/// A zero on a stretched axis is the idiomatic way to say "you decide".
#[derive(Debug, Clone, PartialEq)]
pub struct LayerSurfaceConfig {
    /// The compositor's identifier for this surface, used by its rules and by
    /// the user's configuration to target it. Conventionally short and stable,
    /// e.g. `"panel"`.
    pub namespace: String,
    /// Which stacking band the surface sits in.
    pub layer: LayerKind,
    /// Which edges it is pinned to — and, for opposite pairs, stretched between.
    pub anchor: LayerAnchor,
    /// The size wanted on any axis the anchoring does not decide, in logical
    /// pixels.
    pub size: Size,
    /// How much space to reserve from other surfaces.
    pub exclusive_zone: LayerExclusiveZone,
    /// Gaps between the surface and the edges it is anchored to, in logical
    /// pixels. An unanchored edge's margin is ignored.
    pub margin: Margins,
    /// Whether the surface takes keyboard focus.
    pub keyboard_interactivity: LayerKeyboardInteractivity,
    /// Which output to place the surface on.
    pub output: LayerOutput,
}

/// Per-edge gaps between a layer surface and the edges it is anchored to.
///
/// Separate from [`Insets`](obscur_geometry::Insets) because these are **integer
/// logical pixels** in the protocol, and rounding a float at the boundary is a
/// decision better made once, here, than at every call site.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Margins {
    /// Gap from the top edge.
    pub top: i32,
    /// Gap from the right edge.
    pub right: i32,
    /// Gap from the bottom edge.
    pub bottom: i32,
    /// Gap from the left edge.
    pub left: i32,
}

impl Margins {
    /// The same gap on every edge.
    #[must_use]
    pub const fn uniform(px: i32) -> Self {
        Self {
            top: px,
            right: px,
            bottom: px,
            left: px,
        }
    }
}

impl LayerSurfaceConfig {
    /// A surface in the given namespace, with every other choice at its default:
    /// the `Top` band, no anchor, no reservation, no keyboard.
    ///
    /// Unanchored and with a zero size this asks the compositor for a centred
    /// surface of its own choosing, which is a legal request but rarely the
    /// intended one — the constructors below are.
    #[must_use]
    pub fn new(namespace: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
            layer: LayerKind::default(),
            anchor: LayerAnchor::NONE,
            size: Size::new(0.0, 0.0),
            exclusive_zone: LayerExclusiveZone::default(),
            margin: Margins::default(),
            keyboard_interactivity: LayerKeyboardInteractivity::default(),
            output: LayerOutput::default(),
        }
    }

    /// A bar across the top of the output, `height` logical pixels tall,
    /// reserving exactly that much space from everything else.
    ///
    /// The reservation is the difference between a bar and a floating strip that
    /// maximised windows slide underneath, so it is part of the constructor
    /// rather than something to remember afterwards.
    #[must_use]
    pub fn top_bar(namespace: impl Into<String>, height: u32) -> Self {
        Self {
            anchor: LayerAnchor::TOP_BAR,
            #[allow(clippy::cast_precision_loss)]
            size: Size::new(0.0, height as f32),
            exclusive_zone: LayerExclusiveZone::Reserve(height),
            ..Self::new(namespace)
        }
    }

    /// A bar across the bottom of the output, reserving its own height.
    #[must_use]
    pub fn bottom_bar(namespace: impl Into<String>, height: u32) -> Self {
        Self {
            anchor: LayerAnchor::BOTTOM_BAR,
            #[allow(clippy::cast_precision_loss)]
            size: Size::new(0.0, height as f32),
            exclusive_zone: LayerExclusiveZone::Reserve(height),
            ..Self::new(namespace)
        }
    }

    /// A full-output overlay above everything, reserving nothing and ignoring
    /// every other reservation — the lock-screen / on-screen-keyboard shape.
    #[must_use]
    pub fn overlay(namespace: impl Into<String>) -> Self {
        Self {
            layer: LayerKind::Overlay,
            anchor: LayerAnchor::ALL,
            exclusive_zone: LayerExclusiveZone::Ignore,
            ..Self::new(namespace)
        }
    }

    /// Sets the stacking band.
    #[must_use]
    pub const fn with_layer(mut self, layer: LayerKind) -> Self {
        self.layer = layer;
        self
    }

    /// Sets the anchored edges.
    #[must_use]
    pub const fn with_anchor(mut self, anchor: LayerAnchor) -> Self {
        self.anchor = anchor;
        self
    }

    /// Sets the requested size, in logical pixels.
    #[must_use]
    pub const fn with_size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }

    /// Sets how much space to reserve.
    #[must_use]
    pub const fn with_exclusive_zone(mut self, zone: LayerExclusiveZone) -> Self {
        self.exclusive_zone = zone;
        self
    }

    /// Sets the per-edge margins.
    #[must_use]
    pub const fn with_margin(mut self, margin: Margins) -> Self {
        self.margin = margin;
        self
    }

    /// Places the surface on a particular output, by connector name.
    ///
    /// A name that matches nothing connected falls back to the compositor's
    /// choice rather than failing — a panel must still appear when a monitor is
    /// unplugged.
    #[must_use]
    pub fn with_output(mut self, output: LayerOutput) -> Self {
        self.output = output;
        self
    }

    /// Sets whether the surface takes keyboard focus.
    #[must_use]
    pub const fn with_keyboard_interactivity(
        mut self,
        interactivity: LayerKeyboardInteractivity,
    ) -> Self {
        self.keyboard_interactivity = interactivity;
        self
    }

    /// The size the surface should *request*, with a zero on every axis the
    /// anchoring stretches — the encoding a compositor reads as "you decide".
    ///
    /// Asking for a width while anchored to both side edges is not an error and
    /// not honoured; normalising it here means the request sent on the wire says
    /// what will actually happen, so a surface that later disagrees with its
    /// configure event has a real reason to.
    #[must_use]
    pub fn requested_size(&self) -> (u32, u32) {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let width = if self.anchor.stretches_horizontally() {
            0
        } else {
            self.size.width.max(0.0) as u32
        };
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let height = if self.anchor.stretches_vertically() {
            0
        } else {
            self.size.height.max(0.0) as u32
        };
        (width, height)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_top_bar_reserves_exactly_its_height() {
        let bar = LayerSurfaceConfig::top_bar("panel", 32);
        assert_eq!(bar.exclusive_zone, LayerExclusiveZone::Reserve(32));
        assert!(bar.anchor.stretches_horizontally());
        assert_eq!(bar.layer, LayerKind::Top);
    }

    /// The interaction the docs warn about, pinned: a stretched axis requests
    /// zero however large a size was set, so the wire request and the outcome
    /// agree.
    #[test]
    fn a_stretched_axis_requests_zero_whatever_the_size_says() {
        let bar = LayerSurfaceConfig::top_bar("panel", 32).with_size(Size::new(500.0, 32.0));
        assert_eq!(bar.requested_size(), (0, 32));

        let side = LayerSurfaceConfig::new("dock")
            .with_anchor(LayerAnchor::LEFT_BAR)
            .with_size(Size::new(64.0, 900.0));
        assert_eq!(side.requested_size(), (64, 0));
    }

    #[test]
    fn an_unanchored_surface_requests_both_axes() {
        let floating = LayerSurfaceConfig::new("popup").with_size(Size::new(300.0, 200.0));
        assert_eq!(floating.requested_size(), (300, 200));
    }

    #[test]
    fn a_negative_size_requests_zero_rather_than_wrapping() {
        let odd = LayerSurfaceConfig::new("x").with_size(Size::new(-10.0, -1.0));
        assert_eq!(odd.requested_size(), (0, 0));
    }

    #[test]
    fn an_overlay_ignores_every_other_reservation() {
        let overlay = LayerSurfaceConfig::overlay("lock");
        assert_eq!(overlay.layer, LayerKind::Overlay);
        assert_eq!(overlay.exclusive_zone, LayerExclusiveZone::Ignore);
        assert_eq!(
            overlay.requested_size(),
            (0, 0),
            "anchored to all four edges"
        );
    }

    #[test]
    fn an_output_choice_defaults_to_the_compositors() {
        let config = LayerSurfaceConfig::new("panel");
        assert!(config.output.is_compositor_choice());
        let pinned = config.with_output(LayerOutput::Named("eDP-1".to_owned()));
        assert_eq!(pinned.output.name(), Some("eDP-1"));
    }

    #[test]
    fn the_builders_compose() {
        let config = LayerSurfaceConfig::new("kbd")
            .with_layer(LayerKind::Overlay)
            .with_anchor(LayerAnchor::BOTTOM_BAR)
            .with_size(Size::new(0.0, 240.0))
            .with_exclusive_zone(LayerExclusiveZone::Reserve(240))
            .with_margin(Margins::uniform(8))
            .with_keyboard_interactivity(LayerKeyboardInteractivity::None);
        assert_eq!(config.namespace, "kbd");
        assert_eq!(config.margin.left, 8);
        assert_eq!(config.requested_size(), (0, 240));
    }
}
