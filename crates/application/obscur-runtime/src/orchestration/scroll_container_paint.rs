//! What a scroll container contributes to the display list: whether its
//! `overflow` clips its subtree, and the vertical scrollbar painted in the
//! scrollport's right gutter (CSS Overflow 3 §scroll-container, §scrollbars).

use obscur_geometry::{Point, Rect, Size};
use obscur_scene::color::Color;
use obscur_scene::command::{DisplayListBuilder, PaintCommand};
use obscur_scene::paint::Brush;
use obscur_style::computed::{ComputedStyle, OverflowAxes};

/// The width, in logical pixels, of a scroll container's vertical scrollbar
/// (track and thumb) painted in the right gutter of the scrollport.
const SCROLLBAR_WIDTH: f32 = 8.0;

/// The smallest a scrollbar thumb is allowed to shrink to (logical px), so a very
/// tall content still leaves a grabbable-sized thumb.
const SCROLLBAR_MIN_THUMB: f32 = 16.0;

/// The scrollbar track color: a faint translucent wash so it reads as a gutter
/// without dominating. A host has no hook to override this yet (a documented
/// fixed default, like the selection highlight).
const SCROLLBAR_TRACK_COLOR: Color = Color::rgba(0.0, 0.0, 0.0, 0.06);

/// The scrollbar thumb color: a darker translucent grey over the track.
const SCROLLBAR_THUMB_COLOR: Color = Color::rgba(0.0, 0.0, 0.0, 0.35);

#[doc = include_str!("../../docs/api/orchestration/scroll_container_paint.doc/overflow_clips.md")]
pub(super) fn overflow_clips(style: Option<&ComputedStyle>) -> bool {
    OverflowAxes::of(style).clips()
}

#[doc = include_str!("../../docs/api/orchestration/scroll_container_paint.doc/scrollbar_commands.md")]
#[must_use]
pub(super) fn scrollbar_commands(
    scrollport: Rect,
    scroll_top: f32,
    content_height: f32,
    horizontal: bool,
) -> Vec<PaintCommand> {
    let parts = if horizontal {
        horizontal_scrollbar_rects(scrollport, scroll_top, content_height)
    } else {
        scrollbar_rects(scrollport, scroll_top, content_height)
    };
    let Some((track, thumb)) = parts else {
        return Vec::new();
    };
    let mut builder = DisplayListBuilder::new();
    builder.fill_rect(track, Brush::solid(SCROLLBAR_TRACK_COLOR));
    builder.fill_rect(thumb, Brush::solid(SCROLLBAR_THUMB_COLOR));
    builder.build().commands().to_vec()
}

#[doc = include_str!("../../docs/api/orchestration/scroll_container_paint.doc/scrollbar_rects.md")]
#[must_use]
pub(crate) fn scrollbar_rects(
    scrollport: Rect,
    scroll_top: f32,
    content_height: f32,
) -> Option<(Rect, Rect)> {
    let track_h = scrollport.size.height;
    let track_w = SCROLLBAR_WIDTH.min(scrollport.size.width);
    let scroll_range = content_height - track_h;
    if track_h <= 0.0 || track_w <= 0.0 || scroll_range <= 0.0 {
        return None;
    }
    let track_x = scrollport.right() - track_w;
    let track_y = scrollport.origin.y;
    let track = Rect::new(Point::new(track_x, track_y), Size::new(track_w, track_h));

    // Thumb length = the visible fraction of the content, floored to a grabbable
    // minimum but never taller than the track.
    let thumb_h =
        (track_h * track_h / content_height).clamp(SCROLLBAR_MIN_THUMB.min(track_h), track_h);
    let free = (track_h - thumb_h).max(0.0);
    let thumb_y = track_y + free * (scroll_top / scroll_range).clamp(0.0, 1.0);
    let thumb = Rect::new(Point::new(track_x, thumb_y), Size::new(track_w, thumb_h));
    Some((track, thumb))
}

/// Convert the signed offset to a distance from the left end of the scrollbar track.
pub(crate) fn horizontal_thumb_offset(
    scrollport: Rect,
    scroll_left: f32,
    content_width: f32,
    rtl: bool,
) -> f32 {
    if rtl {
        (content_width - scrollport.size.width).max(0.0) + scroll_left
    } else {
        scroll_left
    }
}

/// Track and thumb rectangles on the bottom edge of a horizontal scrollport.
pub(crate) fn horizontal_scrollbar_rects(
    port: Rect,
    offset: f32,
    extent: f32,
) -> Option<(Rect, Rect)> {
    let transpose = |r: Rect| {
        Rect::new(
            Point::new(r.origin.y, r.origin.x),
            Size::new(r.size.height, r.size.width),
        )
    };
    scrollbar_rects(transpose(port), offset, extent)
        .map(|(track, thumb)| (transpose(track), transpose(thumb)))
}
