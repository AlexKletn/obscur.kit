//! The arranged box of a node.

use obscur_geometry::{Insets, Point, Rect, Size};

/// The result of arranging a node: its border-box rectangle in parent-relative
/// logical pixels (spec §4.2).
///
/// Fractional geometry is preserved until paint time; no rounding is applied
/// here (spec §4.6).
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct LayoutBox {
    /// Arranged border-box bounds in parent-relative logical pixels.
    pub bounds: Rect,
}

impl LayoutBox {
    /// Creates a layout box from arranged border-box bounds.
    #[must_use]
    pub const fn new(bounds: Rect) -> Self {
        Self { bounds }
    }

    /// Creates a layout box sized to `size` with its origin at `(0, 0)`.
    #[must_use]
    pub const fn from_size(size: Size) -> Self {
        Self {
            bounds: Rect::new(Point::ORIGIN, size),
        }
    }

    /// The size of the border box.
    #[must_use]
    pub const fn size(self) -> Size {
        self.bounds.size
    }

    /// The origin of the border box, relative to the parent.
    #[must_use]
    pub const fn origin(self) -> Point {
        self.bounds.origin
    }

    /// The content box: the border box inset by `border` then `padding`.
    ///
    /// The content origin is offset by the top/left of `border` + `padding`, and
    /// the content extent is the border-box extent minus the corresponding
    /// edges. Each extent is clamped to be non-negative, so a box smaller than
    /// its insets yields a zero-extent content rect rather than a negative one.
    /// This never panics.
    #[must_use]
    pub fn content_rect(self, padding: Insets, border: Insets) -> Rect {
        let left = border.left + padding.left;
        let top = border.top + padding.top;
        let right = border.right + padding.right;
        let bottom = border.bottom + padding.bottom;

        let origin = Point::new(self.bounds.origin.x + left, self.bounds.origin.y + top);
        let width = (self.bounds.size.width - left - right).max(0.0);
        let height = (self.bounds.size.height - top - bottom).max(0.0);

        Rect::new(origin, Size::new(width, height))
    }
}

#[cfg(test)]
mod tests {
    use super::LayoutBox;
    use obscur_geometry::{Insets, Point, Rect, Size};

    #[test]
    fn from_size_places_origin_at_zero() {
        let b = LayoutBox::from_size(Size::new(30.0, 40.0));
        assert_eq!(b.origin(), Point::ORIGIN);
        assert_eq!(b.size(), Size::new(30.0, 40.0));
    }

    #[test]
    fn accessors_read_bounds() {
        let b = LayoutBox::new(Rect::new(Point::new(4.0, 5.0), Size::new(10.0, 12.0)));
        assert_eq!(b.origin(), Point::new(4.0, 5.0));
        assert_eq!(b.size(), Size::new(10.0, 12.0));
    }

    #[test]
    fn content_rect_insets_by_border_and_padding() {
        let b = LayoutBox::new(Rect::new(Point::new(10.0, 10.0), Size::new(100.0, 80.0)));
        let padding = Insets::uniform(4.0);
        let border = Insets::uniform(1.0);
        let content = b.content_rect(padding, border);
        assert_eq!(content.origin, Point::new(15.0, 15.0));
        assert_eq!(content.size, Size::new(90.0, 70.0));
    }

    #[test]
    fn content_rect_clamps_to_non_negative() {
        let b = LayoutBox::from_size(Size::new(6.0, 6.0));
        let padding = Insets::uniform(5.0);
        let border = Insets::uniform(5.0);
        let content = b.content_rect(padding, border);
        assert_eq!(content.size, Size::ZERO);
    }
}
