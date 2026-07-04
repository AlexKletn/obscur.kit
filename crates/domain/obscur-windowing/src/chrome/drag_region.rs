//! A semantic window-drag area and its double-click action (spec §6.4).

use obscur_geometry::{Point, Rect};

/// The action taken when the user double-clicks a drag region (spec §6.4).
///
/// This is the platform-conventional title-bar double-click action.
/// `DoubleClickAction` is a tightly-coupled part of [`DragRegion`]: it only has
/// meaning as the action of a drag area, so it is colocated here rather than
/// given its own module.
///
/// The enum's own default is [`DoubleClickAction::None`] so that a bare value
/// is inert. A freshly constructed [`DragRegion`] likewise starts with `None`;
/// callers pick the platform-conventional [`DoubleClickAction::Maximize`]
/// explicitly via [`DragRegion::with_double_click_action`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DoubleClickAction {
    /// Double-clicking does nothing.
    None,
    /// Double-clicking toggles the window maximized (the conventional action on
    /// most platforms).
    Maximize,
    /// Double-clicking minimizes the window.
    Minimize,
}

impl Default for DoubleClickAction {
    /// The default action is [`DoubleClickAction::None`].
    fn default() -> Self {
        Self::None
    }
}

/// A semantic window-drag area, such as a custom title bar (spec §6.4).
///
/// A compositor treats clicks within [`Self::bounds`] as a request to move the
/// window, and double-clicks as the configured [`DoubleClickAction`].
/// Coordinates are in window-local logical pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DragRegion {
    /// The window-local area treated as draggable.
    pub bounds: Rect,
    /// The action taken on a double-click within the region.
    pub double_click_action: DoubleClickAction,
}

impl DragRegion {
    /// Creates a drag region over `bounds` with no double-click action.
    #[must_use]
    pub const fn new(bounds: Rect) -> Self {
        Self {
            bounds,
            double_click_action: DoubleClickAction::None,
        }
    }

    /// Sets the double-click action, returning the updated region.
    #[must_use]
    pub const fn with_double_click_action(mut self, action: DoubleClickAction) -> Self {
        self.double_click_action = action;
        self
    }

    /// Returns the window-local draggable area.
    #[must_use]
    pub const fn bounds(&self) -> Rect {
        self.bounds
    }

    /// Returns the action taken on a double-click within the region.
    #[must_use]
    pub const fn double_click_action(&self) -> DoubleClickAction {
        self.double_click_action
    }

    /// Returns whether the window-local point lies within the drag region.
    #[must_use]
    pub fn contains(&self, point: Point) -> bool {
        self.bounds.contains(point)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_geometry::Size;

    fn a_rect() -> Rect {
        Rect::new(Point::new(0.0, 0.0), Size::new(200.0, 30.0))
    }

    #[test]
    fn new_has_no_double_click_action() {
        let region = DragRegion::new(a_rect());
        assert_eq!(region.double_click_action(), DoubleClickAction::None);
        assert_eq!(region.bounds(), a_rect());
        assert_eq!(DoubleClickAction::default(), DoubleClickAction::None);
    }

    #[test]
    fn with_double_click_action_sets_it() {
        let region =
            DragRegion::new(a_rect()).with_double_click_action(DoubleClickAction::Maximize);
        assert_eq!(region.double_click_action(), DoubleClickAction::Maximize);
    }

    #[test]
    fn contains_checks_bounds() {
        let region = DragRegion::new(a_rect());
        assert!(region.contains(Point::new(10.0, 10.0)));
        assert!(!region.contains(Point::new(10.0, 40.0)));
    }
}
