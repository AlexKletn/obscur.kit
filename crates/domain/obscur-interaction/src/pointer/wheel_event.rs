#![doc = include_str!("wheel_event.doc/_module.md")]

use obscur_geometry::Point;

#[doc = include_str!("wheel_event.doc/WheelEvent.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WheelEvent {
    #[doc = include_str!("wheel_event.doc/position.md")]
    pub position: Point,
    #[doc = include_str!("wheel_event.doc/delta_y.md")]
    pub delta_y: f32,
}

impl WheelEvent {
    #[doc = include_str!("wheel_event.doc/new.md")]
    #[must_use]
    pub const fn new(position: Point, delta_y: f32) -> Self {
        Self { position, delta_y }
    }

    #[doc = include_str!("wheel_event.doc/position_2.md")]
    #[must_use]
    pub const fn position(&self) -> Point {
        self.position
    }

    #[doc = include_str!("wheel_event.doc/delta_y_2.md")]
    #[must_use]
    pub const fn delta_y(&self) -> f32 {
        self.delta_y
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_notch_reports_its_delta_and_where_it_was_aimed() {
        let wheel = WheelEvent::new(Point::new(12.0, 34.0), 40.0);
        assert_eq!(wheel.position(), Point::new(12.0, 34.0));
        assert_eq!(wheel.delta_y(), 40.0);
    }

    /// The sign is the whole meaning of the delta, and nothing normalises it on
    /// the way in — a handler reading `delta_y` reads exactly what the container
    /// would have been scrolled by.
    #[test]
    fn the_sign_survives() {
        assert_eq!(
            WheelEvent::new(Point::new(0.0, 0.0), -25.0).delta_y(),
            -25.0
        );
    }
}
