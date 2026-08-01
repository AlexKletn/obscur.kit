#![doc = include_str!("pre_layout_sample.doc/_module.md")]

use crate::timeline::TimelineId;
use crate::value::MotionValue;

#[doc = include_str!("pre_layout_sample.doc/PreLayoutSample.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PreLayoutSample {
    #[doc = include_str!("pre_layout_sample.doc/timeline.md")]
    timeline: TimelineId,
    #[doc = include_str!("pre_layout_sample.doc/progress.md")]
    progress: f32,
    #[doc = include_str!("pre_layout_sample.doc/value.md")]
    value: MotionValue,
}

impl PreLayoutSample {
    #[doc = include_str!("pre_layout_sample.doc/new.md")]
    #[must_use]
    pub const fn new(timeline: TimelineId, progress: f32, value: MotionValue) -> Self {
        Self {
            timeline,
            progress,
            value,
        }
    }

    #[doc = include_str!("pre_layout_sample.doc/timeline_2.md")]
    #[must_use]
    pub const fn timeline(&self) -> TimelineId {
        self.timeline
    }

    #[doc = include_str!("pre_layout_sample.doc/progress_2.md")]
    #[must_use]
    pub const fn progress(&self) -> f32 {
        self.progress
    }

    #[doc = include_str!("pre_layout_sample.doc/value_2.md")]
    #[must_use]
    pub const fn value(&self) -> MotionValue {
        self.value
    }

    #[doc = include_str!("pre_layout_sample.doc/affects_layout.md")]
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
