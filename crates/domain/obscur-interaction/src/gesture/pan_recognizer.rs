#![doc = include_str!("pan_recognizer.doc/_module.md")]

use obscur_geometry::Point;

use super::{GestureKind, GestureRecognizer, RecognizerId, RecognizerState};

#[doc = include_str!("pan_recognizer.doc/PanRecognizer.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PanRecognizer {
    recognizer: GestureRecognizer,
    origin: Point,
    origin_micros: u64,
    last: Point,
    last_micros: u64,
    slop: f32,
    velocity: (f32, f32),
    sampled: bool,
}

impl PanRecognizer {
    #[doc = include_str!("pan_recognizer.doc/new.md")]
    #[must_use]
    pub fn new(id: RecognizerId, origin: Point, micros: u64, slop: f32) -> Self {
        Self {
            recognizer: GestureRecognizer::new(id, GestureKind::Pan),
            origin,
            origin_micros: micros,
            last: origin,
            last_micros: micros,
            slop,
            velocity: (0.0, 0.0),
            sampled: false,
        }
    }

    #[doc = include_str!("pan_recognizer.doc/update.md")]
    pub fn update(&mut self, at: Point, micros: u64) -> Option<(f32, f32)> {
        if self.recognizer.state().is_terminal() {
            return None;
        }
        self.sample_velocity(at, micros);
        let from = if self.recognizer.state() == RecognizerState::Began {
            self.last
        } else if self.travel_from_origin(at) > self.slop {
            self.recognizer.begin();
            self.origin
        } else {
            // Still inside the slop: remember where the contact is, but report
            // nothing — this is not (yet) a pan.
            self.last = at;
            self.last_micros = micros;
            return None;
        };
        self.last = at;
        self.last_micros = micros;
        Some((at.x - from.x, at.y - from.y))
    }

    #[doc = include_str!("pan_recognizer.doc/end.md")]
    pub fn end(&mut self) {
        if self.recognizer.state() == RecognizerState::Began {
            self.recognizer.recognize();
        } else {
            self.recognizer.fail();
        }
    }

    #[doc = include_str!("pan_recognizer.doc/cancel.md")]
    pub fn cancel(&mut self) {
        self.recognizer.cancel();
    }

    #[doc = include_str!("pan_recognizer.doc/has_begun.md")]
    #[must_use]
    pub fn has_begun(&self) -> bool {
        self.recognizer.state() == RecognizerState::Began
    }

    #[doc = include_str!("pan_recognizer.doc/state.md")]
    #[must_use]
    pub fn state(&self) -> RecognizerState {
        self.recognizer.state()
    }

    #[doc = include_str!("pan_recognizer.doc/id.md")]
    #[must_use]
    pub fn id(&self) -> RecognizerId {
        self.recognizer.id()
    }

    #[doc = include_str!("pan_recognizer.doc/origin.md")]
    #[must_use]
    pub fn origin(&self) -> Point {
        self.origin
    }

    #[doc = include_str!("pan_recognizer.doc/origin_micros.md")]
    #[must_use]
    pub fn origin_micros(&self) -> u64 {
        self.origin_micros
    }

    #[doc = include_str!("pan_recognizer.doc/velocity_at.md")]
    #[must_use]
    pub fn velocity_at(&self, micros: u64, rest_micros: u64) -> (f32, f32) {
        if micros.saturating_sub(self.last_micros) > rest_micros {
            return (0.0, 0.0);
        }
        self.velocity
    }

    #[doc = include_str!("pan_recognizer.doc/sample_velocity.md")]
    fn sample_velocity(&mut self, at: Point, micros: u64) {
        let dt = micros.saturating_sub(self.last_micros) as f32 / 1_000_000.0;
        if dt <= 0.0 {
            return;
        }
        let instant = ((at.x - self.last.x) / dt, (at.y - self.last.y) / dt);
        self.velocity = if self.sampled {
            (
                self.velocity
                    .0
                    .mul_add(SMOOTHING, instant.0 * (1.0 - SMOOTHING)),
                self.velocity
                    .1
                    .mul_add(SMOOTHING, instant.1 * (1.0 - SMOOTHING)),
            )
        } else {
            instant
        };
        self.sampled = true;
    }

    #[doc = include_str!("pan_recognizer.doc/travel_from_origin.md")]
    fn travel_from_origin(&self, at: Point) -> f32 {
        (at.x - self.origin.x).hypot(at.y - self.origin.y)
    }
}

#[doc = include_str!("pan_recognizer.doc/SMOOTHING.md")]
const SMOOTHING: f32 = 0.4;

#[cfg(test)]
mod tests {
    use super::*;

    const SLOP: f32 = 10.0;

    fn recognizer() -> PanRecognizer {
        PanRecognizer::new(RecognizerId::from_raw(1), Point::new(100.0, 100.0), 0, SLOP)
    }

    #[test]
    fn a_contact_inside_the_slop_is_not_a_pan() {
        // The jitter every real finger produces while pressing and lifting.
        let mut pan = recognizer();
        assert_eq!(pan.update(Point::new(101.0, 102.0), 1_000), None);
        assert_eq!(pan.update(Point::new(100.0, 98.0), 2_000), None);
        assert!(!pan.has_begun());
        assert_eq!(pan.state(), RecognizerState::Possible);
    }

