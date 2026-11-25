//! A request to show the OS system/window menu at a point (spec §6.4).

use obscur_geometry::Point;

#[doc = include_str!("../../docs/api/chrome/system_menu_request.doc/SystemMenuRequest.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SystemMenuRequest {
    /// The window-local point at which to pop the system menu.
    pub position: Point,
}

impl SystemMenuRequest {
    /// Creates a request to show the system menu at `position`.
    #[must_use]
    pub const fn at(position: Point) -> Self {
        Self { position }
    }

    /// Returns the window-local point at which to pop the menu.
    #[must_use]
    pub const fn position(&self) -> Point {
        self.position
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn at_stores_position() {
        let request = SystemMenuRequest::at(Point::new(12.0, 4.0));
        assert_eq!(request.position(), Point::new(12.0, 4.0));
    }
}
