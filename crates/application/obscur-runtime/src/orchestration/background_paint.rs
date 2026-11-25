//! A node's `background-color` and `background-image` painting: the fill brush,
//! each `background-image` layer, and the `background-size` / `-position` /
//! `-repeat` tile geometry a resolved image is drawn with (spec §5.2).

use std::collections::HashMap;

use obscur_geometry::{CornerRadii, CornerRadius, Point, Rect, Size};
use obscur_scene::clip::Clip;
use obscur_scene::command::DisplayListBuilder;
use obscur_scene::paint::{Brush, SceneImage, SceneVector};
use obscur_scene::path::Path;
use obscur_style::computed::{ComputedStyle, ComputedValue};
use obscur_style::property::{
    BackgroundAttachment, BackgroundBox, BackgroundLayer, BackgroundPosition, BackgroundRepeat,
    BackgroundRepeatAxis, BackgroundSize, Length, PropertyId, PropertyValue, StyleColor,
};

use super::gradient_brush::{conic_gradient_brush, linear_gradient_brush, radial_gradient_brush};

use super::paint_color::{current_color_of, scene_color_from};
use super::replaced_content::emit_vector_tile;

pub(super) fn attachment_at(
    style: Option<&ComputedStyle>,
    layer_index: usize,
) -> BackgroundAttachment {
    match style
        .and_then(|style| style.get(PropertyId::BackgroundAttachment))
        .map(ComputedValue::value)
    {
        Some(PropertyValue::BackgroundAttachments(values)) if !values.is_empty() => {
            values[layer_index % values.len()]
        }
        _ => BackgroundAttachment::Scroll,
    }
}

pub(super) fn background_clips(
    style: Option<&ComputedStyle>,
    layout: obscur_layout::model::LayoutBox,
    outer_radii: CornerRadii,
    layer_index: usize,
    scroll: Option<&super::ScrollPaint>,
) -> [Option<Clip>; 2] {
    let kind = match style
        .and_then(|style| style.get(PropertyId::BackgroundClip))
        .map(ComputedValue::value)
    {
        Some(PropertyValue::BackgroundClips(clips)) if !clips.is_empty() => {
            clips[layer_index % clips.len()]
        }
        _ => BackgroundBox::Border,
    };
    let bounds = layout.bounds;
    if let Some(scroll) = scroll {
        let port = background_box_rect(BackgroundBox::Padding, style, layout);
        let port_radii = inset_background_radii(outer_radii, bounds, port);
        let content = background_box_rect(
            if kind == BackgroundBox::Border {
                BackgroundBox::Padding
            } else {
                kind
            },
            style,
            layout,
        );
        let radii = inset_background_radii(outer_radii, bounds, content);
        let local = local_background_rect(content, port, scroll);
        return [
            Some(Clip::rounded_rect(port, port_radii)),
            Some(Clip::rounded_rect(local, radii.normalized(local))),
        ];
    }
    if kind == BackgroundBox::Border {
        return [None, None];
    }
    let clip = background_box_rect(kind, style, layout);
    [
        Some(Clip::rounded_rect(
            clip,
            inset_background_radii(outer_radii, bounds, clip),
        )),
        None,
    ]
}

pub(super) fn local_background_rect(area: Rect, port: Rect, scroll: &super::ScrollPaint) -> Rect {
    let width = (scroll.content_width - port.size.width).max(0.0);
    let height = (scroll.content_height - port.size.height).max(0.0);
    Rect::new(
        Point::new(
            area.origin.x - scroll.scroll_left,
            area.origin.y - scroll.scroll_top,
        ),
        Size::new(area.size.width + width, area.size.height + height),
    )
}

