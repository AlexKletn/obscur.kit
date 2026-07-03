//! The [`ScrollDriver`] scroll-driven motion source (spec §4.17).

use crate::driver::MotionDriver;

/// A scroll-driven [`MotionDriver`]: a scroll offset mapped onto `[0, 1]` across
/// a scroll range (spec §4.17 "scroll signals — scroll-linked animations").
///
/// The driver maps the current scroll `offset` from the range `[start, end]`
/// that the animation spans onto normalized progress via
/// `(offset - start) / (end - start)`, clamped to `[0, 1]`. This is the **domain
/// half** of a scroll-linked animation: the driver does no scrolling and reads
/// no viewport — the interaction/scroll subsystem feeds it the latest offset via
/// [`set_offset`](Self::set_offset), and the driver deterministically turns that
/// into progress. A degenerate range (`start == end`) has no width to normalize
/// against and yields progress `0`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScrollDriver {
    /// The current scroll position.
    offset: f32,
    /// The scroll position mapped to progress `0` (the animation's start).
    start: f32,
    /// The scroll position mapped to progress `1` (the animation's end).
    end: f32,
}

impl ScrollDriver {
    /// Creates a driver over the scroll range `[start, end]`, starting with its
    /// `offset` at `start` (progress `0`).
    ///
    /// If the endpoints are equal the range is degenerate and progress is always
    /// `0`.
    #[must_use]
    pub const fn new(start: f32, end: f32) -> Self {
        Self {
            offset: start,
            start,
            end,
        }
    }

    /// Returns this driver with its current scroll `offset` set.
    #[must_use]
    pub const fn with_offset(mut self, offset: f32) -> Self {
        self.offset = offset;
        self
    }

    /// Sets the current scroll offset (spec §4.17).
    ///
    /// The scroll subsystem reports its latest scroll position here and the
    /// driver re-normalizes it on the next [`progress`](MotionDriver::progress)
    /// read.
    pub fn set_offset(&mut self, offset: f32) {
        self.offset = offset;
    }

    /// Returns the current scroll offset.
    #[must_use]
    pub const fn offset(&self) -> f32 {
        self.offset
    }

    /// Returns the scroll position mapped to progress `0`.
    #[must_use]
    pub const fn start(&self) -> f32 {
        self.start
    }

    /// Returns the scroll position mapped to progress `1`.
    #[must_use]
    pub const fn end(&self) -> f32 {
        self.end
    }
}

impl MotionDriver for ScrollDriver {
    /// Returns `((offset - start) / (end - start)).clamp(0.0, 1.0)` (spec §4.17).
    ///
    /// A degenerate range (`start == end`) has no width and yields `0.0`.
    fn progress(&self) -> f32 {
        let span = self.end - self.start;
        if span == 0.0 {
            return 0.0;
        }
        ((self.offset - self.start) / span).clamp(0.0, 1.0)
    }

    /// Always `true`: scroll can move either way at any time, so the driver is
    /// never considered settled.
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