    #[test]
    fn crossing_the_slop_begins_the_pan_and_reports_the_whole_travel() {
        // The crossing sample reports the delta from the ORIGIN. Reporting it
        // from the previous sample would silently eat up to one slop radius of
        // the finger's movement, and the content would sit that far behind the
        // finger for the rest of the gesture.
        let mut pan = recognizer();
        assert_eq!(pan.update(Point::new(100.0, 105.0), 1_000), None);
        let delta = pan
            .update(Point::new(100.0, 130.0), 2_000)
            .expect("the pan began");
        assert!((delta.1 - 30.0).abs() < f32::EPSILON, "{delta:?}");
        assert!(pan.has_begun());
        assert_eq!(pan.state(), RecognizerState::Began);
    }

    #[test]
    fn a_begun_pan_reports_each_samples_own_delta() {
        let mut pan = recognizer();
        pan.update(Point::new(100.0, 130.0), 1_000).expect("began");
        let delta = pan.update(Point::new(100.0, 140.0), 2_000).expect("moving");
        assert!((delta.1 - 10.0).abs() < f32::EPSILON, "{delta:?}");
        let back = pan.update(Point::new(100.0, 135.0), 3_000).expect("moving");
        assert!((back.1 + 5.0).abs() < f32::EPSILON, "{back:?}");
    }

    #[test]
    fn the_slop_is_a_radius_not_an_axis_threshold() {
        // Diagonal travel counts: √(8² + 8²) ≈ 11.3 > 10, though neither axis
        // alone crosses. A per-axis test would refuse to start this drag.
        let mut pan = recognizer();
        assert!(pan.update(Point::new(108.0, 108.0), 1_000).is_some());
    }

    #[test]
    fn a_contact_that_lifts_without_travelling_fails_rather_than_recognizing() {
        // This is how the consumer learns it was a tap: the pan says it was not.
        let mut pan = recognizer();
        pan.update(Point::new(102.0, 102.0), 1_000);
        pan.end();
        assert_eq!(pan.state(), RecognizerState::Failed);
    }

    #[test]
    fn a_lifted_pan_is_recognized() {
        let mut pan = recognizer();
        pan.update(Point::new(100.0, 130.0), 1_000);
        pan.end();
        assert_eq!(pan.state(), RecognizerState::Recognized);
    }

    #[test]
    fn a_terminal_recognizer_ignores_further_samples() {
        let mut pan = recognizer();
        pan.update(Point::new(100.0, 130.0), 1_000);
        pan.cancel();
        assert_eq!(pan.state(), RecognizerState::Cancelled);
        assert_eq!(pan.update(Point::new(100.0, 200.0), 2_000), None);
        assert!(!pan.has_begun());
    }

    /// A quarter-second in µs — long enough that a contact counts as rested.
    const REST: u64 = 250_000;

    #[test]
    fn a_moving_contact_reports_the_speed_it_is_moving_at() {
        let mut pan = recognizer();
        // 30px in 30ms, twice: 1000 px/s, and the smoothing has converged.
        pan.update(Point::new(100.0, 130.0), 30_000);
        pan.update(Point::new(100.0, 160.0), 60_000);
        let (_, vy) = pan.velocity_at(60_000, REST);
        assert!((vy - 1_000.0).abs() < 1.0, "got {vy} px/s");
    }

    #[test]
    fn velocity_is_signed_by_direction() {
        let mut pan = recognizer();
        pan.update(Point::new(100.0, 70.0), 30_000);
        pan.update(Point::new(100.0, 40.0), 60_000);
        assert!(pan.velocity_at(60_000, REST).1 < 0.0);
    }

    /// A finger that stops before lifting has thrown nothing.
    ///
    /// Nothing is delivered while a finger rests, so the last sample still holds
    /// the speed from before the pause. Reporting it would fling a list the user
    /// had deliberately brought to a stop — which is exactly what it feels like
    /// when a toolkit gets this wrong.
    #[test]
    fn a_contact_that_came_to_rest_before_lifting_has_no_velocity() {
        let mut pan = recognizer();
        pan.update(Point::new(100.0, 130.0), 30_000);
        pan.update(Point::new(100.0, 160.0), 60_000);
        assert!(
            pan.velocity_at(60_000, REST).1 > 0.0,
            "moving at the sample"
        );

        // Lifted half a second after the last movement.
        let (_, vy) = pan.velocity_at(560_000, REST);
        assert!(
            (vy - 0.0).abs() < f32::EPSILON,
            "got {vy} px/s after resting"
        );
    }

    #[test]
    fn a_contact_that_never_moved_has_no_velocity() {
        let pan = recognizer();
        assert_eq!(pan.velocity_at(0, REST), (0.0, 0.0));
    }

    #[test]
    fn the_origin_is_remembered_for_the_whole_sequence() {
        let mut pan = recognizer();
        pan.update(Point::new(100.0, 130.0), 1_000);
        pan.update(Point::new(100.0, 400.0), 2_000);
        assert_eq!(pan.origin(), Point::new(100.0, 100.0));
        assert_eq!(pan.origin_micros(), 0);
        assert_eq!(pan.id(), RecognizerId::from_raw(1));
    }
}