pub(super) fn inset_background_radii(
    outer_radii: CornerRadii,
    bounds: Rect,
    clip: Rect,
) -> CornerRadii {
    let left = (clip.origin.x - bounds.origin.x).max(0.0);
    let top = (clip.origin.y - bounds.origin.y).max(0.0);
    let right = (bounds.right() - clip.right()).max(0.0);
    let bottom = (bounds.bottom() - clip.bottom()).max(0.0);
    let inset = |radius: CornerRadius, x: f32, y: f32| {
        CornerRadius::new((radius.x - x).max(0.0), (radius.y - y).max(0.0))
    };
    CornerRadii {
        top_left: inset(outer_radii.top_left, left, top),
        top_right: inset(outer_radii.top_right, right, top),
        bottom_right: inset(outer_radii.bottom_right, right, bottom),
        bottom_left: inset(outer_radii.bottom_left, left, bottom),
    }
    .normalized(clip)
}

pub(super) fn background_origin(
    style: Option<&ComputedStyle>,
    layout: obscur_layout::model::LayoutBox,
    layer_index: usize,
) -> Rect {
    let kind = match style
        .and_then(|style| style.get(PropertyId::BackgroundOrigin))
        .map(ComputedValue::value)
    {
        Some(PropertyValue::BackgroundOrigins(boxes)) if !boxes.is_empty() => {
            boxes[layer_index % boxes.len()]
        }
        _ => BackgroundBox::Padding,
    };
    background_box_rect(kind, style, layout)
}

pub(super) fn background_box_rect(
    kind: BackgroundBox,
    style: Option<&ComputedStyle>,
    layout: obscur_layout::model::LayoutBox,
) -> Rect {
    match kind {
        BackgroundBox::Border => layout.bounds,
        BackgroundBox::Padding => super::border_paint::overflow_padding_box(style, layout.bounds),
        BackgroundBox::Content => super::replaced_content::replaced_region(style, layout),
    }
}

#[doc = include_str!("../../docs/api/orchestration/background_paint.doc/background_color_of.md")]
pub(super) fn background_color_of(style: Option<&ComputedStyle>) -> Option<StyleColor> {
    style
        .and_then(|style| style.get(PropertyId::BackgroundColor))
        .and_then(ComputedValue::as_color)
}

#[doc = include_str!("../../docs/api/orchestration/background_paint.doc/background_gradient_brush.md")]
fn background_gradient_brush(value: &ComputedValue, bounds: Rect) -> Option<Brush> {
    if let Some(gradient) = value.as_linear_gradient() {
        return (!gradient.stops().is_empty())
            .then(|| Brush::Linear(linear_gradient_brush(gradient, bounds)));
    }
    if let Some(gradient) = value.as_radial_gradient() {
        return (!gradient.stops().is_empty())
            .then(|| Brush::Radial(radial_gradient_brush(gradient, bounds)));
    }
    if let Some(gradient) = value.as_conic_gradient() {
        return (!gradient.stops().is_empty())
            .then(|| Brush::Conic(conic_gradient_brush(gradient, bounds)));
    }
    None
}

pub(super) fn paint_single_background_gradient(
    paint: &mut DisplayListBuilder,
    style: Option<&ComputedStyle>,
    positioning: Rect,
    bounds: Rect,
    radii: CornerRadii,
    has_radius: bool,
) {
    let Some(value) = style.and_then(|style| style.get(PropertyId::BackgroundImage)) else {
        return;
    };
    if value.as_linear_gradient().is_none()
        && value.as_radial_gradient().is_none()
        && value.as_conic_gradient().is_none()
    {
        return;
    }
    paint_gradient_tiles(
        paint,
        style,
        0,
        positioning,
        bounds,
        radii,
        has_radius,
        |tile| background_gradient_brush(value, tile),
    );
}

