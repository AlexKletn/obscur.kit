#![doc = include_str!("flex_cross_size.doc/_module.md")]

use obscur_geometry::{BoxConstraints, Size};

use crate::model::LayoutStyle;

use super::ChildMeasurer;
use super::flex_axis_bounds::{axis_bp, clamp_cross};

#[doc = include_str!("flex_cross_size.doc/cross_item_size.md")]
#[allow(clippy::too_many_arguments)]
pub(super) fn cross_item_size(
    stretch: bool,
    stretch_target: f32,
    cross_style: &LayoutStyle,
    is_row: bool,
    main: f32,
    container_cross: f32,
    percent_basis: Size,
    index: usize,
    measurer: &mut dyn ChildMeasurer,
) -> f32 {
    let raw = if stretch {
        let cross_dim = if is_row {
            cross_style.height
        } else {
            cross_style.width
        };
        cross_dim
            .resolve(container_cross)
            .map_or(stretch_target, |explicit| {
                cross_style
                    .box_sizing
                    .border_extent(explicit, axis_bp(cross_style, !is_row))
            })
    } else {
        let item_constraints = if is_row {
            BoxConstraints {
                min: Size::new(main, 0.0),
                max: Size::new(main, container_cross),
                percent_basis: Some(percent_basis),
            }
        } else {
            BoxConstraints {
                min: Size::new(0.0, main),
                max: Size::new(container_cross, main),
                percent_basis: Some(percent_basis),
            }
        };
        let measured = measurer.measure_child(index, item_constraints);
        if is_row {
            measured.height
        } else {
            measured.width
        }
    };
    // Every cross size — explicit, stretched, or measured — is clamped by the
    // item's own cross min/max (§9.4 step 11). Clamping here rather than at each
    // call site keeps the hypothetical and used cross sizes consistent.
    clamp_cross(raw, cross_style, is_row, container_cross)
}
