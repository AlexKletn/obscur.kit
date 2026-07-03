//! How a window's frame is provided: system, client-side, or custom chrome
//! (spec §6.4).

/// How the window frame and its controls are provided (spec §6.4).
///
/// This selects who draws the window's decorations. Dynamic switching between
/// modes is allowed where the platform supports it; where it does not,
/// [`ChromeMode::SystemFrame`] is the safe fallback and is therefore the
/// default.
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

    /// Returns whether the frame is drawn client-side rather than by the
    /// platform.
    ///
    /// This is `true` for both [`ChromeMode::ClientSide`] and
    /// [`ChromeMode::Custom`], since both draw their own decorations.
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
