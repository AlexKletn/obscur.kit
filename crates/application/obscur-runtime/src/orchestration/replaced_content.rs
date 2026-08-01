#![doc = include_str!("replaced_content.doc/_module.md")]

use obscur_geometry::{AffineTransform, Point, Rect, Size};
use obscur_identity::NodeId;
use obscur_scene::command::DisplayListBuilder;
use obscur_scene::paint::{SceneImage, SceneVector};
use obscur_style::computed::ComputedValue;
use obscur_style::property::{BackgroundPosition, Length, ObjectFit, PropertyId};
use obscur_tree::tree::RetainedTree;

use super::ComputedStyles;
use super::background_paint::resolve_background_position;
use super::paint_color::{current_color_of, scene_color_from};

#[doc = include_str!("replaced_content.doc/emit_vector.md")]
pub(super) fn emit_vector(
    paint: &mut DisplayListBuilder,
    computed: &ComputedStyles,
    node: NodeId,
    vector: &SceneVector,
    region: Rect,
) {
    let intrinsic = vector.size();
    if intrinsic.width <= 0.0 || intrinsic.height <= 0.0 {
        return;
    }
    let tile = object_fit_tile_sized(computed, node, intrinsic, region);
    let (sx, sy) = (
        tile.size.width / intrinsic.width,
        tile.size.height / intrinsic.height,
    );

    // `currentColor` is the node's computed `color` — the same resolution
    // `border-*-color` already falls back to.
    let current = scene_color_from(current_color_of(computed, node));

    paint.push_transform(
        AffineTransform::translate(tile.origin.x, tile.origin.y)
            .multiply(AffineTransform::scale(sx, sy)),
    );
    for (path, shape_paint) in vector.resolved(current) {
        paint.draw(path, shape_paint);
    }
    paint.pop_transform();
}

#[doc = include_str!("replaced_content.doc/element_image_of.md")]
pub(super) fn element_image_of(tree: &RetainedTree, node: NodeId) -> Option<&str> {
    tree.get(node).and_then(|n| n.attributes().image_source())
}

#[doc = include_str!("replaced_content.doc/object_fit_of.md")]
fn object_fit_of(computed: &ComputedStyles, node: NodeId) -> ObjectFit {
    computed
        .get(&node)
        .and_then(|style| style.get(PropertyId::ObjectFit))
        .and_then(ComputedValue::as_object_fit)
        .unwrap_or(ObjectFit::Fill)
}

#[doc = include_str!("replaced_content.doc/object_position_of.md")]
fn object_position_of(computed: &ComputedStyles, node: NodeId) -> BackgroundPosition {
    computed
        .get(&node)
        .and_then(|style| style.get(PropertyId::ObjectPosition))
        .and_then(ComputedValue::as_background_position)
        .unwrap_or_else(|| BackgroundPosition::new(Length::Percent(50.0), Length::Percent(50.0)))
}

#[doc = include_str!("replaced_content.doc/object_fit_tile.md")]
pub(super) fn object_fit_tile(
    computed: &ComputedStyles,
    node: NodeId,
    image: &SceneImage,
    region: Rect,
) -> Rect {
    let (iw, ih) = image.logical_size();
    object_fit_tile_sized(computed, node, Size::new(iw, ih), region)
}

#[doc = include_str!("replaced_content.doc/object_fit_tile_sized.md")]
fn object_fit_tile_sized(
    computed: &ComputedStyles,
    node: NodeId,
    intrinsic: Size,
    region: Rect,
) -> Rect {
    let fit = object_fit_of(computed, node);
    let (iw, ih) = (intrinsic.width, intrinsic.height);
    let bw = region.size.width;
    let bh = region.size.height;
    let (tw, th) = resolve_object_fit_size(fit, iw, ih, bw, bh);
    // Place the tile per `object-position` (initial `50% 50%` = centered), by the
    // same CSS position formula as `background-position`: a `%` aligns the tile's
    // pct point to the box's pct point, so `(box − tile) × pct` per axis.
    let position = object_position_of(computed, node);
    let ox = region.origin.x + resolve_background_position(position.x(), bw, tw);
    let oy = region.origin.y + resolve_background_position(position.y(), bh, th);
    Rect::new(Point::new(ox, oy), Size::new(tw, th))
}

#[doc = include_str!("replaced_content.doc/resolve_object_fit_size.md")]
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
