//! Presentation and lifecycle state of a window (spec §6.2).

#[doc = include_str!("../../docs/api/window/window_state.doc/WindowState.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WindowState {
    /// A regular, restored window sized by its properties.
    Normal,
    /// Iconified/hidden to the taskbar or dock.
    Minimized,
    /// Expanded to fill the available work area (excluding system chrome).
    Maximized,
    /// Occupying the entire display, typically without decorations.
    Fullscreen,
}

impl WindowState {
    /// Returns whether the window is in the [`WindowState::Normal`] state.
    #[must_use]
    pub const fn is_normal(self) -> bool {
        matches!(self, Self::Normal)
    }

    /// Returns whether the window is minimized.
    #[must_use]
    pub const fn is_minimized(self) -> bool {
        matches!(self, Self::Minimized)
    }

    /// Returns whether the window is maximized.
    #[must_use]
    pub const fn is_maximized(self) -> bool {
        matches!(self, Self::Maximized)
    }

    /// Returns whether the window is fullscreen.
    #[must_use]
    pub const fn is_fullscreen(self) -> bool {
        matches!(self, Self::Fullscreen)
    }
}

impl Default for WindowState {
    /// The default presentation state is [`WindowState::Normal`].
    fn default() -> Self {
        Self::Normal
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_normal() {
        assert_eq!(WindowState::default(), WindowState::Normal);
    }

    #[test]
    fn predicates() {
        assert!(WindowState::Normal.is_normal());
        assert!(WindowState::Minimized.is_minimized());
        assert!(WindowState::Maximized.is_maximized());
        assert!(WindowState::Fullscreen.is_fullscreen());
        assert!(!WindowState::Normal.is_fullscreen());
    }
}
