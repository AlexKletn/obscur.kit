#![doc = include_str!("scroll_paint.doc/_module.md")]

#[doc = include_str!("scroll_paint.doc/ScrollPaint.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScrollPaint {
    #[doc = include_str!("scroll_paint.doc/scroll_top.md")]
    pub scroll_top: f32,
    #[doc = include_str!("scroll_paint.doc/content_height.md")]
    pub content_height: f32,
    #[doc = include_str!("scroll_paint.doc/shows_scrollbar.md")]
    pub shows_scrollbar: bool,
}
