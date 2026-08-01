#![doc = include_str!("window_event.doc/_module.md")]

use obscur_geometry::{Point, ScaleFactor, Size};

use crate::window::WindowState;

#[doc = include_str!("window_event.doc/WindowEvent.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WindowEvent {
    #[doc = include_str!("window_event.doc/Created.md")]
    Created,
    #[doc = include_str!("window_event.doc/CloseRequested.md")]
    CloseRequested,
    #[doc = include_str!("window_event.doc/Destroyed.md")]
    Destroyed,
    #[doc = include_str!("window_event.doc/Resized.md")]
    Resized(Size),
    #[doc = include_str!("window_event.doc/Moved.md")]
    Moved(Point),
    #[doc = include_str!("window_event.doc/StateChanged.md")]
    StateChanged(WindowState),
    #[doc = include_str!("window_event.doc/FocusGained.md")]
    FocusGained,
    #[doc = include_str!("window_event.doc/FocusLost.md")]
    FocusLost,
    #[doc = include_str!("window_event.doc/ScaleFactorChanged.md")]
    ScaleFactorChanged(ScaleFactor),
    #[doc = include_str!("window_event.doc/Occluded.md")]
    Occluded(bool),
}

impl WindowEvent {
    #[doc = include_str!("window_event.doc/resized_size.md")]
    #[must_use]
    pub const fn resized_size(&self) -> Option<Size> {
        match self {
            Self::Resized(size) => Some(*size),
            _ => None,
        }
    }

    #[doc = include_str!("window_event.doc/moved_position.md")]
    #[must_use]
    pub const fn moved_position(&self) -> Option<Point> {
        match self {
            Self::Moved(point) => Some(*point),
            _ => None,
        }
    }

    #[doc = include_str!("window_event.doc/changed_state.md")]
    #[must_use]
    pub const fn changed_state(&self) -> Option<WindowState> {
        match self {
            Self::StateChanged(state) => Some(*state),
            _ => None,
        }
    }

    #[doc = include_str!("window_event.doc/changed_scale_factor.md")]
    #[must_use]
    pub const fn changed_scale_factor(&self) -> Option<ScaleFactor> {
        match self {
            Self::ScaleFactorChanged(scale) => Some(*scale),
            _ => None,
        }
    }

    #[doc = include_str!("window_event.doc/is_close_requested.md")]
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
