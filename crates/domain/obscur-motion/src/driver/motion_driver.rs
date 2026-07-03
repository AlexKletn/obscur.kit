//! The [`MotionDriver`] driver port shared by every motion source (spec §4.17).
//!
//! A *driver* is the **source** of a timeline's progression: it turns some
//! external input — elapsed logical time, an explicit scrubber value, a
//! drag/cursor/gesture signal, or a scroll offset — into a single normalized
//! progress in `[0, 1]`. That progress is what feeds a track's `sample(t)`
//! downstream. Drivers are deliberately narrow (one read of progress plus a
//! liveness flag) so a heterogeneous set of sources can be held behind
//! `dyn MotionDriver`.
//!
//! Like the rest of the motion engine, drivers are **pure and deterministic**
//! and **time-agnostic**: a driver never reads a clock. Every input is pushed in
//! explicitly (a [`TimeDriver`](super::TimeDriver) accumulates an explicit
//! `delta`, a scroll/signal driver is handed its latest value), so a skipped
//! frame is just a larger input and runs through the same math.

/// A source of normalized motion progress sampled by tracks (spec §4.17).
///
/// Every driver — [`TimeDriver`](super::TimeDriver),
/// [`ProgressDriver`](super::ProgressDriver),
/// [`SignalDriver`](super::SignalDriver),
/// [`ScrollDriver`](super::ScrollDriver) — implements this port: it reports the
/// [`progress`](Self::progress) it currently dictates and whether it
/// [`is_active`](Self::is_active). The trait is deliberately **object-safe** (no
/// generics, no `Self`-returning methods) so drivers of different kinds can be
/// held together as `Box<dyn MotionDriver>` or borrowed as `&dyn MotionDriver`.
pub trait MotionDriver {
    /// Returns the normalized progress this driver currently dictates (spec
    /// §4.17).
    ///
    /// The value is normally clamped to `[0, 1]` — each implementation documents
    /// exactly how it derives and bounds it — and is what feeds a track's
    /// `sample(t)`. Pure and deterministic: reading the same driver in the same
    /// state always yields the same progress.
    #[must_use]
    fn progress(&self) -> f32;

    /// Returns whether the driver is still producing motion (spec §4.17).
    ///
    /// The default is `true`: a driver keeps going unless it can settle or
    /// finish. Drivers that reach a terminal state — such as a
    /// [`TimeDriver`](super::TimeDriver) whose elapsed time has caught up to its
    /// duration — override this to report when they have stopped advancing.
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
