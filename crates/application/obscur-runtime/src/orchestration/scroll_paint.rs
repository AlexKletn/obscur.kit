//! The live scroll state a scroll container is composed with: [`ScrollPaint`].

use obscur_geometry::Point;

#[doc = include_str!("../../docs/api/orchestration/scroll_paint.doc/ScrollPaint.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScrollPaint {
    /// The current vertical scroll position (logical px), already clamped by the
    /// runtime to `0 ..= content_height − scrollport_height`.
    pub scroll_top: f32,
    /// The scrollable content height (logical px), `> scrollport_height`.
    pub content_height: f32,
    #[doc = include_str!("../../docs/api/orchestration/scroll_paint.doc/shows_scrollbar.md")]
    pub shows_scrollbar: bool,
    /// Horizontal scrollbar paint input.
    pub scroll_left: f32,
    /// Horizontal scrollbar paint input.
    pub content_width: f32,
    /// Whether the horizontal scroll origin is on the right edge.
    pub rtl: bool,
    /// Horizontal scrollbar paint input.
    pub shows_horizontal_scrollbar: bool,
}

impl ScrollPaint {
    pub(crate) fn offset(self) -> Point {
        Point::new(self.scroll_left, self.scroll_top)
    }
}
