//! The fullscreen and presentation display modes a window can take (spec §6.5).

/// How a window occupies the screen when presented (spec §6.5).
///
/// [`FullscreenMode::Normal`] is the windowed baseline — not fullscreen at all.
/// The remaining variants are the fullscreen forms;
/// [`FullscreenMode::Exclusive`] is honored only where the platform offers a
/// true exclusive mode, and otherwise degrades to a borderless equivalent.
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
    /// Returns whether this mode is any form of fullscreen.
    ///
    /// This is `true` for every variant except [`FullscreenMode::Normal`].
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
