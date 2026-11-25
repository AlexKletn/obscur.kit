//! The [`PreLayoutSample`] pre-layout phase sample (spec §4.17 "motion engine",
//! pre-layout phase / before-layout geometry).

use crate::timeline::TimelineId;
use crate::value::MotionValue;

#[doc = include_str!("../../docs/api/phase/pre_layout_sample.doc/PreLayoutSample.md")]
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

    #[doc = include_str!("../../docs/api/phase/pre_layout_sample.doc/affects_layout.md")]
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
