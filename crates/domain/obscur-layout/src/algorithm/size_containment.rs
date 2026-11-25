use obscur_geometry::{BoxConstraints, Insets};

use crate::model::{BoxSizing, Dimension, LayoutStyle, SizeContainment};

use super::clamp_content;

// Freeze the contained axes before laying out the real children. In-flow auto
// blocks still stretch in a definite inline offer. Intrinsic probes and loose
// block offers use the explicit intrinsic content size instead.
pub(super) fn contained_style(
    mut style: LayoutStyle,
    constraints: BoxConstraints,
) -> Option<LayoutStyle> {
    if style.size_containment == SizeContainment::None {
        return None;
    }
    let insets = combined_insets(&style);
    let vertical = style.writing_mode.is_vertical();
    if style.size_containment.contains_width(style.writing_mode) {
        style.width = contained_axis(
            style.width,
            style.contained_intrinsic_size.width,
            style.min_width,
            style.max_width,
            constraints.percent_width(),
            constraints.min.width,
            constraints.max.width,
            !vertical,
            insets.left + insets.right,
            style.box_sizing,
        );
        style.min_width = Dimension::Auto;
        style.max_width = Dimension::Auto;
    }
    if style.size_containment.contains_height(style.writing_mode) {
        style.height = contained_axis(
            style.height,
            style.contained_intrinsic_size.height,
            style.min_height,
            style.max_height,
            constraints.percent_height(),
            constraints.min.height,
            constraints.max.height,
            vertical,
            insets.top + insets.bottom,
            style.box_sizing,
        );
        style.min_height = Dimension::Auto;
        style.max_height = Dimension::Auto;
    }
    Some(style)
}

#[allow(clippy::too_many_arguments)]
fn contained_axis(
    preferred: Dimension,
    intrinsic: f32,
    min: Dimension,
    max: Dimension,
    basis: f32,
    offered_min: f32,
    offered_max: f32,
    inline: bool,
    insets: f32,
    sizing: BoxSizing,
) -> Dimension {
    let content = if offered_min >= offered_max && offered_max.is_finite() && offered_max > 0.0 {
        // A parent-assigned size, such as flex grow or grid stretch.
        (offered_max - insets).max(0.0)
    } else {
        let content = preferred.resolve(basis).map_or_else(
            || {
                if inline && !preferred.is_intrinsic() && offered_max.is_finite() {
                    (offered_max - insets).max(0.0)
                } else {
                    intrinsic
                }
            },
            |extent| sizing.content_extent(extent, insets),
        );
        let intrinsic = Dimension::Px(match sizing {
            BoxSizing::ContentBox => intrinsic,
            BoxSizing::BorderBox => intrinsic + insets,
        });
        let min = if min.is_intrinsic() {
            intrinsic.clone()
        } else {
            min
        };
        let max = if max.is_intrinsic() { intrinsic } else { max };
        clamp_content(content, min, max, basis, insets, sizing).max((offered_min - insets).max(0.0))
    };
    Dimension::Px(match sizing {
        BoxSizing::ContentBox => content,
        BoxSizing::BorderBox => content + insets,
    })
}

pub(super) fn combined_insets(style: &LayoutStyle) -> Insets {
    Insets {
        left: style.border.left + style.padding.left,
        right: style.border.right + style.padding.right,
        top: style.border.top + style.padding.top,
        bottom: style.border.bottom + style.padding.bottom,
    }
}
