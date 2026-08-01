#![doc = include_str!("layer_surface_config.doc/_module.md")]

use obscur_geometry::Size;

use super::{LayerAnchor, LayerExclusiveZone, LayerKeyboardInteractivity, LayerKind, LayerOutput};

#[doc = include_str!("layer_surface_config.doc/LayerSurfaceConfig.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct LayerSurfaceConfig {
    #[doc = include_str!("layer_surface_config.doc/namespace.md")]
    pub namespace: String,
    #[doc = include_str!("layer_surface_config.doc/layer.md")]
    pub layer: LayerKind,
    #[doc = include_str!("layer_surface_config.doc/anchor.md")]
    pub anchor: LayerAnchor,
    #[doc = include_str!("layer_surface_config.doc/size.md")]
    pub size: Size,
    #[doc = include_str!("layer_surface_config.doc/exclusive_zone.md")]
    pub exclusive_zone: LayerExclusiveZone,
    #[doc = include_str!("layer_surface_config.doc/margin.md")]
    pub margin: Margins,
    #[doc = include_str!("layer_surface_config.doc/keyboard_interactivity.md")]
    pub keyboard_interactivity: LayerKeyboardInteractivity,
    #[doc = include_str!("layer_surface_config.doc/output.md")]
    pub output: LayerOutput,
}

#[doc = include_str!("layer_surface_config.doc/Margins.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Margins {
    #[doc = include_str!("layer_surface_config.doc/top.md")]
    pub top: i32,
    #[doc = include_str!("layer_surface_config.doc/right.md")]
    pub right: i32,
    #[doc = include_str!("layer_surface_config.doc/bottom.md")]
    pub bottom: i32,
    #[doc = include_str!("layer_surface_config.doc/left.md")]
    pub left: i32,
}

impl Margins {
    #[doc = include_str!("layer_surface_config.doc/uniform.md")]
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
    #[doc = include_str!("layer_surface_config.doc/new.md")]
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

    #[doc = include_str!("layer_surface_config.doc/top_bar.md")]
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

    #[doc = include_str!("layer_surface_config.doc/bottom_bar.md")]
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

    #[doc = include_str!("layer_surface_config.doc/overlay.md")]
    #[must_use]
    pub fn overlay(namespace: impl Into<String>) -> Self {
        Self {
            layer: LayerKind::Overlay,
            anchor: LayerAnchor::ALL,
            exclusive_zone: LayerExclusiveZone::Ignore,
            ..Self::new(namespace)
        }
    }

    #[doc = include_str!("layer_surface_config.doc/with_layer.md")]
    #[must_use]
    pub const fn with_layer(mut self, layer: LayerKind) -> Self {
        self.layer = layer;
        self
    }

    #[doc = include_str!("layer_surface_config.doc/with_anchor.md")]
    #[must_use]
    pub const fn with_anchor(mut self, anchor: LayerAnchor) -> Self {
        self.anchor = anchor;
        self
    }

    #[doc = include_str!("layer_surface_config.doc/with_size.md")]
    #[must_use]
    pub const fn with_size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }

    #[doc = include_str!("layer_surface_config.doc/with_exclusive_zone.md")]
    #[must_use]
    pub const fn with_exclusive_zone(mut self, zone: LayerExclusiveZone) -> Self {
        self.exclusive_zone = zone;
        self
    }

    #[doc = include_str!("layer_surface_config.doc/with_margin.md")]
    #[must_use]
    pub const fn with_margin(mut self, margin: Margins) -> Self {
        self.margin = margin;
        self
    }

    #[doc = include_str!("layer_surface_config.doc/with_output.md")]
    #[must_use]
    pub fn with_output(mut self, output: LayerOutput) -> Self {
        self.output = output;
        self
    }

    #[doc = include_str!("layer_surface_config.doc/with_keyboard_interactivity.md")]
    #[must_use]
    pub const fn with_keyboard_interactivity(
        mut self,
        interactivity: LayerKeyboardInteractivity,
    ) -> Self {
        self.keyboard_interactivity = interactivity;
        self
    }

    #[doc = include_str!("layer_surface_config.doc/requested_size.md")]
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
