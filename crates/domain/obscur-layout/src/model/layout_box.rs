//! The arranged box of a node.

use obscur_geometry::{Insets, Point, Rect, Size};

#[doc = include_str!("../../docs/api/model/layout_box.doc/LayoutBox.md")]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct LayoutBox {
    /// Arranged border-box bounds in parent-relative logical pixels.
    pub bounds: Rect,
    content_insets: Option<Insets>,
}

impl LayoutBox {
    /// Creates a layout box from arranged border-box bounds.
    #[must_use]
    pub const fn new(bounds: Rect) -> Self {
        Self {
            bounds,
            content_insets: None,
        }
    }

    /// Creates a layout box sized to `size` with its origin at `(0, 0)`.
    #[must_use]
    pub const fn from_size(size: Size) -> Self {
        Self::new(Rect::new(Point::ORIGIN, size))
    }

    /// Records the used border and padding offsets from the border box.
    #[must_use]
    pub const fn with_content_insets(mut self, insets: Insets) -> Self {
        self.content_insets = Some(insets);
        self
    }

    /// Returns the content rectangle when the arrange pass captured its used insets.
    #[must_use]
    pub fn used_content_rect(self) -> Option<Rect> {
        self.content_insets
            .map(|insets| self.content_rect(insets, Insets::uniform(0.0)))
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

    #[doc = include_str!("../../docs/api/model/layout_box.doc/content_rect.md")]
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
