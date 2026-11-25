#![doc = include_str!("../../docs/api/pointer/wheel_event.doc/_module.md")]

use obscur_geometry::Point;

use super::{WheelDeltaMode, WheelPhase};
use crate::event::EventTrust;

#[doc = include_str!("../../docs/api/pointer/wheel_event.doc/WheelEvent.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WheelEvent {
    /// Where the pointer was, in the same space a dispatched pointer event's position
    /// is in — content space, folded through the scroll containers the point lies
    /// within. That is the space the hit test that found the target ran in, so a handler
    /// comparing this against a node's box is comparing like with like.
    pub position: Point,
    #[doc = include_str!("../../docs/api/pointer/wheel_event.doc/delta_y.md")]
    pub delta_y: f32,
    #[doc = include_str!("../../docs/api/pointer/wheel_event.doc/delta_x.md")]
    pub delta_x: f32,
    /// What unit the deltas are in — a notched wheel reports lines, a trackpad pixels.
    pub mode: WheelDeltaMode,
    /// Microseconds on whatever monotonic origin the host chose, matching `PointerEvent` and `KeyEvent` exactly. Zero means unstamped.
    pub timestamp_micros: u64,
    #[doc = include_str!("../../docs/api/pointer/wheel_event.doc/trust.md")]
    pub trust: EventTrust,
    /// Contact phase reported by the window backend.
    pub phase: WheelPhase,
}

impl WheelEvent {
    /// A notch of `delta_y` logical px aimed at `position`.
    #[must_use]
    pub const fn new(position: Point, delta_y: f32) -> Self {
        Self {
            position,
            delta_y,
            delta_x: 0.0,
            mode: WheelDeltaMode::Pixels,
            timestamp_micros: 0,
            trust: EventTrust::Untrusted,
            phase: WheelPhase::Unspecified,
        }
    }

    /// Records the contact phase supplied by the window backend.
    #[must_use]
    pub const fn with_phase(mut self, phase: WheelPhase) -> Self {
        self.phase = phase;
        self
    }

    /// Returns the event with a horizontal delta.
    #[must_use]
    pub const fn with_delta_x(mut self, delta_x: f32) -> Self {
        self.delta_x = delta_x;
        self
    }

    /// Returns the event with its deltas read as LINES rather than pixels.
    #[must_use]
    pub const fn in_lines(mut self) -> Self {
        self.mode = WheelDeltaMode::Lines;
        self
    }

    /// Returns the event stamped with a host timestamp, in microseconds.
    #[must_use]
    pub const fn with_timestamp(mut self, timestamp_micros: u64) -> Self {
        self.timestamp_micros = timestamp_micros;
        self
    }

    /// Returns the event marked trusted. The only way to confer trust, and a platform adapter is what calls it.
    #[must_use]
    pub const fn as_trusted(mut self) -> Self {
        self.trust = EventTrust::Trusted;
        self
    }

    #[doc = include_str!("../../docs/api/pointer/wheel_event.doc/at.md")]
    #[must_use]
    pub const fn at(mut self, position: Point) -> Self {
        self.position = position;
        self
    }

    /// Where the pointer was.
    #[must_use]
    pub const fn position(&self) -> Point {
        self.position
    }

    /// Logical pixels to scroll by; positive is toward the content's end.
    #[must_use]
    pub const fn delta_y(&self) -> f32 {
        self.delta_y
    }

    /// The horizontal delta, in [`mode`](WheelEvent::mode)'s unit.
    #[must_use]
    pub const fn delta_x(&self) -> f32 {
        self.delta_x
    }

    /// What unit the deltas are in.
    #[must_use]
    pub const fn mode(&self) -> WheelDeltaMode {
        self.mode
    }

    /// The vertical delta in pixels, given what a line is worth.
    #[must_use]
    pub fn pixels_y(&self, line_height: f32) -> f32 {
        self.mode.resolve(self.delta_y, line_height)
    }

    /// The horizontal delta in pixels, given what a line is worth.
    #[must_use]
    pub fn pixels_x(&self, line_height: f32) -> f32 {
        self.mode.resolve(self.delta_x, line_height)
    }

    /// The host timestamp in microseconds, or zero if unstamped.
    #[must_use]
    pub const fn timestamp(&self) -> u64 {
        self.timestamp_micros
    }

    /// Whether this notch came from a real device or was synthesised.
    #[must_use]
    pub const fn trust(&self) -> EventTrust {
        self.trust
    }

    /// Whether this notch came from a real device.
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
