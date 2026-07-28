//! The [`Fling`] — the coast a flicked surface keeps after the finger has gone
//! (spec §6.16 `Swipe`/`Pan` continuation).

/// The decaying motion a surface continues with once a flick releases it.
///
/// A finger that leaves the glass mid-movement has thrown the content, and a
/// surface that stopped dead at the release would feel nailed down: the whole
/// impression of touching the content directly comes from it carrying on and
/// slowing to a stop. This models that as friction — velocity decaying
/// exponentially toward zero — and answers one question per frame: how far has
/// the surface moved since the last one.
///
/// The velocity decays with a time constant `tau`: after `tau` seconds about 37%
/// of the launch speed remains, and total travel is `velocity × tau`, which is
/// the number to reason with when tuning. Both `tau` and the speed below which
/// the coast is over are the caller's to choose — they are feel, and feel
/// belongs to whoever knows the display and the content, not to this type.
///
/// Reads no clock: it is told each step's duration, like everything else here.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Fling {
    velocity: f32,
    tau: f32,
    rest_speed: f32,
}

impl Fling {
    /// Launches a fling at `velocity` (units per second, signed), decaying with
    /// time constant `tau` seconds, ending once the speed falls below
    /// `rest_speed`.
    #[must_use]
    pub fn new(velocity: f32, tau: f32, rest_speed: f32) -> Self {
        Self {
            velocity,
            tau: tau.max(f32::EPSILON),
            rest_speed: rest_speed.max(0.0),
        }
    }

    /// Advances the coast by `seconds` and returns how far the surface travelled
    /// during that step (signed, in the velocity's units).
    ///
    /// The distance is the integral of the decaying velocity over the step —
    /// `tau × (v_before − v_after)` — not `velocity × seconds`. Multiplying the
    /// start-of-step velocity by the step would overshoot on every frame, and by
    /// more the longer the frame took, so a stuttering frame would fling further
    /// than a smooth one.
    pub fn advance(&mut self, seconds: f32) -> f32 {
        if self.is_done() || seconds <= 0.0 {
            return 0.0;
        }
        let decayed = self.velocity * (-seconds / self.tau).exp();
        let travelled = (self.velocity - decayed) * self.tau;
        self.velocity = decayed;
        if self.is_done() {
            self.velocity = 0.0;
        }
        travelled
    }

    /// Whether the coast has come to rest and should be dropped.
    #[must_use]
    pub fn is_done(&self) -> bool {
        self.velocity.abs() <= self.rest_speed
    }

    /// The current velocity (units per second, signed).
    #[must_use]
    pub fn velocity(&self) -> f32 {
        self.velocity
    }

    /// Stops the coast immediately — a new contact on the surface, or the surface
    /// reaching the end of its content.
    pub fn stop(&mut self) {
        self.velocity = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::Fling;

    const TAU: f32 = 0.325;
    const REST: f32 = 20.0;

    fn fling(velocity: f32) -> Fling {
        Fling::new(velocity, TAU, REST)
    }

    #[test]
    fn a_fling_travels_in_the_direction_it_was_thrown() {
        assert!(fling(1_000.0).advance(0.016) > 0.0);
        assert!(fling(-1_000.0).advance(0.016) < 0.0);
    }

    #[test]
    fn the_coast_slows_down() {
        let mut f = fling(1_000.0);
        let first = f.advance(0.016);
        let second = f.advance(0.016);
        assert!(second < first, "{second} should be less than {first}");
        assert!(f.velocity() < 1_000.0);
    }

    #[test]
    fn it_comes_to_rest_rather_than_coasting_forever() {
        let mut f = fling(1_000.0);
        let mut steps = 0;
        while !f.is_done() && steps < 10_000 {
            f.advance(0.016);
            steps += 1;
        }
        assert!(f.is_done(), "the coast ended");
        assert!(steps < 1_000, "and in a sane number of frames ({steps})");
        assert_eq!(f.advance(0.016), 0.0, "a finished fling moves nothing");
    }

    /// The property that makes the coast frame-rate independent: total distance
    /// depends on the time that passed, not on how it was chopped up.
    ///
    /// This is the whole reason the step integrates the decay instead of
    /// multiplying the current velocity by the frame length — under the naive
    /// form, one long frame flings much further than the many short frames
    /// covering the same span, so a stuttering list would end up somewhere a
    /// smooth one would not.
    #[test]
    fn the_distance_does_not_depend_on_the_frame_rate() {
        let mut coarse = fling(1_000.0);
        let far = coarse.advance(0.1);

        let mut fine = fling(1_000.0);
        let mut near = 0.0;
        for _ in 0..10 {
            near += fine.advance(0.01);
        }

        assert!(
            (far - near).abs() < 0.5,
            "one 100ms step travelled {far}, ten 10ms steps travelled {near}"
        );
        assert!((coarse.velocity() - fine.velocity()).abs() < 1.0);
    }

    #[test]
    fn total_travel_is_velocity_times_tau() {
        // The number to tune with: a 1000 px/s flick with tau 0.325 coasts about
        // 325px. Worth pinning, because it is the only intuitive handle on a
        // decay constant.
        let mut f = Fling::new(1_000.0, TAU, 0.0);
        let mut total = 0.0;
        for _ in 0..1_000 {
            total += f.advance(0.01);
        }
        assert!((total - 325.0).abs() < 1.0, "coasted {total}");
    }

    #[test]
    fn a_stopped_fling_is_done_and_moves_nothing() {
        let mut f = fling(1_000.0);
        f.stop();
        assert!(f.is_done());
        assert_eq!(f.advance(0.016), 0.0);
    }

    #[test]
    fn a_flick_below_the_rest_speed_never_starts() {
        let mut f = fling(10.0);
        assert!(f.is_done());
        assert_eq!(f.advance(0.016), 0.0);
    }
}
