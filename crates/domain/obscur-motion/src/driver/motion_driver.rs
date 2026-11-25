#![doc = include_str!("../../docs/api/driver/motion_driver.doc/_module.md")]

#[doc = include_str!("../../docs/api/driver/motion_driver.doc/MotionDriver.md")]
pub trait MotionDriver {
    #[doc = include_str!("../../docs/api/driver/motion_driver.doc/progress.md")]
    #[must_use]
    fn progress(&self) -> f32;

    #[doc = include_str!("../../docs/api/driver/motion_driver.doc/is_active.md")]
    #[must_use]
    fn is_active(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::driver::{ProgressDriver, ScrollDriver, SignalDriver, TimeDriver};

    #[test]
    fn trait_is_object_safe_via_box() {
        // A heterogeneous set of drivers can be held behind the object-safe port.
        let drivers: [Box<dyn MotionDriver>; 4] = [
            Box::new(TimeDriver::new(1.0)),
            Box::new(ProgressDriver::at(0.5)),
            Box::new(SignalDriver::new(0.0, 10.0)),
            Box::new(ScrollDriver::new(0.0, 100.0)),
        ];
        for driver in &drivers {
            assert!((0.0..=1.0).contains(&driver.progress()));
        }
    }

    #[test]
    fn borrowed_as_dyn_reports_progress() {
        let driver = ProgressDriver::at(0.25);
        let as_dyn: &dyn MotionDriver = &driver;
        assert!((as_dyn.progress() - 0.25).abs() < 1e-6);
        assert!(as_dyn.is_active());
    }

    #[test]
    fn default_is_active_is_true() {
        // ProgressDriver relies on the default `is_active` body.
        let as_dyn: &dyn MotionDriver = &ProgressDriver::new();
        assert!(as_dyn.is_active());
    }
}
