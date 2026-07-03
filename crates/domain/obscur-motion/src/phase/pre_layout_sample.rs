//! The [`PreLayoutSample`] pre-layout phase sample (spec §4.17 "motion engine",
//! pre-layout phase / before-layout geometry).

use crate::timeline::TimelineId;
use crate::value::MotionValue;

/// A motion value sampled in the **pre-layout** phase (spec §4.17).
///
/// The motion engine produces presentation values in two phases keyed to the
/// layout pass. A `PreLayoutSample` carries a *layout-affecting* animated value —
/// one that changes geometry, such as an animated width, height or margin — that
/// must be resolved **before** the layout pass runs, so the layout engine reads
/// the animated input while measuring and arranging (spec §4.17 "before-layout
/// geometry"). Contrast [`PostLayoutSample`](super::PostLayoutSample), which
/// carries presentation-only values applied after layout.
///
/// The sample records which [`Timeline`](crate::timeline::Timeline) produced it,
/// the timeline [`progress`](Self::progress) at the moment of sampling, and the
/// resolved [`MotionValue`](Self::value). It is a plain immutable record: the
/// timeline is advanced elsewhere and each frame yields a fresh sample.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PreLayoutSample {
    /// The timeline that produced this sample.
    timeline: TimelineId,
    /// The timeline progress at the moment of sampling (normalized `0..=1`).
    progress: f32,
    /// The sampled layout-affecting value (e.g. an animated `Size` width).
    value: MotionValue,
}

impl PreLayoutSample {
    /// Creates a pre-layout sample tagging `value` (sampled at `progress`) to the
    /// `timeline` that produced it.
    #[must_use]
    pub const fn new(timeline: TimelineId, progress: f32, value: MotionValue) -> Self {
        Self {
            timeline,
            progress,
            value,
        }
    }

    /// Returns the timeline that produced this sample.
    #[must_use]
    pub const fn timeline(&self) -> TimelineId {
        self.timeline
    }

    /// Returns the timeline progress at the moment of sampling.
    #[must_use]
    pub const fn progress(&self) -> f32 {
        self.progress
    }

    /// Returns the sampled layout-affecting value.
    #[must_use]
    pub const fn value(&self) -> MotionValue {
        self.value
    }

    /// Returns whether this sample affects layout — always `true`.
    ///
    /// A pre-layout sample is, by definition, a layout-affecting value resolved
    /// before the layout pass runs (spec §4.17); this constant mirrors
    /// [`PostLayoutSample::affects_layout`](super::PostLayoutSample::affects_layout),
    /// which is always `false`, so the two phases can be discriminated uniformly.
    #[must_use]
    pub const fn affects_layout(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_geometry::Size;

    fn id(n: u64) -> TimelineId {
        TimelineId::from_raw(n)
    }

    #[test]
    fn new_exposes_fields() {
        let value = MotionValue::size(Size::new(120.0, 40.0));
        let sample = PreLayoutSample::new(id(7), 0.25, value);
        assert_eq!(sample.timeline(), id(7));
        assert_eq!(sample.progress(), 0.25);
        assert_eq!(sample.value(), value);
    }

    #[test]
    fn always_affects_layout() {
        let sample = PreLayoutSample::new(id(1), 0.0, MotionValue::scalar(10.0));
        assert!(sample.affects_layout());
    }
}
