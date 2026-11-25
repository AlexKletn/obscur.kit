//! The group-layer effects a node composites its whole subtree through:
//! `opacity`, `mix-blend-mode`, `filter` and `backdrop-filter` (spec §5.1,
//! compositing-1 §5, CSS Filter Effects §).

use obscur_scene::paint::BlendMode;
use obscur_style::computed::{ComputedStyle, ComputedValue};
use obscur_style::property::{FilterOp, PropertyId, PropertyValue, layout_keyword};

use super::filter_color_matrix::color_matrix_of_ops;

#[doc = include_str!("../../docs/api/orchestration/composite_effects.doc/filter_ops_of.md")]
pub(super) fn filter_ops_of(style: Option<&ComputedStyle>) -> Option<&[FilterOp]> {
    style
        .and_then(|style| style.get(PropertyId::Filter))
        .and_then(ComputedValue::as_filter_ops)
}

#[doc = include_str!("../../docs/api/orchestration/composite_effects.doc/filter_blur_of.md")]
pub(super) fn filter_blur_of(ops: Option<&[FilterOp]>) -> Option<f32> {
    let radius = ops?
        .iter()
        .filter_map(FilterOp::as_blur)
        .fold(0.0_f32, f32::max);
    (radius > 0.0).then_some(radius)
}

#[doc = include_str!("../../docs/api/orchestration/composite_effects.doc/filter_color_matrix_of.md")]
pub(super) fn filter_color_matrix_of(ops: Option<&[FilterOp]>) -> Option<[f32; 20]> {
    color_matrix_of_ops(ops?)
}

#[doc = include_str!("../../docs/api/orchestration/composite_effects.doc/backdrop_blur_of.md")]
pub(super) fn backdrop_blur_of(style: Option<&ComputedStyle>) -> Option<f32> {
    let radius = style
        .and_then(|style| style.get(PropertyId::BackdropFilter))
        .and_then(ComputedValue::as_filter_ops)?
        .iter()
        .filter_map(FilterOp::as_blur)
        .fold(0.0_f32, f32::max);
    (radius > 0.0).then_some(radius)
}

/// Whether the element isolates descendant blending.
pub(super) fn isolation_of(style: Option<&ComputedStyle>) -> bool {
    style
        .and_then(|style| style.get(PropertyId::Isolation))
        .is_some_and(|value| {
            matches!(
                value.value(),
                PropertyValue::Keyword(layout_keyword::ISOLATION_ISOLATE)
            )
        })
}

#[doc = include_str!("../../docs/api/orchestration/composite_effects.doc/opacity_of.md")]
pub(super) fn opacity_of(style: Option<&ComputedStyle>) -> Option<f32> {
    let alpha = style.and_then(ComputedStyle::opacity)?;
    let alpha = alpha.clamp(0.0, 1.0);
    (alpha < 1.0).then_some(alpha)
}

#[doc = include_str!("../../docs/api/orchestration/composite_effects.doc/blend_mode_of.md")]
pub(super) fn blend_mode_of(style: Option<&ComputedStyle>) -> Option<BlendMode> {
    let sym = match style
        .and_then(|style| style.get(PropertyId::MixBlendMode))
        .map(ComputedValue::value)
    {
        Some(&PropertyValue::Keyword(sym)) => sym,
        _ => return None,
    };
    blend_mode_for_keyword(sym)
}

pub(super) fn blend_mode_for_keyword(sym: obscur_identity::InternedSymbol) -> Option<BlendMode> {
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
    } else if sym == layout_keyword::MIX_BLEND_MODE_HUE {
        BlendMode::Hue
    } else if sym == layout_keyword::MIX_BLEND_MODE_SATURATION {
        BlendMode::Saturation
    } else if sym == layout_keyword::MIX_BLEND_MODE_COLOR {
        BlendMode::Color
    } else if sym == layout_keyword::MIX_BLEND_MODE_LUMINOSITY {
        BlendMode::Luminosity
    } else {
        // `normal` or an unmodelled keyword: no blend layer.
        return None;
    };
    Some(mode)
}
