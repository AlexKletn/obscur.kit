//! How a window's frame is provided: system, client-side, or custom chrome
//! (spec §6.4).

#[doc = include_str!("../../docs/api/chrome/chrome_mode.doc/ChromeMode.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChromeMode {
    /// Native window decorations drawn by the platform (the system-frame
    /// fallback).
    SystemFrame,
    /// Client-side decorations where the kit draws the standard controls.
    ClientSide,
    /// A fully custom title bar drawn entirely by the application.
    Custom,
}

impl ChromeMode {
    /// Returns whether this is a fully custom title bar.
    #[must_use]
    pub const fn is_custom(self) -> bool {
        matches!(self, Self::Custom)
    }

    #[doc = include_str!("../../docs/api/chrome/chrome_mode.doc/uses_client_side.md")]
    #[must_use]
    pub const fn uses_client_side(self) -> bool {
        matches!(self, Self::ClientSide | Self::Custom)
    }
}

impl Default for ChromeMode {
    /// The default chrome mode is [`ChromeMode::SystemFrame`], the safe
    /// system-frame fallback.
    fn default() -> Self {
        Self::SystemFrame
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_system_frame() {
        assert_eq!(ChromeMode::default(), ChromeMode::SystemFrame);
    }

    #[test]
    fn predicates() {
        assert!(ChromeMode::Custom.is_custom());
        assert!(!ChromeMode::ClientSide.is_custom());
        assert!(!ChromeMode::SystemFrame.is_custom());

        assert!(ChromeMode::ClientSide.uses_client_side());
        assert!(ChromeMode::Custom.uses_client_side());
        assert!(!ChromeMode::SystemFrame.uses_client_side());
    }
}
