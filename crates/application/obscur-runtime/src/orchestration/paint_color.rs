#![doc = include_str!("paint_color.doc/_module.md")]

use obscur_identity::NodeId;
use obscur_scene::color::Color;
use obscur_style::computed::ComputedValue;
use obscur_style::property::{PropertyId, StyleColor};

use super::ComputedStyles;

#[doc = include_str!("paint_color.doc/current_color_of.md")]
pub(super) fn current_color_of(computed: &ComputedStyles, node: NodeId) -> StyleColor {
    computed
        .get(&node)
        .and_then(|style| style.get(PropertyId::Color))
        .and_then(ComputedValue::as_color)
        .unwrap_or(StyleColor::BLACK)
}

#[doc = include_str!("paint_color.doc/text_color_of.md")]
pub(super) fn text_color_of(computed: &ComputedStyles, node: NodeId) -> Color {
    computed
        .get(&node)
        .and_then(|style| style.get(PropertyId::Color))
        .and_then(ComputedValue::as_color)
        .map_or(Color::WHITE, scene_color_from)
}

#[doc = include_str!("paint_color.doc/scene_color_from.md")]
pub(super) fn scene_color_from(c: StyleColor) -> Color {
    Color::rgba(c.r, c.g, c.b, c.a)
}