fn paint_gradient_tiles(
    paint: &mut DisplayListBuilder,
    style: Option<&ComputedStyle>,
    layer_index: usize,
    positioning: Rect,
    bounds: Rect,
    radii: CornerRadii,
    has_radius: bool,
    brush_for: impl Fn(Rect) -> Option<Brush>,
) {
    let (tile, repeat) = background_image_geometry(style, layer_index, None, positioning);
    if tile == bounds {
        if let Some(brush) = brush_for(tile) {
            fill_background_box(paint, bounds, radii, has_radius, brush);
        }
        return;
    }

    let mut tiles = background_tiles(tile, positioning, bounds, repeat).peekable();
    if tiles.peek().is_none() {
        return;
    }
    let clip = if has_radius {
        Clip::rounded_rect(bounds, radii)
    } else {
        Clip::rect(bounds)
    };
    paint.push_clip(clip);
    for tile in tiles {
        if let Some(brush) = brush_for(tile) {
            paint.fill_rect(tile, brush);
        }
    }
    paint.pop_clip();
}

#[doc = include_str!("../../docs/api/orchestration/background_paint.doc/background_layers_of.md")]
pub(super) fn background_layers_of(style: Option<&ComputedStyle>) -> Option<&[BackgroundLayer]> {
    style
        .and_then(|style| style.get(PropertyId::BackgroundImage))
        .and_then(ComputedValue::as_background_layers)
}

/// Fills a node's border box with `brush`, choosing a rounded-rect outline when
/// the node has a positive corner radius and a plain rectangle otherwise —
/// the shared fill used by both the single background path and each background
/// layer.
fn fill_background_box(
    paint: &mut DisplayListBuilder,
    bounds: Rect,
    radii: CornerRadii,
    has_radius: bool,
    brush: Brush,
) {
    if has_radius {
        paint.fill(Path::rounded_rect(bounds, radii), brush);
    } else {
        paint.fill_rect(bounds, brush);
    }
}

#[doc = include_str!("../../docs/api/orchestration/background_paint.doc/paint_background_layer.md")]
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_background_layer(
    paint: &mut DisplayListBuilder,
    layer: &BackgroundLayer,
    style: Option<&ComputedStyle>,
    layer_index: usize,
    positioning: Rect,
    bounds: Rect,
    radii: CornerRadii,
    has_radius: bool,
    images: &HashMap<Box<str>, SceneImage>,
    vectors: &HashMap<Box<str>, SceneVector>,
) {
    match layer {
        BackgroundLayer::Linear(gradient) if !gradient.stops().is_empty() => {
            paint_gradient_tiles(
                paint,
                style,
                layer_index,
                positioning,
                bounds,
                radii,
                has_radius,
                |tile| Some(Brush::Linear(linear_gradient_brush(gradient, tile))),
            );
        }
        BackgroundLayer::Radial(gradient) if !gradient.stops().is_empty() => {
            paint_gradient_tiles(
                paint,
                style,
                layer_index,
                positioning,
                bounds,
                radii,
                has_radius,
                |tile| Some(Brush::Radial(radial_gradient_brush(gradient, tile))),
            );
        }
        BackgroundLayer::Conic(gradient) if !gradient.stops().is_empty() => {
            paint_gradient_tiles(
                paint,
                style,
                layer_index,
                positioning,
                bounds,
                radii,
                has_radius,
                |tile| Some(Brush::Conic(conic_gradient_brush(gradient, tile))),
            );
        }
        BackgroundLayer::Url(url) => {
            paint_background_url(
                paint,
                url,
                style,
                layer_index,
                positioning,
                bounds,
                images,
                vectors,
            );
        }
        BackgroundLayer::None
        | BackgroundLayer::Linear(_)
        | BackgroundLayer::Radial(_)
        | BackgroundLayer::Conic(_) => {}
    }
}

/// Paints a resolved bitmap or vector URL within the background bounds.
pub(super) fn paint_background_url(
    paint: &mut DisplayListBuilder,
    url: &str,
    style: Option<&ComputedStyle>,
    layer_index: usize,
    positioning: Rect,
    bounds: Rect,
    images: &HashMap<Box<str>, SceneImage>,
    vectors: &HashMap<Box<str>, SceneVector>,
) {
    if let Some(image) = images.get(url) {
        let (width, height) = image.logical_size();
        let (tile, repeat) = background_image_geometry(
            style,
            layer_index,
            Some(Size::new(width, height)),
            positioning,
        );
        let (x, y) = repeat.axes();
        if x == BackgroundRepeatAxis::Space || y == BackgroundRepeatAxis::Space {
            for tile in background_tiles(tile, positioning, bounds, repeat) {
                paint.draw_image(image.clone(), tile, bounds, false, false);
            }
        } else {
            paint.draw_image(
                image.clone(),
                tile,
                bounds,
                repeat.repeat_x(),
                repeat.repeat_y(),
            );
        }
    } else if let Some(vector) = vectors.get(url) {
        paint_vector_background(paint, style, layer_index, vector, positioning, bounds);
    }
}

