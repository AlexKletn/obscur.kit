//! The [`PostLayoutSample`] post-layout phase sample (spec §4.17 "motion
//! engine", post-layout phase / after-layout geometry).

use obscur_geometry::Rect;

use crate::timeline::TimelineId;
use crate::value::MotionValue;

/// A motion value sampled in the **post-layout** phase (spec §4.17).
///
/// The counterpart to [`PreLayoutSample`](super::PreLayoutSample): a
/// `PostLayoutSample` carries a *presentation-only* animated value — a transform
/// or an opacity — that is applied **after** the layout pass and does **not**
/// change geometry (spec §4.17 "after-layout geometry"). Because it runs after
/// layout, the element's final laid-out box is available: an optional
/// [`post_layout_bounds`](Self::post_layout_bounds) captures that box for
/// post-layout transforms and shared-element work, and is `None` when no bounds
/// were captured.
///
/// The sample records which [`Timeline`](crate::timeline::Timeline) produced it,
/// the timeline [`progress`](Self::progress) at the moment of sampling, the
/// resolved [`MotionValue`](Self::value), and the optional captured bounds.
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
    /// Creates a post-layout sample tagging `value` (sampled at `progress`) to
    /// the `timeline` that produced it, with no captured bounds.
    ///
    /// Use [`with_bounds`](Self::with_bounds) to attach the element's laid-out
    /// box once layout has run.
    #[must_use]
    pub const fn new(timeline: TimelineId, progress: f32, value: MotionValue) -> Self {
        Self {
            timeline,
            progress,
            value,
            post_layout_bounds: None,
        }
    }

    /// Returns this sample with `bounds` recorded as the element's captured
    /// post-layout box.
    ///
    /// Fluent builder: consumes and returns `self` so it can be chained after
    /// [`new`](Self::new).
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

    /// Returns whether this sample affects layout — always `false`.
    ///
    /// A post-layout sample is a presentation-only value applied after layout
    /// (spec §4.17); it never changes geometry. Mirrors
    /// [`PreLayoutSample::affects_layout`](super::PreLayoutSample::affects_layout),
    /// which is always `true`.
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
