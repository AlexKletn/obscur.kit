#![doc = include_str!("signal_driver.doc/_module.md")]

use crate::driver::MotionDriver;

#[doc = include_str!("signal_driver.doc/SignalDriver.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SignalDriver {
    #[doc = include_str!("signal_driver.doc/value.md")]
    value: f32,
    #[doc = include_str!("signal_driver.doc/min.md")]
    min: f32,
    #[doc = include_str!("signal_driver.doc/max.md")]
    max: f32,
}

impl SignalDriver {
    #[doc = include_str!("signal_driver.doc/new.md")]
    #[must_use]
    pub const fn new(min: f32, max: f32) -> Self {
        Self {
            value: min,
            min,
            max,
        }
    }

    #[doc = include_str!("signal_driver.doc/with_value.md")]
    #[must_use]
    pub const fn with_value(mut self, value: f32) -> Self {
        self.value = value;
        self
    }

    #[doc = include_str!("signal_driver.doc/set_value.md")]
    pub fn set_value(&mut self, value: f32) {
        self.value = value;
    }

    #[doc = include_str!("signal_driver.doc/value_2.md")]
    #[must_use]
    pub const fn value(&self) -> f32 {
        self.value
    }

    #[doc = include_str!("signal_driver.doc/min_2.md")]
    #[must_use]
    pub const fn min(&self) -> f32 {
        self.min
    }

    #[doc = include_str!("signal_driver.doc/max_2.md")]
    #[must_use]
    pub const fn max(&self) -> f32 {
        self.max
    }
}

impl MotionDriver for SignalDriver {
    #[doc = include_str!("signal_driver.doc/progress.md")]
    fn progress(&self) -> f32 {
        let span = self.max - self.min;
        if span == 0.0 {
            return 0.0;
        }
        ((self.value - self.min) / span).clamp(0.0, 1.0)
    }

    #[doc = include_str!("signal_driver.doc/is_active.md")]
    fn is_active(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_starts_at_min_zero_progress() {
        let d = SignalDriver::new(0.0, 10.0);
        assert_eq!(d.value(), 0.0);
        assert_eq!(d.min(), 0.0);
        assert_eq!(d.max(), 10.0);
        assert_eq!(d.progress(), 0.0);
    }

    #[test]
    fn value_at_min_max_and_mid() {
        let base = SignalDriver::new(0.0, 10.0);
        assert_eq!(base.with_value(0.0).progress(), 0.0);
        assert_eq!(base.with_value(10.0).progress(), 1.0);
        assert!((base.with_value(5.0).progress() - 0.5).abs() < 1e-6);
    }

    #[test]
    fn out_of_range_values_clamp() {
        let base = SignalDriver::new(0.0, 10.0);
        assert_eq!(base.with_value(-5.0).progress(), 0.0);
        assert_eq!(base.with_value(20.0).progress(), 1.0);
    }

    #[test]
    fn set_value_updates_progress() {
        let mut d = SignalDriver::new(0.0, 4.0);
        d.set_value(1.0);
        assert!((d.progress() - 0.25).abs() < 1e-6);
        d.set_value(3.0);
        assert!((d.progress() - 0.75).abs() < 1e-6);
    }

    #[test]
    fn equal_domain_yields_zero() {
        let d = SignalDriver::new(5.0, 5.0).with_value(5.0);
        assert_eq!(d.progress(), 0.0);
    }

    #[test]
    fn inverted_domain_normalizes_correctly() {
        // min > max: value at min -> 0, at max -> 1, midpoint -> 0.5.
        let base = SignalDriver::new(10.0, 0.0);
        assert_eq!(base.with_value(10.0).progress(), 0.0);
        assert_eq!(base.with_value(0.0).progress(), 1.0);
        assert!((base.with_value(5.0).progress() - 0.5).abs() < 1e-6);
    }

    #[test]
    fn always_active() {
        assert!(SignalDriver::new(0.0, 1.0).is_active());
    }

    #[test]
    fn object_safe_via_box() {
        let boxed: Box<dyn MotionDriver> = Box::new(SignalDriver::new(0.0, 2.0).with_value(1.0));
        assert!((boxed.progress() - 0.5).abs() < 1e-6);
    }
}
