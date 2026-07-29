//! The depth band a layer surface occupies (spec §6.5).

/// Which of the compositor's four stacking bands a layer surface sits in.
///
/// The bands are ordered, and the order is the whole point: a compositor stacks
/// every surface in [`Background`](LayerKind::Background) below every surface in
/// [`Bottom`](LayerKind::Bottom), and so on up. Ordinary application windows sit
/// **between `Bottom` and `Top`**, which is what makes a panel in `Top` stay
/// above them and a wallpaper in `Background` stay below.
///
/// Choosing one is a policy decision, not a hint: a compositor honours it
/// exactly, and there is no negotiation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LayerKind {
    /// Below everything, including application windows. Wallpapers.
    Background,
    /// Below application windows but above the background.
    Bottom,
    /// Above application windows. Panels, bars, docks.
    #[default]
    Top,
    /// Above everything, including fullscreen windows. Notifications, on-screen
    /// keyboards, lock screens.
    Overlay,
}

impl LayerKind {
    /// Whether this band sits above ordinary application windows.
    ///
    /// The dividing line the model draws: application windows live between
    /// `Bottom` and `Top`, so this is `true` for exactly `Top` and `Overlay`.
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
