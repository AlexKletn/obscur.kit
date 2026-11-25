use obscur_geometry::{AffineTransform, Point, Size};

use super::{GesturePhase, TransformGestureUpdate};

/// Reusable zoom and pan state. Use the forward transform for rendering and the inverse coordinate mapping for hit testing.
#[derive(Debug, Clone, Copy)]
pub struct ZoomPan {
    scale: f32,
    offset: Point,
    min: f32,
    max: f32,
    // Scale and translation at the start, plus the content point under the anchor.
    gesture: Option<(f32, Point, Point)>,
}

impl ZoomPan {
    /// Creates a view with positive finite scale limits.
    #[must_use]
    pub fn new(min: f32, max: f32) -> Option<Self> {
        if !min.is_finite() || !max.is_finite() || min <= 0.0 || min > max {
            return None;
        }
        Some(Self {
            scale: 1.0_f32.clamp(min, max),
            offset: Point::new(0.0, 0.0),
            min,
            max,
            gesture: None,
        })
    }

    /// The current clamped scale.
    #[must_use]
    pub const fn scale(&self) -> f32 {
        self.scale
    }

    /// Maps content coordinates into viewport coordinates.
    #[must_use]
    pub fn transform(&self) -> AffineTransform {
        AffineTransform::translate(self.offset.x, self.offset.y)
            .multiply(AffineTransform::scale(self.scale, self.scale))
    }

    /// Maps a viewport point back into the content.
    #[must_use]
    pub fn content_point(&self, viewport: Point) -> Point {
        Point::new(
            (viewport.x - self.offset.x) / self.scale,
            (viewport.y - self.offset.y) / self.scale,
        )
    }

    /// Moves content by the given logical viewport displacement and ends any in-progress transform.
    pub fn pan_by(&mut self, delta: Point) -> bool {
        let offset = Point::new(self.offset.x + delta.x, self.offset.y + delta.y);
        if !self.place(self.scale, offset) {
            return false;
        }
        self.gesture = None;
        true
    }

    /// Compensates for subtracting delta from scene coordinates, keeping their viewport positions fixed. The active pinch and its cancellation baseline are rebased too. Nonfinite results leave the camera unchanged.
    pub fn rebase_origin(&mut self, delta: Point) -> bool {
        let shift_offset = |offset: Point, scale: f32| {
            Point::new(offset.x + delta.x * scale, offset.y + delta.y * scale)
        };
        let offset = shift_offset(self.offset, self.scale);
        let mut gesture = self.gesture;
        if let Some((scale, baseline, anchor)) = gesture.as_mut() {
            *baseline = shift_offset(*baseline, *scale);
            *anchor = Point::new(anchor.x - delta.x, anchor.y - delta.y);
            if ![baseline.x, baseline.y, anchor.x, anchor.y]
                .iter()
                .all(|v| v.is_finite())
            {
                return false;
            }
        }
        if !self.place(self.scale, offset) {
            return false;
        }
        self.gesture = gesture;
        true
    }

    /// Applies an absolute scale while keeping the given viewport point over the same content.
    pub fn zoom_at(&mut self, requested_scale: f32, viewport: Point) -> bool {
        if !self.scale_at(requested_scale, viewport, self.content_point(viewport)) {
            return false;
        }
        self.gesture = None;
        true
    }

    /// Fits the content to the viewport, either by both axes or by width. Width-only fitting leaves the content aligned to the top.
    pub fn fit(&mut self, viewport: Size, content: Size, width_only: bool) -> bool {
        if [
            viewport.width,
            viewport.height,
            content.width,
            content.height,
        ]
        .iter()
        .any(|value| !value.is_finite() || *value <= 0.0)
        {
            return false;
        }
        let width = viewport.width / content.width;
        let scale = if width_only {
            width
        } else {
            width.min(viewport.height / content.height)
        }
        .clamp(self.min, self.max);
        let x = (viewport.width - content.width * scale) * 0.5;
        let y = if width_only {
            0.0
        } else {
            (viewport.height - content.height * scale) * 0.5
        };
        if !self.place(scale, Point::new(x, y)) {
            return false;
        }
        self.gesture = None;
        true
    }

