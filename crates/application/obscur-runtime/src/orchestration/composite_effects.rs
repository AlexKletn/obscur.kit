#![doc = include_str!("composite_effects.doc/_module.md")]

use obscur_identity::NodeId;
use obscur_scene::paint::BlendMode;
use obscur_style::computed::{ComputedStyle, ComputedValue};
use obscur_style::property::{FilterOp, PropertyId, PropertyValue, layout_keyword};

use super::ComputedStyles;
use super::filter_color_matrix::color_matrix_of_ops;

#[doc = include_str!("composite_effects.doc/filter_ops_of.md")]
pub(super) fn filter_ops_of(computed: &ComputedStyles, node: NodeId) -> Option<&[FilterOp]> {
    computed
        .get(&node)
        .and_then(|style| style.get(PropertyId::Filter))
        .and_then(ComputedValue::as_filter_ops)
}

#[doc = include_str!("composite_effects.doc/filter_blur_of.md")]
pub(super) fn filter_blur_of(computed: &ComputedStyles, node: NodeId) -> Option<f32> {
    let radius = filter_ops_of(computed, node)?
        .iter()
        .filter_map(FilterOp::as_blur)
        .fold(0.0_f32, f32::max);
    (radius > 0.0).then_some(radius)
}

#[doc = include_str!("composite_effects.doc/filter_color_matrix_of.md")]
pub(super) fn filter_color_matrix_of(computed: &ComputedStyles, node: NodeId) -> Option<[f32; 20]> {
    color_matrix_of_ops(filter_ops_of(computed, node)?)
}

#[doc = include_str!("composite_effects.doc/backdrop_blur_of.md")]
pub(super) fn backdrop_blur_of(computed: &ComputedStyles, node: NodeId) -> Option<f32> {
    let radius = computed
        .get(&node)
        .and_then(|style| style.get(PropertyId::BackdropFilter))
        .and_then(ComputedValue::as_filter_ops)?
        .iter()
        .filter_map(FilterOp::as_blur)
        .fold(0.0_f32, f32::max);
    (radius > 0.0).then_some(radius)
}

#[doc = include_str!("composite_effects.doc/opacity_of.md")]
pub(super) fn opacity_of(computed: &ComputedStyles, node: NodeId) -> Option<f32> {
    let alpha = computed.get(&node).and_then(ComputedStyle::opacity)?;
    let alpha = alpha.clamp(0.0, 1.0);
    (alpha < 1.0).then_some(alpha)
}

#[doc = include_str!("composite_effects.doc/blend_mode_of.md")]
pub(super) fn blend_mode_of(computed: &ComputedStyles, node: NodeId) -> Option<BlendMode> {
    let sym = match computed
        .get(&node)
        .and_then(|style| style.get(PropertyId::MixBlendMode))
        .map(ComputedValue::value)
    {
        Some(&PropertyValue::Keyword(sym)) => sym,
        _ => return None,
    };
    let mode = if sym == layout_keyword::MIX_BLEND_MODE_MULTIPLY {
        BlendMode::Multiply
    } else if sym == layout_keyword::MIX_BLEND_MODE_SCREEN {
        BlendMode::Screen
    } else if sym == layout_keyword::MIX_BLEND_MODE_OVERLAY {
        BlendMode::Overlay
    } else if sym == layout_keyword::MIX_BLEND_MODE_DARKEN {
        BlendMode::Darken
    } else if sym == layout_keyword::MIX_BLEND_MODE_LIGHTEN {
        BlendMode::Lighten
    } else if sym == layout_keyword::MIX_BLEND_MODE_COLOR_DODGE {
        BlendMode::ColorDodge
    } else if sym == layout_keyword::MIX_BLEND_MODE_COLOR_BURN {
        BlendMode::ColorBurn
    } else if sym == layout_keyword::MIX_BLEND_MODE_HARD_LIGHT {
        BlendMode::HardLight
    } else if sym == layout_keyword::MIX_BLEND_MODE_SOFT_LIGHT {
        BlendMode::SoftLight
    } else if sym == layout_keyword::MIX_BLEND_MODE_DIFFERENCE {
        BlendMode::Difference
    } else if sym == layout_keyword::MIX_BLEND_MODE_EXCLUSION {
        BlendMode::Exclusion
    } else {
        // `normal` or an unmodelled keyword: no blend layer.
        return None;
    };
    Some(mode)
}
