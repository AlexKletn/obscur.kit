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
///
/// An **intrinsic** bound (`min-width: min-content`, …) has no arithmetic value,
/// so this — which only resolves — imposes no limit for one. A formatting context
/// that can measure its content passes the measured value through
/// [`clamp_content_measured`]; one that cannot degrades the bound to `auto`, the
/// same fallback [`Dimension::resolve`] gives every algorithm without intrinsic
/// support.
#[must_use]
pub(crate) fn clamp_content(
    extent: f32,
    min: Dimension,
    max: Dimension,
    available: f32,
    bp: f32,
    box_sizing: BoxSizing,
) -> f32 {
    clamp_content_measured(extent, (min, None), (max, None), available, bp, box_sizing)
}

/// [`clamp_content`], with the intrinsic keywords already **measured** by the
/// caller (spec §4.2, CSS Sizing 3 §4.1).
///
/// Each bound is paired with the content-axis length it measured to, which is
/// [`Some`] only when that bound is an intrinsic keyword — the one case
/// [`Dimension::resolve`] cannot answer, because the answer lives in the content.
/// A caller that hands `(bound, None)` for a non-intrinsic bound loses nothing:
/// the length is resolved from the bound itself, exactly as in [`clamp_content`].
#[must_use]
pub(crate) fn clamp_content_measured(
    extent: f32,
    min: (Dimension, Option<f32>),
    max: (Dimension, Option<f32>),
    available: f32,
    bp: f32,
    box_sizing: BoxSizing,
) -> f32 {
    let mut value = extent;
    if let Some(max_px) = bound_content(max, available, bp, box_sizing) {
        value = value.min(max_px);
    }
    if let Some(min_px) = bound_content(min, available, bp, box_sizing) {
        value = value.max(min_px);
    }
    value.max(0.0)
}

/// One `(bound, measured)` pair as a **content**-axis length, or [`None`] when it
/// imposes no limit (an `auto` bound, or an intrinsic one the caller could not
/// measure).
///
/// # `box-sizing` applies to the authored length, not to the measured one
///
/// An authored `min-width: 200px` is a border-box length under
/// `box-sizing: border-box`, so it is mapped through `box_sizing` like every other
/// authored length. `min-width: min-content` authors no length at all: what came
/// back from measuring is already a content width, so there is nothing for
/// `box-sizing` to reinterpret — the same split
/// [`intrinsic_inline_size`](super::intrinsic_inline_size) makes for an intrinsic
/// `width`.
fn bound_content(
    (bound, measured): (Dimension, Option<f32>),
    available: f32,
    bp: f32,
    box_sizing: BoxSizing,
) -> Option<f32> {
    if bound.is_intrinsic() {
        return measured;
    }
    bound
        .resolve(available)
        .map(|px| box_sizing.content_extent(px, bp))
}

#[cfg(test)]
mod tests {
    use super::{clamp_content, clamp_content_measured};
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
    fn a_measured_min_content_bound_floors_the_extent() {
        // `min-width: min-content` on a 40-wide box whose content cannot break
        // below 86.4: the box is floored at its content and overflows.
        let v = clamp_content_measured(
            40.0,
            (Dimension::MinContent, Some(86.4)),
            (Dimension::Auto, None),
            400.0,
            0.0,
            BoxSizing::ContentBox,
        );
        assert_eq!(v, 86.4);
    }

    #[test]
    fn a_measured_max_content_bound_caps_the_extent() {
        let v = clamp_content_measured(
            400.0,
            (Dimension::Auto, None),
            (Dimension::MaxContent, Some(201.6)),
            400.0,
            0.0,
            BoxSizing::ContentBox,
        );
        assert_eq!(v, 201.6);
    }

    #[test]
    fn an_unmeasured_intrinsic_bound_imposes_no_limit() {
        // What every algorithm without intrinsic support gets: the bound degrades
        // to `auto` rather than clamping to a number nobody measured.
        let v = clamp_content(
            40.0,
            Dimension::MinContent,
            Dimension::MaxContent,
            400.0,
            0.0,
            BoxSizing::ContentBox,
        );
        assert_eq!(v, 40.0);
    }

    #[test]
    fn box_sizing_reinterprets_an_authored_bound_but_not_a_measured_one() {
        // Under `border-box` an authored `min-width: 200px` floors the CONTENT at
        // 160 (40 of border+padding lives inside it)…
        let authored = clamp_content_measured(
            0.0,
            (Dimension::Px(200.0), None),
            (Dimension::Auto, None),
            950.0,
            40.0,
            BoxSizing::BorderBox,
        );
        assert_eq!(authored, 160.0);
        // …while a measured `min-content` of 200 is already a content width: it
        // floors the content at 200, border and padding sit around it.
        let measured = clamp_content_measured(
            0.0,
            (Dimension::MinContent, Some(200.0)),
            (Dimension::Auto, None),
            950.0,
            40.0,
            BoxSizing::BorderBox,
        );
        assert_eq!(measured, 200.0);
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
