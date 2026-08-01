#![doc = include_str!("time_driver.doc/_module.md")]

use crate::driver::MotionDriver;

#[doc = include_str!("time_driver.doc/TimeDriver.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimeDriver {
    #[doc = include_str!("time_driver.doc/elapsed.md")]
    elapsed: f32,
    #[doc = include_str!("time_driver.doc/duration.md")]
    duration: f32,
}

impl TimeDriver {
    #[doc = include_str!("time_driver.doc/MIN_DURATION.md")]
    pub const MIN_DURATION: f32 = f32::EPSILON;

    #[doc = include_str!("time_driver.doc/new.md")]
    #[must_use]
    pub fn new(duration: f32) -> Self {
        Self {
            elapsed: 0.0,
            duration: duration.max(Self::MIN_DURATION),
        }
    }

    #[doc = include_str!("time_driver.doc/advance.md")]
    pub fn advance(&mut self, delta: f32) {
        if delta > 0.0 {
            self.elapsed += delta;
        }
    }

    #[doc = include_str!("time_driver.doc/reset.md")]
    pub fn reset(&mut self) {
        self.elapsed = 0.0;
    }

    #[doc = include_str!("time_driver.doc/elapsed_2.md")]
    #[must_use]
    pub const fn elapsed(&self) -> f32 {
        self.elapsed
    }

    #[doc = include_str!("time_driver.doc/duration_2.md")]
    #[must_use]
    pub const fn duration(&self) -> f32 {
        self.duration
    }
}

impl MotionDriver for TimeDriver {
    #[doc = include_str!("time_driver.doc/progress.md")]
    fn progress(&self) -> f32 {
        (self.elapsed / self.duration).clamp(0.0, 1.0)
    }

    #[doc = include_str!("time_driver.doc/is_active.md")]
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
