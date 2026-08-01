#![doc = include_str!("system_menu_request.doc/_module.md")]

use obscur_geometry::Point;

#[doc = include_str!("system_menu_request.doc/SystemMenuRequest.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SystemMenuRequest {
    #[doc = include_str!("system_menu_request.doc/position.md")]
    pub position: Point,
}

impl SystemMenuRequest {
    #[doc = include_str!("system_menu_request.doc/at.md")]
    #[must_use]
    pub const fn at(position: Point) -> Self {
        Self { position }
    }

    #[doc = include_str!("system_menu_request.doc/position_2.md")]
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
