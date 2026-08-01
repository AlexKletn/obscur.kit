#![doc = include_str!("flex_axis_bounds.doc/_module.md")]

use crate::model::{Dimension, LayoutStyle};

#[doc = include_str!("flex_axis_bounds.doc/main_bounds.md")]
pub(super) fn main_bounds(style: &LayoutStyle, is_row: bool) -> (Dimension, Dimension) {
    if is_row {
        (style.min_width, style.max_width)
    } else {
        (style.min_height, style.max_height)
    }
}

#[doc = include_str!("flex_axis_bounds.doc/clamp_floor.md")]
pub(super) fn clamp_floor(value: f32, min_floor: f32, max_val: f32) -> f32 {
    let mut clamped = value;
    if max_val.is_finite() {
        clamped = clamped.min(max_val);
    }
    clamped.max(min_floor).max(0.0)
}

#[doc = include_str!("flex_axis_bounds.doc/axis_bp.md")]
pub(super) fn axis_bp(style: &LayoutStyle, horizontal: bool) -> f32 {
    if horizontal {
        style.border.left + style.border.right + style.padding.left + style.padding.right
    } else {
        style.border.top + style.border.bottom + style.padding.top + style.padding.bottom
    }
}

#[doc = include_str!("flex_axis_bounds.doc/cross_bounds.md")]
fn cross_bounds(style: &LayoutStyle, is_row: bool) -> (Dimension, Dimension) {
    if is_row {
        (style.min_height, style.max_height)
    } else {
        (style.min_width, style.max_width)
    }
}

#[doc = include_str!("flex_axis_bounds.doc/clamp_cross.md")]
pub(super) fn clamp_cross(
    value: f32,
    style: &LayoutStyle,
    is_row: bool,
    container_cross: f32,
) -> f32 {
    let (min, max) = cross_bounds(style, is_row);
    let bp = axis_bp(style, !is_row);
    let mut clamped = value;
    if let Some(px) = max.resolve(container_cross) {
        clamped = clamped.min(style.box_sizing.border_extent(px, bp));
    }
    if let Some(px) = min.resolve(container_cross) {
        clamped = clamped.max(style.box_sizing.border_extent(px, bp));
    }
    clamped.max(0.0)
}
