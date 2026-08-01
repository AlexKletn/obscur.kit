#![doc = include_str!("layer_kind.doc/_module.md")]

#[doc = include_str!("layer_kind.doc/LayerKind.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LayerKind {
    #[doc = include_str!("layer_kind.doc/Background.md")]
    Background,
    #[doc = include_str!("layer_kind.doc/Bottom.md")]
    Bottom,
    #[doc = include_str!("layer_kind.doc/Top.md")]
    #[default]
    Top,
    #[doc = include_str!("layer_kind.doc/Overlay.md")]
    Overlay,
}

impl LayerKind {
    #[doc = include_str!("layer_kind.doc/is_above_windows.md")]
    #[must_use]
    pub const fn is_above_windows(self) -> bool {
        matches!(self, Self::Top | Self::Overlay)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_two_upper_bands_are_above_application_windows() {
        assert!(LayerKind::Top.is_above_windows());
        assert!(LayerKind::Overlay.is_above_windows());
        assert!(!LayerKind::Bottom.is_above_windows());
        assert!(!LayerKind::Background.is_above_windows());
    }

    #[test]
    fn the_default_is_the_panel_band() {
        // A surface that says nothing about depth is asking to be a panel: above
        // the windows, below the notifications.
        assert_eq!(LayerKind::default(), LayerKind::Top);
    }
}
