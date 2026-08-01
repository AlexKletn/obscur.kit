#![doc = include_str!("background_paint.doc/_module.md")]

use std::collections::HashMap;

use obscur_geometry::{CornerRadii, Point, Rect, Size};
use obscur_identity::NodeId;
use obscur_scene::command::DisplayListBuilder;
use obscur_scene::paint::{Brush, SceneImage};
use obscur_scene::path::Path;
use obscur_style::computed::ComputedValue;
use obscur_style::property::{
    BackgroundLayer, BackgroundPosition, BackgroundRepeat, BackgroundSize, Length, PropertyId,
    StyleColor,
};

use super::ComputedStyles;
use super::gradient_brush::{conic_gradient_brush, linear_gradient_brush, radial_gradient_brush};

#[doc = include_str!("background_paint.doc/background_color_of.md")]
pub(super) fn background_color_of(computed: &ComputedStyles, node: NodeId) -> Option<StyleColor> {
    computed
        .get(&node)
        .and_then(|style| style.get(PropertyId::BackgroundColor))
        .and_then(ComputedValue::as_color)
}

#[doc = include_str!("background_paint.doc/background_gradient_brush.md")]
pub(super) fn background_gradient_brush(
    computed: &ComputedStyles,
    node: NodeId,
    bounds: Rect,
) -> Option<Brush> {
    let value = computed
        .get(&node)
        .and_then(|style| style.get(PropertyId::BackgroundImage))?;
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

#[doc = include_str!("background_paint.doc/background_layers_of.md")]
pub(super) fn background_layers_of(
    computed: &ComputedStyles,
    node: NodeId,
) -> Option<&[BackgroundLayer]> {
    computed
        .get(&node)
        .and_then(|style| style.get(PropertyId::BackgroundImage))
        .and_then(ComputedValue::as_background_layers)
}

#[doc = include_str!("background_paint.doc/fill_background_box.md")]
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

#[doc = include_str!("background_paint.doc/paint_background_layer.md")]
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_background_layer(
    paint: &mut DisplayListBuilder,
    layer: &BackgroundLayer,
    computed: &ComputedStyles,
    node: NodeId,
    bounds: Rect,
    radii: CornerRadii,
    has_radius: bool,
    images: &HashMap<Box<str>, SceneImage>,
) {
    match layer {
        BackgroundLayer::Linear(gradient) if !gradient.stops().is_empty() => {
            let brush = Brush::Linear(linear_gradient_brush(gradient, bounds));
            fill_background_box(paint, bounds, radii, has_radius, brush);
        }
        BackgroundLayer::Radial(gradient) if !gradient.stops().is_empty() => {
            let brush = Brush::Radial(radial_gradient_brush(gradient, bounds));
            fill_background_box(paint, bounds, radii, has_radius, brush);
        }
        BackgroundLayer::Conic(gradient) if !gradient.stops().is_empty() => {
            let brush = Brush::Conic(conic_gradient_brush(gradient, bounds));
            fill_background_box(paint, bounds, radii, has_radius, brush);
        }
        BackgroundLayer::Url(url) => {
            if let Some(image) = images.get(&**url) {
                let (tile, repeat_x, repeat_y) =
                    background_image_geometry(computed, node, image, bounds);
                paint.draw_image(image.clone(), tile, bounds, repeat_x, repeat_y);
            }
        }
        // A stop-less gradient paints nothing.
        BackgroundLayer::Linear(_) | BackgroundLayer::Radial(_) | BackgroundLayer::Conic(_) => {}
    }
}

#[doc = include_str!("background_paint.doc/background_image_url_of.md")]
pub(super) fn background_image_url_of(computed: &ComputedStyles, node: NodeId) -> Option<&str> {
    computed
        .get(&node)
        .and_then(|style| style.get(PropertyId::BackgroundImage))
        .and_then(ComputedValue::as_background_image_url)
}

#[doc = include_str!("background_paint.doc/background_size_of.md")]
fn background_size_of(computed: &ComputedStyles, node: NodeId) -> BackgroundSize {
    computed
        .get(&node)
        .and_then(|style| style.get(PropertyId::BackgroundSize))
        .and_then(ComputedValue::as_background_size)
        .unwrap_or(BackgroundSize::Auto)
}

#[doc = include_str!("background_paint.doc/background_position_of.md")]
fn background_position_of(computed: &ComputedStyles, node: NodeId) -> BackgroundPosition {
    computed
        .get(&node)
        .and_then(|style| style.get(PropertyId::BackgroundPosition))
        .and_then(ComputedValue::as_background_position)
        .unwrap_or_default()
}

#[doc = include_str!("background_paint.doc/background_repeat_of.md")]
fn background_repeat_of(computed: &ComputedStyles, node: NodeId) -> BackgroundRepeat {
    computed
        .get(&node)
        .and_then(|style| style.get(PropertyId::BackgroundRepeat))
        .and_then(ComputedValue::as_background_repeat)
        .unwrap_or(BackgroundRepeat::Repeat)
}

#[doc = include_str!("background_paint.doc/background_image_geometry.md")]
pub(super) fn background_image_geometry(
    computed: &ComputedStyles,
    node: NodeId,
    image: &SceneImage,
    region: Rect,
) -> (Rect, bool, bool) {
    // The *logical* size, not the raw pixel count: a `@2x` DPI variant carries
    // twice the pixels of the box it stands in for, and tiling it at its pixel
    // count would draw it at double size (spec §5.16).
    let (iw, ih) = image.logical_size();
    let bw = region.size.width;
    let bh = region.size.height;
    let size = background_size_of(computed, node);
    let position = background_position_of(computed, node);
    let repeat = background_repeat_of(computed, node);
    let (tw, th) = resolve_background_tile_size(size, iw, ih, bw, bh);
    let ox = region.origin.x + resolve_background_position(position.x(), bw, tw);
    let oy = region.origin.y + resolve_background_position(position.y(), bh, th);
    (
        Rect::new(Point::new(ox, oy), Size::new(tw, th)),
        repeat.repeat_x(),
        repeat.repeat_y(),
    )
}

#[doc = include_str!("background_paint.doc/resolve_background_tile_size.md")]
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

#[doc = include_str!("background_paint.doc/resolve_background_size_length.md")]
fn resolve_background_size_length(length: Length, basis: f32) -> Option<f32> {
    match length {
        Length::Px(px) => Some(px.0),
        Length::Percent(pct) => Some(pct / 100.0 * basis),
        _ => None,
    }
}

#[doc = include_str!("background_paint.doc/resolve_background_position.md")]
pub(super) fn resolve_background_position(
    length: Length,
    box_extent: f32,
    tile_extent: f32,
) -> f32 {
    match length {
        Length::Px(px) => px.0,
        Length::Percent(pct) => (box_extent - tile_extent) * pct / 100.0,
        _ => 0.0,
    }
}
