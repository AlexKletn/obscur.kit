#![doc = include_str!("../../docs/api/orchestration/length_resolution.doc/_module.md")]

use obscur_geometry::{Insets, Size};
use obscur_identity::NodeId;
use obscur_layout::model::{Dimension, Margin, MarginEdges, TrackLimit};
use obscur_style::computed::{ComputedStyle, ComputedValue};
use obscur_style::property::{
    Length, LengthContext, LengthExpr, PropertyId, PropertyValue, layout_keyword,
};
use obscur_text::shaping::ParagraphShaper;

use super::text_leaf_measure::measure_ch_width;
use super::{ComputedStyles, attributes_for, families_for};

/// The initial `font-size` (spec / CSS default), used as the `em`/`rem` base
/// when a node (or the document root) has no computed `font-size`.
const DEFAULT_FONT_SIZE: f32 = 16.0;

#[doc = include_str!("../../docs/api/orchestration/length_resolution.doc/ch_width_for.md")]
pub(super) fn ch_width_for(
    computed: &ComputedStyle,
    paragraph: Option<&ParagraphShaper>,
) -> Option<f32> {
    if !computed.uses_ch() {
        return None;
    }
    let paragraph = paragraph?;
    measure_ch_width(
        paragraph,
        &families_for(Some(computed)),
        &attributes_for(Some(computed)),
        font_size_px(computed),
    )
}

#[doc = include_str!("../../docs/api/orchestration/length_resolution.doc/root_font_size.md")]
#[must_use]
pub fn root_font_size(computed: &ComputedStyles, root: Option<NodeId>) -> f32 {
    root.and_then(|node| computed.get(&node))
        .map_or(DEFAULT_FONT_SIZE, font_size_px)
}

#[doc = include_str!("../../docs/api/orchestration/length_resolution.doc/length_context.md")]
pub(super) fn length_context(
    computed: &ComputedStyle,
    viewport: Size,
    root_font_size: f32,
    ch_width: Option<f32>,
) -> LengthContext {
    let ctx = LengthContext::new(
        font_size_px(computed),
        root_font_size,
        viewport.width,
        viewport.height,
    );
    match ch_width {
        Some(width) => ctx.with_ch_width(width),
        None => ctx,
    }
}

/// One node's computed `font-size` in logical pixels, defaulting to
/// [`DEFAULT_FONT_SIZE`] when absent or not an absolute pixel length.
pub(super) fn font_size_px(computed: &ComputedStyle) -> f32 {
    computed
        .get(PropertyId::FontSize)
        .and_then(ComputedValue::as_length)
        .and_then(Length::as_px)
        .map_or(DEFAULT_FONT_SIZE, |px| px.0)
}

/// Lowers a length-valued property to a layout [`Dimension`]. Fixed pixels
/// and resolved relative units become pixel lengths. Percentages and math
/// that depends on the containing size stay symbolic until layout.
pub(super) fn dimension_of(
    computed: &ComputedStyle,
    property: PropertyId,
    ctx: &LengthContext,
) -> Dimension {
    match computed.get(property).map(ComputedValue::value) {
        Some(&PropertyValue::Length(length)) => match length {
            Length::Px(px) => Dimension::Px(px.0),
            Length::Percent(percent) => Dimension::Percent(percent),
            Length::Auto => Dimension::Auto,
            relative => relative
                .resolve_px(ctx)
                .map_or(Dimension::Auto, Dimension::Px),
        },
        Some(PropertyValue::Calc(expr)) => match expr.affine_components(ctx) {
            Some((pixels, 0.0)) => Dimension::Px(
                if matches!(
                    property,
                    PropertyId::Top | PropertyId::Right | PropertyId::Bottom | PropertyId::Left
                ) {
                    pixels
                } else {
                    pixels.max(0.0)
                },
            ),
            Some((pixels, fraction)) => Dimension::Affine { pixels, fraction },
            None => Dimension::Math(std::sync::Arc::new(track_limit_math(expr, ctx))),
        },
        // The intrinsic sizing keywords carry no length: they name a question
        // layout answers by measuring content (CSS Sizing 3 §4.1).
        Some(&PropertyValue::Keyword(sym)) if sym == layout_keyword::SIZING_MIN_CONTENT => {
            Dimension::MinContent
        }
        Some(&PropertyValue::Keyword(sym)) if sym == layout_keyword::SIZING_MAX_CONTENT => {
            Dimension::MaxContent
        }
        Some(&PropertyValue::Keyword(sym)) if sym == layout_keyword::SIZING_FIT_CONTENT => {
            Dimension::FitContent
        }
        _ => Dimension::Auto,
    }
}

