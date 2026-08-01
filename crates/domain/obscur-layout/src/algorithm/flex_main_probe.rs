#![doc = include_str!("flex_main_probe.doc/_module.md")]

use obscur_geometry::{BoxConstraints, Size};

use crate::model::Dimension;

use super::ChildMeasurer;

#[doc = include_str!("flex_main_probe.doc/content_max_main.md")]
pub(super) fn content_max_main(
    measurer: &mut dyn ChildMeasurer,
    index: usize,
    is_row: bool,
    container_cross: f32,
) -> f32 {
    probe_main(measurer, index, is_row, f32::INFINITY, container_cross)
}

#[doc = include_str!("flex_main_probe.doc/probe_main.md")]
fn probe_main(
    measurer: &mut dyn ChildMeasurer,
    index: usize,
    is_row: bool,
    main_offer: f32,
    container_cross: f32,
) -> f32 {
    let max = if is_row {
        Size::new(main_offer, container_cross)
    } else {
        Size::new(container_cross, main_offer)
    };
    // The basis is stated per axis, and the two axes want different things:
    //
    // * **cross** — `container_cross` is the flex container's content extent on
    //   that axis, which is precisely the containing block a percentage there
    //   should resolve against (css-flexbox-1 §4: a flex item's containing block
    //   is its flex container's content box).
    // * **main** — [`f32::INFINITY`], because `main_offer` is `0` or infinity *on
    //   purpose*: this is the min-content / max-content question, and the whole
    //   point is that no definite main size is offered. The answer CSS gives for a
    //   percentage against an indefinite size is not "use the container's" but
    //   "behave as auto" (`specs/w3c/css-sizing-3.txt:554`: "block percentage
    //   heights resolving against an indefinite size … the property is said to
    //   behave as auto"), and an infinite basis is how this engine spells that —
    //   [`Dimension::resolve`] answers [`None`] against a non-finite base, which
    //   is the `auto` path. Stating `container_main` instead would paper the gap
    //   over with a number the spec does not ask for; stating nothing let the
    //   percentage fold against the *offer*, so a `width: 50%` item answered the
    //   min-content probe with 0 and the max-content probe with garbage.
    //
    // [`Dimension::resolve`]: crate::model::Dimension::resolve
    let percent_basis = if is_row {
        Size::new(f32::INFINITY, container_cross)
    } else {
        Size::new(container_cross, f32::INFINITY)
    };
    let measured = measurer.measure_child(
        index,
        BoxConstraints {
            min: Size::ZERO,
            max,
            percent_basis: Some(percent_basis),
        },
    );
    if is_row {
        measured.width
    } else {
        measured.height
    }
}

#[doc = include_str!("flex_main_probe.doc/intrinsic_main.md")]
pub(super) fn intrinsic_main(
    measurer: &mut dyn ChildMeasurer,
    index: usize,
    dim: Dimension,
    is_row: bool,
    container_main: f32,
    container_cross: f32,
) -> Option<f32> {
    match dim {
        Dimension::MinContent => Some(probe_main(measurer, index, is_row, 0.0, container_cross)),
        Dimension::MaxContent => Some(content_max_main(measurer, index, is_row, container_cross)),
        Dimension::FitContent => {
            let min = probe_main(measurer, index, is_row, 0.0, container_cross);
            let max = content_max_main(measurer, index, is_row, container_cross);
            Some(max.min(container_main.max(min)))
        }
        Dimension::Auto | Dimension::Px(_) | Dimension::Percent(_) => None,
    }
}
