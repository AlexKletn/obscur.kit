#![doc = include_str!("drag_region.doc/_module.md")]

use obscur_geometry::{Point, Rect};

#[doc = include_str!("drag_region.doc/DoubleClickAction.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DoubleClickAction {
    #[doc = include_str!("drag_region.doc/None.md")]
    None,
    #[doc = include_str!("drag_region.doc/Maximize.md")]
    Maximize,
    #[doc = include_str!("drag_region.doc/Minimize.md")]
    Minimize,
}

impl Default for DoubleClickAction {
    #[doc = include_str!("drag_region.doc/default.md")]
    fn default() -> Self {
        Self::None
    }
}

#[doc = include_str!("drag_region.doc/DragRegion.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DragRegion {
    #[doc = include_str!("drag_region.doc/bounds.md")]
    pub bounds: Rect,
    #[doc = include_str!("drag_region.doc/double_click_action.md")]
    pub double_click_action: DoubleClickAction,
}

impl DragRegion {
    #[doc = include_str!("drag_region.doc/new.md")]
    #[must_use]
    pub const fn new(bounds: Rect) -> Self {
        Self {
            bounds,
            double_click_action: DoubleClickAction::None,
        }
    }

    #[doc = include_str!("drag_region.doc/with_double_click_action.md")]
    #[must_use]
    pub const fn with_double_click_action(mut self, action: DoubleClickAction) -> Self {
        self.double_click_action = action;
        self
    }

    #[doc = include_str!("drag_region.doc/bounds_2.md")]
    #[must_use]
    pub const fn bounds(&self) -> Rect {
        self.bounds
    }

    #[doc = include_str!("drag_region.doc/double_click_action_2.md")]
    #[must_use]
    pub const fn double_click_action(&self) -> DoubleClickAction {
        self.double_click_action
    }

    #[doc = include_str!("drag_region.doc/contains.md")]
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
