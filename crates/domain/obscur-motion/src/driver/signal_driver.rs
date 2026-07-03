//! The [`SignalDriver`] value-driven motion source (spec §4.17).

use crate::driver::MotionDriver;

/// A value-driven [`MotionDriver`]: a continuous external signal mapped onto
/// `[0, 1]` (spec §4.17 "value-driven motion; drag/cursor/gesture signals").
///
/// The driver maps a raw signal `value` — a drag distance, cursor coordinate or
/// gesture magnitude — from an input domain `[min, max]` onto normalized
/// progress via `(value - min) / (max - min)`, clamped to `[0, 1]`. The domain
/// endpoints do not have to be ordered: an **inverted** domain (`min > max`)
/// normalizes correctly, mapping `value == min` to progress `0` and
/// `value == max` to progress `1`, so a signal that runs high-to-low still
/// drives forward. A degenerate domain (`min == max`) has no width to normalize
/// against and yields progress `0`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SignalDriver {
    /// The current raw signal value.
    value: f32,
    /// The input-domain endpoint mapped to progress `0`.
    min: f32,
    /// The input-domain endpoint mapped to progress `1`.
    max: f32,
}

impl SignalDriver {
    /// Creates a driver over the input domain `[min, max]`, starting with its
    /// `value` at `min` (progress `0`).
    ///
    /// The endpoints may be inverted (`min > max`); see the type docs. If they
    /// are equal the domain is degenerate and progress is always `0`.
    #[must_use]
    pub const fn new(min: f32, max: f32) -> Self {
        Self {
            value: min,
            min,
            max,
        }
    }

    /// Returns this driver with its current signal set to `value`.
    #[must_use]
    pub const fn with_value(mut self, value: f32) -> Self {
        self.value = value;
        self
    }

    /// Sets the current raw signal value (spec §4.17).
    ///
    /// This is the interaction subsystem's write path: a drag/cursor/gesture
    /// reports its latest reading here and the driver re-normalizes it on the
    /// next [`progress`](MotionDriver::progress) read.
    pub fn set_value(&mut self, value: f32) {
        self.value = value;
    }

    /// Returns the current raw signal value.
    #[must_use]
    pub const fn value(&self) -> f32 {
        self.value
    }

    /// Returns the input-domain endpoint mapped to progress `0`.
    #[must_use]
    pub const fn min(&self) -> f32 {
        self.min
    }

    /// Returns the input-domain endpoint mapped to progress `1`.
    #[must_use]
    pub const fn max(&self) -> f32 {
        self.max
    }
}

impl MotionDriver for SignalDriver {
    /// Returns `((value - min) / (max - min)).clamp(0.0, 1.0)` (spec §4.17).
    ///
    /// A degenerate domain (`min == max`) has no width and yields `0.0`; an
    /// inverted domain normalizes correctly (see the type docs).
    fn progress(&self) -> f32 {
        let span = self.max - self.min;
        if span == 0.0 {
            return 0.0;
        }
        ((self.value - self.min) / span).clamp(0.0, 1.0)
    }

    /// Always `true`: a live drag/cursor/gesture signal can move at any time.
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