    /// Applies pinch scale and midpoint travel. Cancellation restores the original transform. Rotation is available on the sample for surfaces that support it.
    pub fn update(&mut self, sample: TransformGestureUpdate) -> bool {
        if sample.phase == GesturePhase::Cancelled {
            let Some((scale, offset, _)) = self.gesture.take() else {
                return false;
            };
            return self.place(scale, offset);
        }
        let starting = sample.phase == GesturePhase::Started;
        let baseline = if starting {
            (self.scale, self.offset, self.content_point(sample.center))
        } else {
            let Some(baseline) = self.gesture else {
                return false;
            };
            baseline
        };
        if !self.scale_at(baseline.0 * sample.scale, sample.center, baseline.2) {
            if sample.phase == GesturePhase::Completed {
                self.gesture = None;
            }
            return false;
        }
        self.gesture = (sample.phase != GesturePhase::Completed).then_some(baseline);
        true
    }

    fn scale_at(&mut self, requested: f32, viewport: Point, content: Point) -> bool {
        if !requested.is_finite() || requested <= 0.0 {
            return false;
        }
        let scale = requested.clamp(self.min, self.max);
        self.place(
            scale,
            Point::new(
                viewport.x - content.x * scale,
                viewport.y - content.y * scale,
            ),
        )
    }

    fn place(&mut self, scale: f32, offset: Point) -> bool {
        if !offset.x.is_finite() || !offset.y.is_finite() {
            return false;
        }
        self.scale = scale;
        self.offset = offset;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gesture::GestureSource;

    #[test]
    fn pinch_moves_the_anchor_and_cancellation_restores_the_original_view() {
        let mut view = ZoomPan::new(0.5, 2.0).unwrap();
        let original = view.transform();
        let anchor = Point::new(40.0, 30.0);
        let mut sample = TransformGestureUpdate {
            source: GestureSource::Touchscreen,
            finger_count: std::num::NonZeroUsize::new(2),
            phase: GesturePhase::Started,
            center: anchor,
            scale: 1.0,
            rotation: 0.0,
        };
        assert!(view.update(sample));
        sample.phase = GesturePhase::Updated;
        sample.center = Point::new(70.0, 50.0);
        sample.scale = 3.0;
        assert!(view.update(sample));
        assert_eq!(view.scale(), 2.0);
        assert_eq!(view.transform().apply(anchor), sample.center);
        assert_eq!(view.content_point(sample.center), anchor);
        sample.phase = GesturePhase::Cancelled;
        assert!(view.update(sample));
        assert_eq!(view.transform(), original);
    }

    #[test]
    fn rebasing_preserves_an_active_pinch_and_its_cancellation_baseline() {
        let mut camera = ZoomPan::new(0.5, 4.0).unwrap();
        assert!(camera.pan_by(Point::new(-900.0, 300.0)));
        let anchor = Point::new(40.0, 30.0);
        let content = camera.content_point(anchor);
        let mut sample = TransformGestureUpdate {
            source: GestureSource::Touchscreen,
            finger_count: std::num::NonZeroUsize::new(2),
            phase: GesturePhase::Started,
            center: anchor,
            scale: 1.0,
            rotation: 0.0,
        };
        assert!(camera.update(sample));
        sample.phase = GesturePhase::Updated;
        sample.center = Point::new(70.0, 50.0);
        sample.scale = 2.0;
        assert!(camera.update(sample));

        let delta = Point::new(900.0, -300.0);
        let rebased = Point::new(content.x - delta.x, content.y - delta.y);
        assert!(camera.rebase_origin(delta));
        assert_eq!(camera.transform().apply(rebased), sample.center);
        let before_invalid = camera.transform();
        assert!(!camera.rebase_origin(Point::new(f32::MAX, 0.0)));
        assert_eq!(camera.transform(), before_invalid);

        sample.center = Point::new(85.0, 65.0);
        sample.scale = 3.0;
        assert!(camera.update(sample));
        assert_eq!(camera.transform().apply(rebased), sample.center);
        sample.phase = GesturePhase::Cancelled;
        assert!(camera.update(sample));
        assert_eq!(camera.transform().apply(rebased), anchor);
    }
}
