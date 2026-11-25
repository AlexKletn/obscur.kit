//! A node's CSS 2D `transform`, resolved about its `transform-origin` into the
//! absolute-space affine that brackets its subtree paint (spec §5.2).

use obscur_geometry::{AffineTransform, Point, Rect};
use obscur_style::computed::{ComputedStyle, ComputedValue};
use obscur_style::property::{Length, PropertyId, PropertyValue, TransformOrigin};

/// Whether the transform value establishes a containing block and paint context.
pub(super) fn has_transform(style: &ComputedStyle) -> bool {
    style
        .get(PropertyId::Transform)
        .map(ComputedValue::value)
        .is_some_and(|value| match value {
            PropertyValue::Transform(transform) => transform.establishes_containing_block(),
            PropertyValue::TransformFunctions(_) => true,
            _ => false,
        })
}

#[doc = include_str!("../../docs/api/orchestration/paint_transform.doc/resolved_transform.md")]
pub(crate) fn resolved_transform(
    style: Option<&ComputedStyle>,
    bounds: Rect,
) -> Option<AffineTransform> {
    let matrix = transform_matrix(style, bounds)?;
    (!matrix.is_identity()).then(|| about_transform_origin(style, bounds, matrix))
}

/// Resolves the CSS matrix in the node’s transform-origin coordinate system.
pub(crate) fn transform_matrix(
    style: Option<&ComputedStyle>,
    bounds: Rect,
) -> Option<AffineTransform> {
    let matrix = match style?.get(PropertyId::Transform)?.value() {
        PropertyValue::Transform(transform) => AffineTransform {
            coeffs: transform.matrix(),
        },
        PropertyValue::TransformFunctions(functions) => {
            functions
                .iter()
                .try_fold(AffineTransform::IDENTITY, |matrix, function| {
                    let coeffs = function.resolve_in_box(bounds.size.width, bounds.size.height)?;
                    Some(matrix.multiply(AffineTransform { coeffs }))
                })?
        }
        _ => return None,
    };
    matrix
        .coeffs
        .iter()
        .all(|value| value.is_finite())
        .then_some(matrix)
}

/// Conjugates a local matrix around the arranged CSS transform origin.
pub(crate) fn about_transform_origin(
    style: Option<&ComputedStyle>,
    bounds: Rect,
    matrix: AffineTransform,
) -> AffineTransform {
    let Point { x: ox, y: oy } = transform_origin(style, bounds);
    AffineTransform::translate(ox, oy)
        .multiply(matrix)
        .multiply(AffineTransform::translate(-ox, -oy))
}

/// Resolves the transform origin against the arranged border box.
pub(crate) fn transform_origin(style: Option<&ComputedStyle>, bounds: Rect) -> Point {
    let origin = transform_origin_of(style);
    Point::new(
        bounds.origin.x + resolve_origin_length(origin.x(), bounds.size.width),
        bounds.origin.y + resolve_origin_length(origin.y(), bounds.size.height),
    )
}

/// Reads a node's computed `transform-origin`, defaulting to the box center
/// (`50% 50%`) when unauthored (the CSS initial).
fn transform_origin_of(style: Option<&ComputedStyle>) -> TransformOrigin {
    style
        .and_then(|style| style.get(PropertyId::TransformOrigin))
        .and_then(ComputedValue::as_transform_origin)
        .unwrap_or(TransformOrigin::CENTER)
}

/// Resolves one `transform-origin` component to a logical-pixel offset within a
/// box axis of length `basis`: an absolute `px` directly, a `Percent` as a
/// fraction of `basis`. Any other (context-dependent) length falls back to the
/// center of the axis (a documented approximation).
fn resolve_origin_length(length: Length, basis: f32) -> f32 {
    match length {
        Length::Px(px) => px.0,
        Length::Percent(pct) => pct / 100.0 * basis,
        _ => basis / 2.0,
    }
}
