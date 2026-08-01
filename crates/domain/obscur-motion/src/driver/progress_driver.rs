#![doc = include_str!("progress_driver.doc/_module.md")]

use crate::driver::MotionDriver;

#[doc = include_str!("progress_driver.doc/ProgressDriver.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProgressDriver {
    #[doc = include_str!("progress_driver.doc/progress.md")]
    progress: f32,
}

impl ProgressDriver {
    #[doc = include_str!("progress_driver.doc/new.md")]
    #[must_use]
    pub const fn new() -> Self {
        Self { progress: 0.0 }
    }

    #[doc = include_str!("progress_driver.doc/at.md")]
    #[must_use]
    pub fn at(progress: f32) -> Self {
        Self {
            progress: progress.clamp(0.0, 1.0),
        }
    }

    #[doc = include_str!("progress_driver.doc/set.md")]
    pub fn set(&mut self, progress: f32) {
        self.progress = progress.clamp(0.0, 1.0);
    }
}

impl Default for ProgressDriver {
    #[doc = include_str!("progress_driver.doc/default.md")]
    fn default() -> Self {
        Self::new()
    }
}

impl MotionDriver for ProgressDriver {
    #[doc = include_str!("progress_driver.doc/progress_2.md")]
    fn progress(&self) -> f32 {
        self.progress
    }

    #[doc = include_str!("progress_driver.doc/is_active.md")]
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
