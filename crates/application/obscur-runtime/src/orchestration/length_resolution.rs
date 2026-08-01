#![doc = include_str!("length_resolution.doc/_module.md")]

use obscur_geometry::{Insets, Size};
use obscur_identity::NodeId;
use obscur_layout::model::{Dimension, Margin, MarginEdges};
use obscur_style::computed::{ComputedStyle, ComputedValue};
use obscur_style::property::{Length, LengthContext, PropertyId, PropertyValue, layout_keyword};
use obscur_text::shaping::ParagraphShaper;

use super::text_leaf_measure::measure_ch_width;
use super::{ComputedStyles, attributes_for, families_for};

#[doc = include_str!("length_resolution.doc/DEFAULT_FONT_SIZE.md")]
const DEFAULT_FONT_SIZE: f32 = 16.0;

#[doc = include_str!("length_resolution.doc/ch_width_for.md")]
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

#[doc = include_str!("length_resolution.doc/root_font_size.md")]
#[must_use]
pub fn root_font_size(computed: &ComputedStyles, root: Option<NodeId>) -> f32 {
    root.and_then(|node| computed.get(&node))
        .map_or(DEFAULT_FONT_SIZE, font_size_px)
}

#[doc = include_str!("length_resolution.doc/length_context.md")]
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

#[doc = include_str!("length_resolution.doc/font_size_px.md")]
pub(super) fn font_size_px(computed: &ComputedStyle) -> f32 {
    computed
        .get(PropertyId::FontSize)
        .and_then(ComputedValue::as_length)
        .and_then(Length::as_px)
        .map_or(DEFAULT_FONT_SIZE, |px| px.0)
}

#[doc = include_str!("length_resolution.doc/dimension_of.md")]
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
        Some(PropertyValue::Calc(expr)) => expr.resolve(ctx).map_or(Dimension::Auto, Dimension::Px),
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

#[doc = include_str!("length_resolution.doc/length_px_of.md")]
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

#[doc = include_str!("length_resolution.doc/margin_edges_of.md")]
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

#[doc = include_str!("length_resolution.doc/margin_of.md")]
fn margin_of(computed: &ComputedStyle, property: PropertyId, ctx: &LengthContext) -> Margin {
    match computed.get(property).map(ComputedValue::value) {
        Some(&PropertyValue::Length(Length::Auto)) => Margin::Auto,
        Some(&PropertyValue::Length(Length::Percent(percent))) => Margin::Percent(percent),
        _ => Margin::Px(length_px_of(computed, property, ctx)),
    }
}

#[doc = include_str!("length_resolution.doc/insets_of.md")]
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
        let mut computed = ComputedStyles::new();
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