/// Paints visible vector tiles with the requested CSS sizing, positioning and repeat. Requests exceeding 8192 visible tiles are skipped to bound frame work.
fn paint_vector_background(
    paint: &mut DisplayListBuilder,
    style: Option<&ComputedStyle>,
    layer_index: usize,
    vector: &SceneVector,
    positioning: Rect,
    bounds: Rect,
) {
    let intrinsic = vector.size();
    if vector.is_empty()
        || !intrinsic.width.is_finite()
        || !intrinsic.height.is_finite()
        || intrinsic.width <= 0.0
        || intrinsic.height <= 0.0
    {
        return;
    }
    let (tile, repeat) =
        background_image_geometry(style, layer_index, Some(intrinsic), positioning);
    let mut tiles = background_tiles(tile, positioning, bounds, repeat).peekable();
    if tiles.peek().is_none() {
        return;
    }
    let shapes = vector.resolved(scene_color_from(current_color_of(style)));
    paint.push_clip(Clip::rect(bounds));
    for tile in tiles {
        paint.push_clip(Clip::rect(tile));
        emit_vector_tile(paint, shapes.iter().cloned(), intrinsic, tile);
        paint.pop_clip();
    }
    paint.pop_clip();
}

fn background_tiles(
    tile: Rect,
    positioning: Rect,
    bounds: Rect,
    repeat: BackgroundRepeat,
) -> impl Iterator<Item = Rect> {
    let (repeat_x, repeat_y) = repeat.axes();
    let (x, step_x, cols) = background_tile_span(
        repeat_x,
        (bounds.origin.x, bounds.right()),
        (positioning.origin.x, positioning.size.width),
        tile.origin.x,
        tile.size.width,
    );
    let (y, step_y, rows) = background_tile_span(
        repeat_y,
        (bounds.origin.y, bounds.bottom()),
        (positioning.origin.y, positioning.size.height),
        tile.origin.y,
        tile.size.height,
    );
    const MAX_TILES: usize = 8192;
    let rows = if cols.saturating_mul(rows) <= MAX_TILES {
        rows
    } else {
        0
    };
    (0..rows).flat_map(move |row| {
        (0..cols).map(move |col| {
            Rect::new(
                Point::new(x + col as f32 * step_x, y + row as f32 * step_y),
                tile.size,
            )
        })
    })
}

/// Returns the origin and count of intersecting background tiles along one axis.
fn background_tile_span(
    repeat: BackgroundRepeatAxis,
    (start, end): (f32, f32),
    (position_start, position_extent): (f32, f32),
    origin: f32,
    extent: f32,
) -> (f32, f32, usize) {
    if !start.is_finite()
        || !end.is_finite()
        || !origin.is_finite()
        || !extent.is_finite()
        || start >= end
        || extent <= 0.0
    {
        return (origin, extent, 0);
    }
    let (anchor, step) = match repeat {
        BackgroundRepeatAxis::Space => {
            let count = (position_extent / extent).floor() as usize;
            if count < 2 {
                return (
                    origin,
                    extent,
                    usize::from(origin < end && origin + extent > start),
                );
            }
            (
                position_start,
                (position_extent - extent) / (count - 1) as f32,
            )
        }
        BackgroundRepeatAxis::NoRepeat => {
            return (
                origin,
                extent,
                usize::from(origin < end && origin + extent > start),
            );
        }
        BackgroundRepeatAxis::Repeat | BackgroundRepeatAxis::Round => (origin, extent),
    };
    let first = start - (start - anchor).rem_euclid(step);
    (first, step, ((end - first) / step).ceil() as usize)
}

