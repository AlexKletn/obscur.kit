#![doc = include_str!("wheel_event.doc/_module.md")]

use obscur_geometry::Point;

use super::WheelDeltaMode;
use crate::event::EventTrust;

#[doc = include_str!("wheel_event.doc/WheelEvent.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WheelEvent {
    #[doc = include_str!("wheel_event.doc/position.md")]
    pub position: Point,
    #[doc = include_str!("wheel_event.doc/delta_y.md")]
    pub delta_y: f32,
    #[doc = include_str!("wheel_event.doc/delta_x.md")]
    pub delta_x: f32,
    #[doc = include_str!("wheel_event.doc/mode.md")]
    pub mode: WheelDeltaMode,
    #[doc = include_str!("wheel_event.doc/timestamp_micros.md")]
    pub timestamp_micros: u64,
    #[doc = include_str!("wheel_event.doc/trust.md")]
    pub trust: EventTrust,
}

impl WheelEvent {
    #[doc = include_str!("wheel_event.doc/new.md")]
    #[must_use]
    pub const fn new(position: Point, delta_y: f32) -> Self {
        Self {
            position,
            delta_y,
            delta_x: 0.0,
            mode: WheelDeltaMode::Pixels,
            timestamp_micros: 0,
            trust: EventTrust::Untrusted,
        }
    }

    #[doc = include_str!("wheel_event.doc/with_delta_x.md")]
    #[must_use]
    pub const fn with_delta_x(mut self, delta_x: f32) -> Self {
        self.delta_x = delta_x;
        self
    }

    #[doc = include_str!("wheel_event.doc/in_lines.md")]
    #[must_use]
    pub const fn in_lines(mut self) -> Self {
        self.mode = WheelDeltaMode::Lines;
        self
    }

    #[doc = include_str!("wheel_event.doc/with_timestamp.md")]
    #[must_use]
    pub const fn with_timestamp(mut self, timestamp_micros: u64) -> Self {
        self.timestamp_micros = timestamp_micros;
        self
    }

    #[doc = include_str!("wheel_event.doc/as_trusted.md")]
    #[must_use]
    pub const fn as_trusted(mut self) -> Self {
        self.trust = EventTrust::Trusted;
        self
    }

    #[doc = include_str!("wheel_event.doc/at.md")]
    #[must_use]
    pub const fn at(mut self, position: Point) -> Self {
        self.position = position;
        self
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

    #[doc = include_str!("wheel_event.doc/delta_x_2.md")]
    #[must_use]
    pub const fn delta_x(&self) -> f32 {
        self.delta_x
    }

    #[doc = include_str!("wheel_event.doc/mode_2.md")]
    #[must_use]
    pub const fn mode(&self) -> WheelDeltaMode {
        self.mode
    }

    #[doc = include_str!("wheel_event.doc/pixels_y.md")]
    #[must_use]
    pub fn pixels_y(&self, line_height: f32) -> f32 {
        self.mode.resolve(self.delta_y, line_height)
    }

    #[doc = include_str!("wheel_event.doc/pixels_x.md")]
    #[must_use]
    pub fn pixels_x(&self, line_height: f32) -> f32 {
        self.mode.resolve(self.delta_x, line_height)
    }

    #[doc = include_str!("wheel_event.doc/timestamp.md")]
    #[must_use]
    pub const fn timestamp(&self) -> u64 {
        self.timestamp_micros
    }

    #[doc = include_str!("wheel_event.doc/trust_2.md")]
    #[must_use]
    pub const fn trust(&self) -> EventTrust {
        self.trust
    }

    #[doc = include_str!("wheel_event.doc/is_trusted.md")]
    #[must_use]
    pub const fn is_trusted(&self) -> bool {
        self.trust.is_trusted()
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

    #[test]
    fn a_fresh_notch_is_untrusted_unstamped_flat_and_in_pixels() {
        // Fails closed on every axis it can, exactly as `PointerEvent::new` and
        // `KeyEvent::new` do: trust is conferred at the platform boundary, a
        // host that stamps no clock gets zero, and the unit is the one that
        // multiplies nothing.
        let wheel = WheelEvent::new(Point::new(1.0, 2.0), 10.0);
        assert!(!wheel.is_trusted());
        assert_eq!(wheel.trust(), EventTrust::Untrusted);
        assert_eq!(wheel.timestamp(), 0);
        assert_eq!(wheel.delta_x(), 0.0);
        assert_eq!(wheel.mode(), WheelDeltaMode::Pixels);
    }

    #[test]
    fn lines_resolve_against_a_line_height_and_pixels_pass_through() {
        // The distinction the winit runner used to destroy by multiplying every
        // line report by 48 before the runtime ever saw it.
        let notched = WheelEvent::new(Point::new(0.0, 0.0), -3.0).in_lines();
        assert_eq!(notched.mode(), WheelDeltaMode::Lines);
        assert_eq!(notched.pixels_y(48.0), -144.0);
        assert_eq!(notched.delta_y(), -3.0, "the platform's own number is kept");

        let trackpad = WheelEvent::new(Point::new(0.0, 0.0), -25.0);
        assert_eq!(trackpad.pixels_y(48.0), -25.0);
    }

    #[test]
    fn the_horizontal_delta_is_carried_even_though_nothing_scrolls_by_it() {
        // The runtime scrolls vertically only, and reporting `delta_x` anyway is
        // the point: the platform knows it, an application with a horizontal
        // list needs it, and dropping it at the door is how it used to be lost.
        let wheel = WheelEvent::new(Point::new(0.0, 0.0), 0.0)
            .with_delta_x(-12.0)
            .in_lines();
        assert_eq!(wheel.delta_x(), -12.0);
        assert_eq!(wheel.pixels_x(48.0), -576.0);
    }

    #[test]
    fn every_builder_leaves_the_other_fields_alone() {
        let base = WheelEvent::new(Point::new(5.0, 6.0), -7.0);
        let built = base
            .with_delta_x(3.0)
            .in_lines()
            .with_timestamp(999)
            .as_trusted()
            .at(Point::new(1.0, 1.0));
        assert_eq!(built.delta_y(), base.delta_y());
        assert_eq!(built.delta_x(), 3.0);
        assert_eq!(built.mode(), WheelDeltaMode::Lines);
        assert_eq!(built.timestamp(), 999);
        assert!(built.is_trusted());
        assert_eq!(built.position(), Point::new(1.0, 1.0));
    }
}
