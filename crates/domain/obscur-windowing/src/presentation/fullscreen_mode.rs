#![doc = include_str!("fullscreen_mode.doc/_module.md")]

#[doc = include_str!("fullscreen_mode.doc/FullscreenMode.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FullscreenMode {
    #[doc = include_str!("fullscreen_mode.doc/Normal.md")]
    Normal,
    #[doc = include_str!("fullscreen_mode.doc/Borderless.md")]
    Borderless,
    #[doc = include_str!("fullscreen_mode.doc/Exclusive.md")]
    Exclusive,
    #[doc = include_str!("fullscreen_mode.doc/Kiosk.md")]
    Kiosk,
    #[doc = include_str!("fullscreen_mode.doc/Presentation.md")]
    Presentation,
}

impl FullscreenMode {
    #[doc = include_str!("fullscreen_mode.doc/is_fullscreen.md")]
    #[must_use]
    pub const fn is_fullscreen(self) -> bool {
        !matches!(self, Self::Normal)
    }
}

impl Default for FullscreenMode {
    #[doc = include_str!("fullscreen_mode.doc/default.md")]
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
