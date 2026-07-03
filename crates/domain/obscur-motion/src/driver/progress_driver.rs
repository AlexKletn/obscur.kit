//! The [`ProgressDriver`] progress-driven motion source (spec §4.17).

use crate::driver::MotionDriver;

/// A progress-driven [`MotionDriver`]: progress set explicitly from the outside
/// (spec §4.17 "progress-driven motion").
///
/// Unlike a [`TimeDriver`](super::TimeDriver), this driver integrates nothing —
/// it simply holds a normalized progress that some external control writes
/// directly, such as a scrubber, slider or timeline-scrubbing gesture. The
/// stored value is always kept within `[0, 1]`, so [`progress`](MotionDriver::progress)
/// returns it verbatim.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProgressDriver {
    /// The externally-set normalized progress, always kept within `[0, 1]` by
    /// the [`at`](Self::at) and [`set`](Self::set) entry points.
    progress: f32,
}

impl ProgressDriver {
    /// Creates a driver at progress `0.0`.
    #[must_use]
    pub const fn new() -> Self {
        Self { progress: 0.0 }
    }

    /// Creates a driver at the given `progress`, clamped to `[0, 1]`.
    #[must_use]
    pub fn at(progress: f32) -> Self {
        Self {
            progress: progress.clamp(0.0, 1.0),
        }
    }

    /// Sets the current progress, clamped to `[0, 1]` (spec §4.17).
    ///
    /// This is the external control's write path: a scrubber reports its new
    /// position here and the driver reflects it on the next
    /// [`progress`](MotionDriver::progress) read.
    pub fn set(&mut self, progress: f32) {
        self.progress = progress.clamp(0.0, 1.0);
    }
}

impl Default for ProgressDriver {
    /// The default driver sits at progress `0.0`.
    fn default() -> Self {
        Self::new()
    }
}

impl MotionDriver for ProgressDriver {
    /// Returns the stored normalized progress (always within `[0, 1]`).
    fn progress(&self) -> f32 {
        self.progress
    }

    /// Always `true`: an external scrubber is considered permanently live, since
    /// it can move to any progress at any time.
    fn is_active(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_and_default_start_at_zero() {
        assert_eq!(ProgressDriver::new().progress(), 0.0);
        assert_eq!(ProgressDriver::default().progress(), 0.0);
    }

    #[test]
    fn at_stores_and_clamps() {
        assert!((ProgressDriver::at(0.4).progress() - 0.4).abs() < 1e-6);
        assert_eq!(ProgressDriver::at(2.0).progress(), 1.0);
        assert_eq!(ProgressDriver::at(-1.0).progress(), 0.0);
    }

    #[test]
    fn set_updates_and_clamps() {
        let mut d = ProgressDriver::new();
        d.set(0.75);
        assert!((d.progress() - 0.75).abs() < 1e-6);
        d.set(5.0);
        assert_eq!(d.progress(), 1.0);
        d.set(-5.0);
        assert_eq!(d.progress(), 0.0);
    }

    #[test]
    fn always_active() {
        assert!(ProgressDriver::at(0.0).is_active());
        assert!(ProgressDriver::at(1.0).is_active());
    }

    #[test]
    fn object_safe_via_box() {
        let boxed: Box<dyn MotionDriver> = Box::new(ProgressDriver::at(0.5));
        assert!((boxed.progress() - 0.5).abs() < 1e-6);
    }
}
