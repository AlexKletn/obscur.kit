//! Placing a replaced element's own content — a bitmap `src` or a vector
//! document — inside its box by `object-fit` / `object-position` (CSS Images 3
//! §5, spec §5.17).

use obscur_geometry::{AffineTransform, Insets, Point, Rect, Size};
use obscur_identity::NodeId;
use obscur_layout::model::LayoutBox;
use obscur_scene::clip::Clip;
use obscur_scene::command::DisplayListBuilder;
use obscur_scene::paint::{Paint, SceneImage, SceneVector};
use obscur_scene::path::Path;
use obscur_style::computed::{ComputedStyle, ComputedValue};
use obscur_style::property::{BackgroundPosition, Length, ObjectFit, PropertyId};
use obscur_tree::tree::RetainedTree;

use super::background_paint::resolve_background_position;
use super::paint_color::{current_color_of, scene_color_from};

/// Returns the content box used to position replaced image content.
pub(super) fn replaced_region(style: Option<&ComputedStyle>, layout_box: LayoutBox) -> Rect {
    if let Some(content) = layout_box.used_content_rect() {
        return content;
    }
    let px = |property| {
        style
            .and_then(|style| style.get(property))
            .and_then(ComputedValue::as_length)
            .and_then(Length::as_px)
            .map_or(0.0, |length| length.0.max(0.0))
    };
    let [top, right, bottom, left] = super::border_paint::border_widths_of(style);
    layout_box.content_rect(
        Insets {
            top: px(PropertyId::PaddingTop),
            right: px(PropertyId::PaddingRight),
            bottom: px(PropertyId::PaddingBottom),
            left: px(PropertyId::PaddingLeft),
        },
        Insets {
            top,
            right,
            bottom,
            left,
        },
    )
}

#[doc = include_str!("../../docs/api/orchestration/replaced_content.doc/emit_vector.md")]
pub(super) fn emit_vector(
    paint: &mut DisplayListBuilder,
    style: Option<&ComputedStyle>,
    vector: &SceneVector,
    region: Rect,
) {
    let intrinsic = vector.size();
    if intrinsic.width <= 0.0 || intrinsic.height <= 0.0 {
        return;
    }
    let tile = object_fit_tile_sized(style, intrinsic, region);
    let clips = tile.origin.x < region.origin.x
        || tile.origin.y < region.origin.y
        || tile.right() > region.right()
        || tile.bottom() > region.bottom();
    if clips {
        paint.push_clip(Clip::rect(region));
    }
    let shapes = vector.resolved(scene_color_from(current_color_of(style)));
    emit_vector_tile(paint, shapes, intrinsic, tile);
    if clips {
        paint.pop_clip();
    }
}

/// Maps resolved vector outlines from document coordinates into a destination tile.
pub(super) fn emit_vector_tile(
    paint: &mut DisplayListBuilder,
    shapes: impl IntoIterator<Item = (Path, Paint)>,
    intrinsic: Size,
    tile: Rect,
) {
    paint.push_transform(
        AffineTransform::translate(tile.origin.x, tile.origin.y).multiply(AffineTransform::scale(
            tile.size.width / intrinsic.width,
            tile.size.height / intrinsic.height,
        )),
    );
    for (path, shape_paint) in shapes {
        paint.draw(path, shape_paint);
    }
    paint.pop_transform();
}

#[doc = include_str!("../../docs/api/orchestration/replaced_content.doc/element_image_of.md")]
pub(super) fn element_image_of(tree: &RetainedTree, node: NodeId) -> Option<&str> {
    tree.get(node).and_then(|n| n.attributes().image_source())
}

/// Reads a node's computed `object-fit`, defaulting to the CSS initial `fill`
/// (stretch to the box) when absent.
fn object_fit_of(style: Option<&ComputedStyle>) -> ObjectFit {
    style
        .and_then(|style| style.get(PropertyId::ObjectFit))
        .and_then(ComputedValue::as_object_fit)
        .unwrap_or(ObjectFit::Fill)
}

/// Reads a node's computed `object-position`, defaulting to the CSS initial
/// `50% 50%` (centered) when absent — NOT `background-position`'s `0% 0%`. Placed
/// by the same [`BackgroundPosition`] value type since it is the same
/// `<position>` grammar (CSS Images 3 §5.2).
fn object_position_of(style: Option<&ComputedStyle>) -> BackgroundPosition {
    style
        .and_then(|style| style.get(PropertyId::ObjectPosition))
        .and_then(ComputedValue::as_background_position)
        .unwrap_or_else(|| BackgroundPosition::new(Length::Percent(50.0), Length::Percent(50.0)))
}

#[doc = include_str!("../../docs/api/orchestration/replaced_content.doc/object_fit_tile.md")]
pub(super) fn object_fit_tile(
    style: Option<&ComputedStyle>,
    image: &SceneImage,
    region: Rect,
) -> Rect {
    let (iw, ih) = image.logical_size();
    object_fit_tile_sized(style, Size::new(iw, ih), region)
}

/// [`object_fit_tile`] against a bare intrinsic size, so a replaced element whose
/// content is not a bitmap — a vector document — places identically to an image
/// rather than through a parallel rule.
fn object_fit_tile_sized(style: Option<&ComputedStyle>, intrinsic: Size, region: Rect) -> Rect {
    let fit = object_fit_of(style);
    let (iw, ih) = (intrinsic.width, intrinsic.height);
    let bw = region.size.width;
    let bh = region.size.height;
    let (tw, th) = resolve_object_fit_size(fit, iw, ih, bw, bh);
    // Place the tile per `object-position` (initial `50% 50%` = centered), by the
    // same CSS position formula as `background-position`: a `%` aligns the tile's
    // pct point to the box's pct point, so `(box − tile) × pct` per axis.
    let position = object_position_of(style);
    let [from_right, from_bottom] = position.end_edges();
    let ox = region.origin.x + resolve_background_position(position.x(), bw, tw, from_right);
    let oy = region.origin.y + resolve_background_position(position.y(), bh, th, from_bottom);
    Rect::new(Point::new(ox, oy), Size::new(tw, th))
}

#[doc = include_str!("../../docs/api/orchestration/replaced_content.doc/resolve_object_fit_size.md")]
fn resolve_object_fit_size(fit: ObjectFit, iw: f32, ih: f32, bw: f32, bh: f32) -> (f32, f32) {
    if iw <= 0.0 || ih <= 0.0 {
        return (bw, bh);
    }
    match fit {
        ObjectFit::Fill => (bw, bh),
        ObjectFit::Contain => {
            let scale = (bw / iw).min(bh / ih);
            (iw * scale, ih * scale)
        }
        ObjectFit::Cover => {
            let scale = (bw / iw).max(bh / ih);
            (iw * scale, ih * scale)
        }
        ObjectFit::None => (iw, ih),
        ObjectFit::ScaleDown => {
            // min(none, contain): the contain scale, but never larger than 1.
            let scale = (bw / iw).min(bh / ih).min(1.0);
            (iw * scale, ih * scale)
        }
    }
}
