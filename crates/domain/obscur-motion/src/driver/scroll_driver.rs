#![doc = include_str!("scroll_driver.doc/_module.md")]

use crate::driver::MotionDriver;

#[doc = include_str!("scroll_driver.doc/ScrollDriver.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScrollDriver {
    #[doc = include_str!("scroll_driver.doc/offset.md")]
    offset: f32,
    #[doc = include_str!("scroll_driver.doc/start.md")]
    start: f32,
    #[doc = include_str!("scroll_driver.doc/end.md")]
    end: f32,
}

impl ScrollDriver {
    #[doc = include_str!("scroll_driver.doc/new.md")]
    #[must_use]
    pub const fn new(start: f32, end: f32) -> Self {
        Self {
            offset: start,
            start,
            end,
        }
    }

    #[doc = include_str!("scroll_driver.doc/with_offset.md")]
    #[must_use]
    pub const fn with_offset(mut self, offset: f32) -> Self {
        self.offset = offset;
        self
    }

    #[doc = include_str!("scroll_driver.doc/set_offset.md")]
    pub fn set_offset(&mut self, offset: f32) {
        self.offset = offset;
    }

    #[doc = include_str!("scroll_driver.doc/offset_2.md")]
    #[must_use]
    pub const fn offset(&self) -> f32 {
        self.offset
    }

    #[doc = include_str!("scroll_driver.doc/start_2.md")]
    #[must_use]
    pub const fn start(&self) -> f32 {
        self.start
    }

    #[doc = include_str!("scroll_driver.doc/end_2.md")]
    #[must_use]
    pub const fn end(&self) -> f32 {
        self.end
    }
}

impl MotionDriver for ScrollDriver {
    #[doc = include_str!("scroll_driver.doc/progress.md")]
    fn progress(&self) -> f32 {
        let span = self.end - self.start;
        if span == 0.0 {
            return 0.0;
        }
        ((self.offset - self.start) / span).clamp(0.0, 1.0)
    }

    #[doc = include_str!("scroll_driver.doc/is_active.md")]
    fn is_active(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_starts_at_start_zero_progress() {
        let d = ScrollDriver::new(100.0, 500.0);
        assert_eq!(d.offset(), 100.0);
        assert_eq!(d.start(), 100.0);
        assert_eq!(d.end(), 500.0);
        assert_eq!(d.progress(), 0.0);
    }

    #[test]
    fn offset_at_start_end_and_mid() {
        let base = ScrollDriver::new(0.0, 200.0);
        assert_eq!(base.with_offset(0.0).progress(), 0.0);
        assert_eq!(base.with_offset(200.0).progress(), 1.0);
        assert!((base.with_offset(100.0).progress() - 0.5).abs() < 1e-6);
    }

    #[test]
    fn out_of_range_offsets_clamp() {
        let base = ScrollDriver::new(0.0, 200.0);
        assert_eq!(base.with_offset(-50.0).progress(), 0.0);
        assert_eq!(base.with_offset(400.0).progress(), 1.0);
    }

    #[test]
    fn set_offset_updates_progress() {
        let mut d = ScrollDriver::new(0.0, 4.0);
        d.set_offset(1.0);
        assert!((d.progress() - 0.25).abs() < 1e-6);
        d.set_offset(3.0);
        assert!((d.progress() - 0.75).abs() < 1e-6);
    }

    #[test]
    fn equal_range_yields_zero() {
        let d = ScrollDriver::new(50.0, 50.0).with_offset(50.0);
        assert_eq!(d.progress(), 0.0);
    }

    #[test]
    fn always_active() {
        assert!(ScrollDriver::new(0.0, 1.0).is_active());
    }

    #[test]
    fn object_safe_via_box() {
        let boxed: Box<dyn MotionDriver> = Box::new(ScrollDriver::new(0.0, 2.0).with_offset(1.0));
        assert!((boxed.progress() - 0.5).abs() < 1e-6);
    }
}
