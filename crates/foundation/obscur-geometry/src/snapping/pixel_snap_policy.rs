//! Physical pixel alignment for logical paint rectangles.

use crate::{Point, Rect, Size, snapping::PixelSnapMode};

/// Pixel alignment policy applied to a copy of paint geometry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PixelSnapPolicy {
    mode: PixelSnapMode,
}

impl PixelSnapPolicy {
    /// Creates a policy from the requested snap mode.
    #[must_use]
    pub const fn new(mode: PixelSnapMode) -> Self {
        Self { mode }
    }

    /// Snaps each edge to the nearest physical pixel in `Edges` mode, then returns
    /// the rectangle in logical coordinates. `Auto` and `None` preserve the rectangle.
    /// A nonpositive or non-finite scale also preserves the original rectangle.
    #[must_use]
    pub fn snap_rect(self, rect: Rect, device_scale: f32) -> Rect {
        if self.mode != PixelSnapMode::Edges || !device_scale.is_finite() || device_scale <= 0.0 {
            return rect;
        }
        let snap = |edge: f32| (edge * device_scale).round() / device_scale;
        let [left, top, right, bottom] = [
            snap(rect.origin.x),
            snap(rect.origin.y),
            snap(rect.right()),
            snap(rect.bottom()),
        ];
        let width = right - left;
        let height = bottom - top;
        if [left, top, width, height]
            .iter()
            .all(|value| value.is_finite())
            && width >= 0.0
            && height >= 0.0
        {
            Rect::new(Point::new(left, top), Size::new(width, height))
        } else {
            rect
        }
    }
}