#[doc = include_str!("../../docs/api/orchestration/background_paint.doc/background_image_url_of.md")]
pub(super) fn background_image_url_of(style: Option<&ComputedStyle>) -> Option<&str> {
    style
        .and_then(|style| style.get(PropertyId::BackgroundImage))
        .and_then(ComputedValue::as_background_image_url)
}

/// Reads a node's computed `background-size`, defaulting to the CSS initial
/// `auto` when absent (an unauthored / unresolved property).
fn background_size_of(style: Option<&ComputedStyle>, layer_index: usize) -> BackgroundSize {
    match style
        .and_then(|style| style.get(PropertyId::BackgroundSize))
        .map(ComputedValue::value)
    {
        Some(PropertyValue::BackgroundSize(value)) => *value,
        Some(PropertyValue::BackgroundSizes(values)) if !values.is_empty() => {
            values[layer_index % values.len()]
        }
        _ => BackgroundSize::Auto,
    }
}

/// Reads a node's computed `background-position`, defaulting to the CSS initial
/// `0% 0%` (top-left) when absent.
fn background_position_of(style: Option<&ComputedStyle>, layer_index: usize) -> BackgroundPosition {
    match style
        .and_then(|style| style.get(PropertyId::BackgroundPosition))
        .map(ComputedValue::value)
    {
        Some(PropertyValue::BackgroundPosition(value)) => *value,
        Some(PropertyValue::BackgroundPositions(values)) if !values.is_empty() => {
            values[layer_index % values.len()]
        }
        _ => BackgroundPosition::default(),
    }
}

/// Reads a node's computed `background-repeat`, defaulting to the CSS initial
/// `repeat` (tile both axes) when absent.
fn background_repeat_of(style: Option<&ComputedStyle>, layer_index: usize) -> BackgroundRepeat {
    match style
        .and_then(|style| style.get(PropertyId::BackgroundRepeat))
        .map(ComputedValue::value)
    {
        Some(PropertyValue::BackgroundRepeat(repeat)) => *repeat,
        Some(PropertyValue::BackgroundRepeats(repeats)) if !repeats.is_empty() => {
            repeats[layer_index % repeats.len()]
        }
        _ => BackgroundRepeat::Repeat,
    }
}

#[doc = include_str!("../../docs/api/orchestration/background_paint.doc/background_image_geometry.md")]
fn background_image_geometry(
    style: Option<&ComputedStyle>,
    layer_index: usize,
    intrinsic: Option<Size>,
    region: Rect,
) -> (Rect, BackgroundRepeat) {
    let bw = region.size.width;
    let bh = region.size.height;
    let size = background_size_of(style, layer_index);
    let position = background_position_of(style, layer_index);
    let repeat = background_repeat_of(style, layer_index);
    let (mut tw, mut th) = match (intrinsic, size) {
        (Some(image), size) => {
            resolve_background_tile_size(size, image.width, image.height, bw, bh)
        }
        (None, BackgroundSize::Explicit { width, height }) => (
            resolve_background_size_length(width, bw).unwrap_or(bw),
            resolve_background_size_length(height, bh).unwrap_or(bh),
        ),
        (None, _) => (bw, bh),
    };
    let (x, y) = repeat.axes();
    let round_x = x == BackgroundRepeatAxis::Round;
    let round_y = y == BackgroundRepeatAxis::Round;
    let (auto_x, auto_y) = match size {
        BackgroundSize::Auto => (true, true),
        BackgroundSize::Explicit { width, height } => (width.is_auto(), height.is_auto()),
        _ => (false, false),
    };
    if round_x && tw > 0.0 {
        let rounded = bw / (bw / tw).round().max(1.0);
        if !round_y && auto_y && intrinsic.is_some() {
            th *= rounded / tw;
        }
        tw = rounded;
    }
    if round_y && th > 0.0 {
        let rounded = bh / (bh / th).round().max(1.0);
        if !round_x && auto_x && intrinsic.is_some() {
            tw *= rounded / th;
        }
        th = rounded;
    }
    let [from_right, from_bottom] = position.end_edges();
    let ox = region.origin.x + resolve_background_position(position.x(), bw, tw, from_right);
    let oy = region.origin.y + resolve_background_position(position.y(), bh, th, from_bottom);
    (Rect::new(Point::new(ox, oy), Size::new(tw, th)), repeat)
}

