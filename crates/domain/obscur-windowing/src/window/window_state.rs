#![doc = include_str!("window_state.doc/_module.md")]

#[doc = include_str!("window_state.doc/WindowState.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WindowState {
    #[doc = include_str!("window_state.doc/Normal.md")]
    Normal,
    #[doc = include_str!("window_state.doc/Minimized.md")]
    Minimized,
    #[doc = include_str!("window_state.doc/Maximized.md")]
    Maximized,
    #[doc = include_str!("window_state.doc/Fullscreen.md")]
    Fullscreen,
}

impl WindowState {
    #[doc = include_str!("window_state.doc/is_normal.md")]
    #[must_use]
    pub const fn is_normal(self) -> bool {
        matches!(self, Self::Normal)
    }

    #[doc = include_str!("window_state.doc/is_minimized.md")]
    #[must_use]
    pub const fn is_minimized(self) -> bool {
        matches!(self, Self::Minimized)
    }

    #[doc = include_str!("window_state.doc/is_maximized.md")]
    #[must_use]
    pub const fn is_maximized(self) -> bool {
        matches!(self, Self::Maximized)
    }

    #[doc = include_str!("window_state.doc/is_fullscreen.md")]
    #[must_use]
    pub const fn is_fullscreen(self) -> bool {
        matches!(self, Self::Fullscreen)
    }
}

impl Default for WindowState {
    #[doc = include_str!("window_state.doc/default.md")]
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
