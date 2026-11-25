//! A semantic window-drag area and its double-click action (spec §6.4).

use obscur_geometry::{Point, Rect};

use super::DoubleClickAction;

#[doc = include_str!("../../docs/api/chrome/drag_region.doc/DragRegion.md")]
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
