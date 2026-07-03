//! Events a backend reports up to the application about a window (spec §6.1).

use obscur_geometry::{Point, ScaleFactor, Size};

use crate::window::WindowState;

/// An event describing a change to a window, reported by a backend up to the
/// application (spec §6.1).
///
/// These are the inter-window state and lifecycle notifications a backend
/// surfaces: creation and destruction, geometry and presentation changes, focus
/// transitions, scale-factor changes, and occlusion. Each variant is a plain,
/// backend-agnostic value.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WindowEvent {
    /// The window was created and is now available.
    Created,
    /// The user or system requested that the window close; the application
    /// decides whether to honor it.
    CloseRequested,
    /// The window was destroyed and is no longer valid.
    Destroyed,
    /// The window's content size changed to the given size.
    Resized(Size),
    /// The window's top-left position changed to the given point.
    Moved(Point),
    /// The window's presentation state changed to the given state.
    StateChanged(WindowState),
    /// The window gained input focus.
    FocusGained,
    /// The window lost input focus.
    FocusLost,
    /// The window's display scale factor changed to the given value.
    ScaleFactorChanged(ScaleFactor),
    /// The window's occlusion changed; `true` means it is now fully occluded.
    Occluded(bool),
}

impl WindowEvent {
    /// Returns the new size for a [`WindowEvent::Resized`] event, else `None`.
    #[must_use]
    pub const fn resized_size(&self) -> Option<Size> {
        match self {
            Self::Resized(size) => Some(*size),
            _ => None,
        }
    }

    /// Returns the new position for a [`WindowEvent::Moved`] event, else `None`.
    #[must_use]
    pub const fn moved_position(&self) -> Option<Point> {
        match self {
            Self::Moved(point) => Some(*point),
            _ => None,
        }
    }

    /// Returns the new state for a [`WindowEvent::StateChanged`] event, else
    /// `None`.
    #[must_use]
    pub const fn changed_state(&self) -> Option<WindowState> {
        match self {
            Self::StateChanged(state) => Some(*state),
            _ => None,
        }
    }

    /// Returns the new scale factor for a [`WindowEvent::ScaleFactorChanged`]
    /// event, else `None`.
    #[must_use]
    pub const fn changed_scale_factor(&self) -> Option<ScaleFactor> {
        match self {
            Self::ScaleFactorChanged(scale) => Some(*scale),
            _ => None,
        }
    }

    /// Returns whether this event requests that the window close.
    #[must_use]
    pub const fn is_close_requested(&self) -> bool {
        matches!(self, Self::CloseRequested)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resized_size_accessor() {
        let e = WindowEvent::Resized(Size::new(640.0, 480.0));
        assert_eq!(e.resized_size(), Some(Size::new(640.0, 480.0)));
        assert_eq!(WindowEvent::FocusGained.resized_size(), None);
    }

    #[test]
    fn moved_position_accessor() {
        let e = WindowEvent::Moved(Point::new(5.0, 6.0));
        assert_eq!(e.moved_position(), Some(Point::new(5.0, 6.0)));
        assert_eq!(WindowEvent::Destroyed.moved_position(), None);
    }

    #[test]
    fn changed_state_accessor() {
        let e = WindowEvent::StateChanged(WindowState::Fullscreen);
        assert_eq!(e.changed_state(), Some(WindowState::Fullscreen));
        assert_eq!(WindowEvent::Created.changed_state(), None);
    }

    #[test]
    fn changed_scale_factor_accessor() {
        let e = WindowEvent::ScaleFactorChanged(ScaleFactor::new(2.0));
        assert_eq!(e.changed_scale_factor(), Some(ScaleFactor::new(2.0)));
        assert_eq!(WindowEvent::FocusLost.changed_scale_factor(), None);
    }

    #[test]
    fn close_requested_predicate() {
        assert!(WindowEvent::CloseRequested.is_close_requested());
        assert!(!WindowEvent::Created.is_close_requested());
    }
}