/// Resolves length units that do not require an arranged grid axis.
pub(super) fn track_limit_length(length: Length, ctx: &LengthContext) -> Dimension {
    match length {
        Length::Percent(percent) => Dimension::Percent(percent),
        length => length
            .resolve_px(ctx)
            .map_or(Dimension::Auto, Dimension::Px),
    }
}

/// Lowers a CSS math expression while retaining the grid-relative percentage leaves.
pub(super) fn track_limit_math(expr: &LengthExpr, ctx: &LengthContext) -> TrackLimit {
    let node = |term: &LengthExpr| Box::new(track_limit_math(term, ctx));
    match expr {
        LengthExpr::Leaf(length) => TrackLimit::Length(track_limit_length(*length, ctx)),
        LengthExpr::Sum(a, b) => TrackLimit::Sum(node(a), node(b)),
        LengthExpr::Diff(a, b) => TrackLimit::Difference(node(a), node(b)),
        LengthExpr::Product(value, scalar) => TrackLimit::Scale(node(value), *scalar),
        LengthExpr::Quotient(value, scalar) => TrackLimit::Quotient(node(value), *scalar),
        LengthExpr::Min(terms) => TrackLimit::Min(
            terms
                .iter()
                .map(|term| track_limit_math(term, ctx))
                .collect(),
        ),
        LengthExpr::Max(terms) => TrackLimit::Max(
            terms
                .iter()
                .map(|term| track_limit_math(term, ctx))
                .collect(),
        ),
        LengthExpr::Clamp(min, value, max) => TrackLimit::Clamp(node(min), node(value), node(max)),
    }
}

#[doc = include_str!("../../docs/api/orchestration/length_resolution.doc/length_px_of.md")]
pub(super) fn length_px_of(
    computed: &ComputedStyle,
    property: PropertyId,
    ctx: &LengthContext,
) -> f32 {
    match computed.get(property).map(ComputedValue::value) {
        Some(&PropertyValue::Length(length)) => match length {
            Length::Px(px) => px.0,
            Length::Percent(_) | Length::Auto => 0.0,
            relative => relative.resolve_px(ctx).unwrap_or(0.0),
        },
        Some(PropertyValue::Calc(expr)) => expr.resolve(ctx).unwrap_or(0.0),
        _ => 0.0,
    }
}

#[doc = include_str!("../../docs/api/orchestration/length_resolution.doc/margin_edges_of.md")]
pub(super) fn margin_edges_of(
    computed: &ComputedStyle,
    top: PropertyId,
    right: PropertyId,
    bottom: PropertyId,
    left: PropertyId,
    ctx: &LengthContext,
) -> MarginEdges {
    MarginEdges {
        top: margin_of(computed, top, ctx),
        right: margin_of(computed, right, ctx),
        bottom: margin_of(computed, bottom, ctx),
        left: margin_of(computed, left, ctx),
    }
}

/// Resolves one margin longhand into a [`Margin`]: an authored `auto` stays
/// [`Margin::Auto`] and an authored `%` stays [`Margin::Percent`], while every
/// other value folds to logical pixels exactly as [`length_px_of`] does.
fn margin_of(computed: &ComputedStyle, property: PropertyId, ctx: &LengthContext) -> Margin {
    match computed.get(property).map(ComputedValue::value) {
        Some(&PropertyValue::Length(Length::Auto)) => Margin::Auto,
        Some(&PropertyValue::Length(Length::Percent(percent))) => Margin::Percent(percent),
        Some(PropertyValue::Calc(expr)) if expr.uses_percent() => {
            Margin::Math(std::sync::Arc::new(track_limit_math(expr, ctx)))
        }
        _ => Margin::Px(length_px_of(computed, property, ctx)),
    }
}

/// Builds pixel [`Insets`] from the four edge longhands (top/right/bottom/left).
pub(super) fn insets_of(
    computed: &ComputedStyle,
    top: PropertyId,
    right: PropertyId,
    bottom: PropertyId,
    left: PropertyId,
    ctx: &LengthContext,
) -> Insets {
    Insets {
        top: length_px_of(computed, top, ctx),
        right: length_px_of(computed, right, ctx),
        bottom: length_px_of(computed, bottom, ctx),
        left: length_px_of(computed, left, ctx),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn root_font_size_reads_root_node_defaulting_to_sixteen() {
        let mut computed = ComputedStyles::default();
        let mut root = ComputedStyle::new();
        root.set(
            PropertyId::FontSize,
            ComputedValue::length(Length::px(24.0)),
        );
        computed.insert(NodeId::from_raw(1), root);
        assert_eq!(root_font_size(&computed, Some(NodeId::from_raw(1))), 24.0);
        // Absent root or missing font-size falls back to the initial 16px.
        assert_eq!(root_font_size(&computed, Some(NodeId::from_raw(9))), 16.0);
        assert_eq!(root_font_size(&computed, None), 16.0);
    }
}
