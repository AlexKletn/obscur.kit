//! The [`TimeDriver`] time-driven motion source (spec §4.17).

use crate::driver::MotionDriver;

#[doc = include_str!("../../docs/api/driver/time_driver.doc/TimeDriver.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimeDriver {
    /// Accumulated logical seconds since the last [`reset`](Self::reset)
    /// (or construction). Never decreases through normal advancing.
    elapsed: f32,
    /// Logical seconds the driver spans. Always strictly positive: a zero,
    /// negative or `NaN` `duration` passed to [`new`](Self::new) is clamped up
    /// to [`MIN_DURATION`](Self::MIN_DURATION) to keep the progress division
    /// well-defined.
    duration: f32,
}

impl TimeDriver {
    #[doc = include_str!("../../docs/api/driver/time_driver.doc/MIN_DURATION.md")]
    pub const MIN_DURATION: f32 = f32::EPSILON;

    #[doc = include_str!("../../docs/api/driver/time_driver.doc/new.md")]
    #[must_use]
    pub fn new(duration: f32) -> Self {
        Self {
            elapsed: 0.0,
            duration: duration.max(Self::MIN_DURATION),
        }
    }

    #[doc = include_str!("../../docs/api/driver/time_driver.doc/advance.md")]
    pub fn advance(&mut self, delta: f32) {
        if delta > 0.0 {
            self.elapsed += delta;
        }
    }

    /// Resets accumulated time back to `0` (progress `0`), leaving `duration`
    /// unchanged so the driver can be replayed.
    pub fn reset(&mut self) {
        self.elapsed = 0.0;
    }

    /// Returns the accumulated logical seconds.
    #[must_use]
    pub const fn elapsed(&self) -> f32 {
        self.elapsed
    }

    /// Returns the span in logical seconds (always strictly positive).
    #[must_use]
    pub const fn duration(&self) -> f32 {
        self.duration
    }
}

impl MotionDriver for TimeDriver {
    /// Returns `(elapsed / duration).clamp(0.0, 1.0)` (spec §4.17).
    fn progress(&self) -> f32 {
        (self.elapsed / self.duration).clamp(0.0, 1.0)
    }

    /// Returns `elapsed < duration`: the driver is active until accumulated time
    /// reaches its span, after which it has settled.
    fn is_active(&self) -> bool {
        self.elapsed < self.duration
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_starts_at_zero_progress_and_active() {
        let d = TimeDriver::new(2.0);
        assert_eq!(d.elapsed(), 0.0);
        assert_eq!(d.duration(), 2.0);
        assert_eq!(d.progress(), 0.0);
        assert!(d.is_active());
    }

    #[test]
    fn non_positive_duration_is_clamped() {
        assert!(TimeDriver::new(0.0).duration() > 0.0);
        assert!(TimeDriver::new(-3.0).duration() > 0.0);
        assert!(TimeDriver::new(f32::NAN).duration() > 0.0);
    }

    #[test]
    fn advance_ramps_progress_zero_to_one() {
        let mut d = TimeDriver::new(1.0);
        d.advance(0.25);
        assert!((d.progress() - 0.25).abs() < 1e-6);
        d.advance(0.25);
        assert!((d.progress() - 0.5).abs() < 1e-6);
        d.advance(0.5);
        assert!((d.progress() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn over_advance_clamps_and_settles() {
        let mut d = TimeDriver::new(1.0);
        // A single large delta (e.g. a skipped frame) overshoots the duration.
        d.advance(10.0);
        assert_eq!(d.progress(), 1.0);
        assert!(!d.is_active());
    }

    #[test]
    fn negative_delta_is_ignored() {
        let mut d = TimeDriver::new(1.0);
        d.advance(0.4);
        d.advance(-0.9);
        assert!((d.progress() - 0.4).abs() < 1e-6);
        assert!((d.elapsed() - 0.4).abs() < 1e-6);
    }

    #[test]
    fn reset_returns_to_start() {
        let mut d = TimeDriver::new(1.0);
        d.advance(0.7);
        d.reset();
        assert_eq!(d.elapsed(), 0.0);
        assert_eq!(d.progress(), 0.0);
        assert!(d.is_active());
    }

    #[test]
    fn same_total_delta_is_deterministic_across_frame_splits() {
        // Many small deltas or one big delta reach the same progress.
        let mut split = TimeDriver::new(1.0);
        for _ in 0..8 {
            split.advance(0.1);
        }
        let mut whole = TimeDriver::new(1.0);
        whole.advance(0.8);
        assert!((split.progress() - whole.progress()).abs() < 1e-6);
    }

    #[test]
    fn object_safe_via_box_and_ref() {
        let boxed: Box<dyn MotionDriver> = Box::new(TimeDriver::new(1.0));
        assert_eq!(boxed.progress(), 0.0);
        let driver = TimeDriver::new(1.0);
        let as_dyn: &dyn MotionDriver = &driver;
        assert!(as_dyn.is_active());
    }
}
