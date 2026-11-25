//! The fullscreen and presentation display modes a window can take (spec §6.5).

#[doc = include_str!("../../docs/api/presentation/fullscreen_mode.doc/FullscreenMode.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FullscreenMode {
    /// A normal windowed presentation; not fullscreen.
    Normal,
    /// A borderless window covering the display.
    Borderless,
    /// True exclusive fullscreen, where the platform makes it available.
    Exclusive,
    /// A locked-down kiosk presentation.
    Kiosk,
    /// A presentation mode intended for slideshows and similar full-screen
    /// content.
    Presentation,
}

impl FullscreenMode {
    #[doc = include_str!("../../docs/api/presentation/fullscreen_mode.doc/is_fullscreen.md")]
    #[must_use]
    pub const fn is_fullscreen(self) -> bool {
        !matches!(self, Self::Normal)
    }
}

impl Default for FullscreenMode {
    /// The default mode is [`FullscreenMode::Normal`], the windowed baseline.
    fn default() -> Self {
        Self::Normal
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_normal() {
        assert_eq!(FullscreenMode::default(), FullscreenMode::Normal);
    }

    #[test]
    fn is_fullscreen_excludes_normal() {
        assert!(!FullscreenMode::Normal.is_fullscreen());
        assert!(FullscreenMode::Borderless.is_fullscreen());
        assert!(FullscreenMode::Exclusive.is_fullscreen());
        assert!(FullscreenMode::Kiosk.is_fullscreen());
        assert!(FullscreenMode::Presentation.is_fullscreen());
    }
}
