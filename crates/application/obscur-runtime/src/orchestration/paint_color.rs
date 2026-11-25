//! The scene paint colors a node paints with, resolved from its computed
//! style: its `currentColor`, its text ink, and the authored-to-scene color
//! conversion every painter in this pass shares.

use obscur_scene::color::Color;
use obscur_style::computed::{ComputedStyle, ComputedValue};
use obscur_style::property::{PropertyId, StyleColor};

/// Reads a node's computed `color` as a [`StyleColor`] for the `currentColor`
/// border fallback, defaulting to [`StyleColor::BLACK`] (the CSS `color` initial)
/// when absent.
pub(super) fn current_color_of(style: Option<&ComputedStyle>) -> StyleColor {
    style
        .and_then(|style| style.get(PropertyId::Color))
        .and_then(ComputedValue::as_color)
        .unwrap_or(StyleColor::BLACK)
}

#[doc = include_str!("../../docs/api/orchestration/paint_color.doc/text_color_of.md")]
pub(super) fn text_color_of(style: Option<&ComputedStyle>) -> Color {
    style
        .and_then(|style| style.get(PropertyId::Color))
        .and_then(ComputedValue::as_color)
        .map_or(Color::WHITE, scene_color_from)
}

#[doc = include_str!("../../docs/api/orchestration/paint_color.doc/scene_color_from.md")]
pub(super) fn scene_color_from(c: StyleColor) -> Color {
    Color::rgba(c.r, c.g, c.b, c.a)
}
