#![doc = include_str!("paint_transform.doc/_module.md")]

use obscur_geometry::{AffineTransform, Rect};
use obscur_identity::NodeId;
use obscur_style::computed::ComputedValue;
use obscur_style::property::{Length, PropertyId, Transform, TransformOrigin};

use super::ComputedStyles;

#[doc = include_str!("paint_transform.doc/resolved_transform.md")]
pub(crate) fn resolved_transform(
    computed: &ComputedStyles,
    node: NodeId,
    bounds: Rect,
) -> Option<AffineTransform> {
    let transform = transform_of(computed, node)?;
    if transform.is_identity() {
        return None;
    }
    let origin = transform_origin_of(computed, node);
    let ox = bounds.origin.x + resolve_origin_length(origin.x(), bounds.size.width);
    let oy = bounds.origin.y + resolve_origin_length(origin.y(), bounds.size.height);
    let matrix = AffineTransform {
        coeffs: transform.matrix(),
    };
    Some(
        AffineTransform::translate(ox, oy)
            .multiply(matrix)
            .multiply(AffineTransform::translate(-ox, -oy)),
    )
}

#[doc = include_str!("paint_transform.doc/transform_of.md")]
fn transform_of(computed: &ComputedStyles, node: NodeId) -> Option<Transform> {
    computed
        .get(&node)
        .and_then(|style| style.get(PropertyId::Transform))
        .and_then(ComputedValue::as_transform)
}

#[doc = include_str!("paint_transform.doc/transform_origin_of.md")]
fn transform_origin_of(computed: &ComputedStyles, node: NodeId) -> TransformOrigin {
    computed
        .get(&node)
        .and_then(|style| style.get(PropertyId::TransformOrigin))
        .and_then(ComputedValue::as_transform_origin)
        .unwrap_or(TransformOrigin::CENTER)
}

#[doc = include_str!("paint_transform.doc/resolve_origin_length.md")]
fn resolve_origin_length(length: Length, basis: f32) -> f32 {
    match length {
        Length::Px(px) => px.0,
        Length::Percent(pct) => pct / 100.0 * basis,
        _ => basis / 2.0,
    }
}
