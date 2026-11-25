//! The [`PostLayoutSample`] post-layout phase sample (spec §4.17 "motion
//! engine", post-layout phase / after-layout geometry).

use obscur_geometry::Rect;

use crate::timeline::TimelineId;
use crate::value::MotionValue;

#[doc = include_str!("../../docs/api/phase/post_layout_sample.doc/PostLayoutSample.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PostLayoutSample {
    /// The timeline that produced this sample.
    timeline: TimelineId,
    /// The timeline progress at the moment of sampling (normalized `0..=1`).
    progress: f32,
    /// The sampled presentation-only value (e.g. an animated transform/opacity).
    value: MotionValue,
    /// The element's final laid-out box, captured after layout; `None` when it
    /// was not captured.
    post_layout_bounds: Option<Rect>,
}

impl PostLayoutSample {
    #[doc = include_str!("../../docs/api/phase/post_layout_sample.doc/new.md")]
    #[must_use]
    pub const fn new(timeline: TimelineId, progress: f32, value: MotionValue) -> Self {
        Self {
            timeline,
            progress,
            value,
            post_layout_bounds: None,
        }
    }

    #[doc = include_str!("../../docs/api/phase/post_layout_sample.doc/with_bounds.md")]
    #[must_use]
    pub const fn with_bounds(mut self, bounds: Rect) -> Self {
        self.post_layout_bounds = Some(bounds);
        self
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

    /// Returns the sampled presentation-only value.
    #[must_use]
    pub const fn value(&self) -> MotionValue {
        self.value
    }

    /// Returns the element's captured post-layout box, or `None` if none was
    /// captured.
    #[must_use]
    pub const fn post_layout_bounds(&self) -> Option<Rect> {
        self.post_layout_bounds
    }

    #[doc = include_str!("../../docs/api/phase/post_layout_sample.doc/affects_layout.md")]
    #[must_use]
    pub const fn affects_layout(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_geometry::{Point, Size};

    fn id(n: u64) -> TimelineId {
        TimelineId::from_raw(n)
    }

    #[test]
    fn new_exposes_fields_and_no_bounds() {
        let value = MotionValue::scalar(0.5);
        let sample = PostLayoutSample::new(id(3), 0.75, value);
        assert_eq!(sample.timeline(), id(3));
        assert_eq!(sample.progress(), 0.75);
        assert_eq!(sample.value(), value);
        assert_eq!(sample.post_layout_bounds(), None);
    }

    #[test]
    fn never_affects_layout() {
        let sample = PostLayoutSample::new(id(1), 0.0, MotionValue::scalar(1.0));
        assert!(!sample.affects_layout());
    }

    #[test]
    fn with_bounds_captures_box() {
        let bounds = Rect::new(Point::new(1.0, 2.0), Size::new(3.0, 4.0));
        let sample =
            PostLayoutSample::new(id(1), 0.0, MotionValue::scalar(1.0)).with_bounds(bounds);
        assert_eq!(sample.post_layout_bounds(), Some(bounds));
    }
}
