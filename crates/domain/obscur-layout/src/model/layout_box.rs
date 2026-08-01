#![doc = include_str!("layout_box.doc/_module.md")]

use obscur_geometry::{Insets, Point, Rect, Size};

#[doc = include_str!("layout_box.doc/LayoutBox.md")]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct LayoutBox {
    #[doc = include_str!("layout_box.doc/bounds.md")]
    pub bounds: Rect,
}

impl LayoutBox {
    #[doc = include_str!("layout_box.doc/new.md")]
    #[must_use]
    pub const fn new(bounds: Rect) -> Self {
        Self { bounds }
    }

    #[doc = include_str!("layout_box.doc/from_size.md")]
    #[must_use]
    pub const fn from_size(size: Size) -> Self {
        Self {
            bounds: Rect::new(Point::ORIGIN, size),
        }
    }

    #[doc = include_str!("layout_box.doc/size.md")]
    #[must_use]
    pub const fn size(self) -> Size {
        self.bounds.size
    }

    #[doc = include_str!("layout_box.doc/origin.md")]
    #[must_use]
    pub const fn origin(self) -> Point {
        self.bounds.origin
    }

    #[doc = include_str!("layout_box.doc/content_rect.md")]
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
