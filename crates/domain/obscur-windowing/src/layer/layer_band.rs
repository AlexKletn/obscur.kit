//! The depth band a layer surface occupies.

#[doc = include_str!("../../docs/api/layer/layer_band.doc/LayerBand.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LayerBand {
    /// Below every application window — the wallpaper band.
    Background,
    /// Above the wallpaper, still below every application window.
    Bottom,
    #[doc = include_str!("../../docs/api/layer/layer_band.doc/Top.md")]
    #[default]
    Top,
    /// Above everything, including other layer surfaces — notifications, lock
    /// screens, on-screen keyboards.
    Overlay,
}

impl LayerBand {
    #[doc = include_str!("../../docs/api/layer/layer_band.doc/is_above_windows.md")]
    #[must_use]
    pub const fn is_above_windows(self) -> bool {
        matches!(self, Self::Top | Self::Overlay)
    }

    #[doc = include_str!("../../docs/api/layer/layer_band.doc/depth_order.md")]
    #[must_use]
    pub const fn depth_order(self) -> u8 {
        match self {
            Self::Background => 0,
            Self::Bottom => 1,
            Self::Top => 2,
            Self::Overlay => 3,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_two_upper_bands_are_above_application_windows() {
        assert!(LayerBand::Top.is_above_windows());
        assert!(LayerBand::Overlay.is_above_windows());
        assert!(!LayerBand::Bottom.is_above_windows());
        assert!(!LayerBand::Background.is_above_windows());
    }

    #[test]
    fn the_default_is_the_panel_band() {
        assert_eq!(LayerBand::default(), LayerBand::Top);
    }

    #[test]
    fn the_bands_are_ordered_from_wallpaper_to_overlay() {
        let mut previous = LayerBand::Background.depth_order();
        for band in [LayerBand::Bottom, LayerBand::Top, LayerBand::Overlay] {
            assert!(
                band.depth_order() > previous,
                "{band:?} must sit above the band below it"
            );
            previous = band.depth_order();
        }
    }

    #[test]
    fn the_split_between_the_bands_is_where_windows_live() {
        // A backend that cannot name four bands still has to place a surface
        // above or below ordinary windows, and that is the only cut it needs.
        assert!(LayerBand::Bottom.depth_order() < LayerBand::Top.depth_order());
        assert!(!LayerBand::Bottom.is_above_windows());
        assert!(LayerBand::Top.is_above_windows());
    }
}
