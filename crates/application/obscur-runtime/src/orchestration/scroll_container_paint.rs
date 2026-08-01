#![doc = include_str!("scroll_container_paint.doc/_module.md")]

use obscur_geometry::{Point, Rect, Size};
use obscur_identity::NodeId;
use obscur_scene::color::Color;
use obscur_scene::command::{DisplayListBuilder, PaintCommand};
use obscur_scene::paint::Brush;
use obscur_style::computed::ComputedValue;
use obscur_style::property::{PropertyId, PropertyValue, layout_keyword};

use super::ComputedStyles;

#[doc = include_str!("scroll_container_paint.doc/SCROLLBAR_WIDTH.md")]
const SCROLLBAR_WIDTH: f32 = 8.0;

#[doc = include_str!("scroll_container_paint.doc/SCROLLBAR_MIN_THUMB.md")]
const SCROLLBAR_MIN_THUMB: f32 = 16.0;

#[doc = include_str!("scroll_container_paint.doc/SCROLLBAR_TRACK_COLOR.md")]
const SCROLLBAR_TRACK_COLOR: Color = Color::rgba(0.0, 0.0, 0.0, 0.06);

#[doc = include_str!("scroll_container_paint.doc/SCROLLBAR_THUMB_COLOR.md")]
const SCROLLBAR_THUMB_COLOR: Color = Color::rgba(0.0, 0.0, 0.0, 0.35);

#[doc = include_str!("scroll_container_paint.doc/overflow_clips.md")]
pub(super) fn overflow_clips(computed: &ComputedStyles, node: NodeId) -> bool {
    matches!(
        computed
            .get(&node)
            .and_then(|style| style.get(PropertyId::Overflow))
            .map(ComputedValue::value),
        Some(&PropertyValue::Keyword(sym)) if sym != layout_keyword::OVERFLOW_VISIBLE
    )
}

#[doc = include_str!("scroll_container_paint.doc/scrollbar_commands.md")]
#[must_use]
pub(super) fn scrollbar_commands(
    scrollport: Rect,
    scroll_top: f32,
    content_height: f32,
) -> Vec<PaintCommand> {
    let Some((track, thumb)) = scrollbar_rects(scrollport, scroll_top, content_height) else {
        return Vec::new();
    };
    let mut builder = DisplayListBuilder::new();
    builder.fill_rect(track, Brush::solid(SCROLLBAR_TRACK_COLOR));
    builder.fill_rect(thumb, Brush::solid(SCROLLBAR_THUMB_COLOR));
    builder.build().commands().to_vec()
}

#[doc = include_str!("scroll_container_paint.doc/scrollbar_rects.md")]
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
