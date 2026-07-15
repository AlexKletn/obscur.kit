//! Clamping a node's own content-axis size into its authored min/max (spec §4.2).
//!
//! A node's resolved content width/height must be clamped into its
//! `min-`/`max-` bounds for that axis before its border box is formed. Every
//! formatting context (flow, flex, grid) needs the same rule for the node's
//! *own* box, so it lives here rather than being duplicated (and, historically,
//! dropped) per algorithm.

use crate::model::{BoxSizing, Dimension};

/// Clamps a resolved content-axis `extent` into the authored `min`/`max` for
/// that axis.
///
/// Each bound is resolved against `available` and mapped through `box_sizing`
/// (an authored min/max is a border-box length under `border-box`), matching how
/// the explicit size is resolved. An [`Auto`](Dimension::Auto) bound imposes no
/// limit. The max is applied before the min, so a min larger than the max wins
/// (per CSS). The result is never negative.
#[must_use]
pub(crate) fn clamp_content(
    extent: f32,
    min: Dimension,
    max: Dimension,
    available: f32,
    bp: f32,
    box_sizing: BoxSizing,
) -> f32 {
    let mut value = extent;
    if let Some(max_px) = max.resolve(available) {
        value = value.min(box_sizing.content_extent(max_px, bp));
    }
    if let Some(min_px) = min.resolve(available) {
        value = value.max(box_sizing.content_extent(min_px, bp));
    }
    value.max(0.0)
}

#[cfg(test)]
mod tests {
    use super::clamp_content;
    use crate::model::{BoxSizing, Dimension};

    #[test]
    fn auto_bounds_leave_the_extent_unchanged() {
        let v = clamp_content(
            100.0,
            Dimension::Auto,
            Dimension::Auto,
            500.0,
            0.0,
            BoxSizing::ContentBox,
        );
        assert_eq!(v, 100.0);
    }

    #[test]
    fn min_floors_a_short_extent() {
        // min-height 100% of 950 available floors a 680 content extent to 950.
        let v = clamp_content(
            680.0,
            Dimension::Percent(100.0),
            Dimension::Auto,
            950.0,
            0.0,
            BoxSizing::ContentBox,
        );
        assert_eq!(v, 950.0);
    }

    #[test]
    fn max_caps_a_tall_extent() {
        let v = clamp_content(
            800.0,
            Dimension::Auto,
            Dimension::Px(300.0),
            950.0,
            0.0,
            BoxSizing::ContentBox,
        );
        assert_eq!(v, 300.0);
    }

    #[test]
    fn min_wins_over_a_smaller_max() {
        let v = clamp_content(
            50.0,
            Dimension::Px(400.0),
            Dimension::Px(300.0),
            950.0,
            0.0,
            BoxSizing::ContentBox,
        );
        assert_eq!(v, 400.0);
    }

    #[test]
    fn border_box_min_subtracts_border_and_padding() {
        // Under border-box, a min of 200 with 40 of border+padding floors the
        // *content* extent to 160.
        let v = clamp_content(
            0.0,
            Dimension::Px(200.0),
            Dimension::Auto,
            950.0,
            40.0,
            BoxSizing::BorderBox,
        );
        assert_eq!(v, 160.0);
    }
}