#[doc = include_str!("../../docs/api/orchestration/background_paint.doc/resolve_background_tile_size.md")]
pub(super) fn resolve_background_tile_size(
    size: BackgroundSize,
    iw: f32,
    ih: f32,
    bw: f32,
    bh: f32,
) -> (f32, f32) {
    // A degenerate intrinsic size cannot preserve an aspect ratio; stretch to fill.
    if iw <= 0.0 || ih <= 0.0 {
        return (bw, bh);
    }
    match size {
        BackgroundSize::Auto => (iw, ih),
        BackgroundSize::Cover => {
            let scale = (bw / iw).max(bh / ih);
            (iw * scale, ih * scale)
        }
        BackgroundSize::Contain => {
            let scale = (bw / iw).min(bh / ih);
            (iw * scale, ih * scale)
        }
        BackgroundSize::Explicit { width, height } => {
            let w = resolve_background_size_length(width, bw);
            let h = resolve_background_size_length(height, bh);
            match (w, h) {
                (Some(w), Some(h)) => (w, h),
                // A single `auto` axis preserves the intrinsic aspect ratio.
                (Some(w), None) => (w, w * ih / iw),
                (None, Some(h)) => (h * iw / ih, h),
                (None, None) => (iw, ih),
            }
        }
    }
}

/// Resolves one explicit `background-size` component against a box extent: an
/// absolute `px` directly, a `Percent` as a fraction of the box extent. `auto`
/// (and any context-dependent unit, a documented approximation) yields [`None`]
/// so the axis is derived from the other.
fn resolve_background_size_length(length: Length, basis: f32) -> Option<f32> {
    match length {
        Length::Px(px) => Some(px.0),
        Length::Percent(pct) => Some(pct / 100.0 * basis),
        _ => None,
    }
}

/// Resolves a positioned image's offset from the near edge. A percentage uses
/// the difference between the positioning-area extent and tile extent. When
/// the authored offset starts at the far edge, subtract it from that difference.
pub(super) fn resolve_background_position(
    length: Length,
    box_extent: f32,
    tile_extent: f32,
    from_end: bool,
) -> f32 {
    let available = box_extent - tile_extent;
    let offset = match length {
        Length::Px(px) => px.0,
        Length::Percent(pct) => available * pct / 100.0,
        _ => 0.0,
    };
    if from_end { available - offset } else { offset }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_with_an_auto_axis_preserves_intrinsic_ratio() {
        use BackgroundRepeatAxis::{NoRepeat, Round};
        let mut style = ComputedStyle::new();
        style.set(
            PropertyId::BackgroundRepeat,
            ComputedValue::from_concrete(PropertyValue::BackgroundRepeat(BackgroundRepeat::Axes(
                Round, NoRepeat,
            )))
            .unwrap(),
        );
        let bounds = Rect::new(Point::new(0.0, 0.0), Size::new(24.0, 24.0));
        let (tile, _) =
            background_image_geometry(Some(&style), 0, Some(Size::new(9.0, 4.5)), bounds);
        assert_eq!(tile.size, Size::new(8.0, 4.0));
    }

    #[test]
    fn space_with_room_for_only_one_tile_keeps_its_authored_position() {
        assert_eq!(
            background_tile_span(
                BackgroundRepeatAxis::Space,
                (0.0, 24.0),
                (0.0, 24.0),
                4.0,
                16.0
            ),
            (4.0, 16.0, 1)
        );
    }
}
