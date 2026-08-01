#![doc = include_str!("spec_ch4_alignment_gap.doc/_module.md")]
#![forbid(unsafe_code)]

#[path = "spec_ch4_support/mod.rs"]
mod support;

use obscur_kit::view::view::View;
use support::{approx, at, drive, el, geom, view};

const VW: f32 = 1000.0;

#[doc = include_str!("spec_ch4_alignment_gap.doc/items.md")]
fn items(n: usize) -> View {
    let kids = (0..n).map(|i| el(100 + i as u64, "i", vec![])).collect();
    view(vec![el(10, "c", kids)])
}

#[doc = include_str!("spec_ch4_alignment_gap.doc/xs.md")]
fn xs(rt: &obscur_kit::runtime::UiRuntime, n: usize) -> Vec<f32> {
    (0..n).map(|i| geom(rt, at(rt, &[0, i])).0).collect()
}

#[doc = include_str!("spec_ch4_alignment_gap.doc/space_around_puts_half_size_gaps_at_the_ends.md")]
#[test]
fn space_around_puts_half_size_gaps_at_the_ends() {
    let rt = drive(
        VW,
        VW,
        ".c{display:flex;width:400px;height:40px;justify-content:space-around;gap:0} \
         .i{width:100px;height:10px;flex:0 0 auto}",
        &items(3),
    );
    let x = xs(&rt, 3);
    approx(x[0], 16.667, "end gap is half the inter-item gap");
    approx(x[1], 150.0, "second item");
    approx(x[2], 283.333, "third item");
}

#[doc = include_str!("spec_ch4_alignment_gap.doc/space_evenly_puts_full_size_gaps_at_the_ends.md")]
#[test]
fn space_evenly_puts_full_size_gaps_at_the_ends() {
    let rt = drive(
        VW,
        VW,
        ".c{display:flex;width:400px;height:40px;justify-content:space-evenly;gap:0} \
         .i{width:100px;height:10px;flex:0 0 auto}",
        &items(3),
    );
    let x = xs(&rt, 3);
    approx(x[0], 25.0, "all four gaps are 25px");
    approx(x[1], 150.0, "second item");
    approx(x[2], 275.0, "third item");
}

#[doc = include_str!("spec_ch4_alignment_gap.doc/space_between_falls_back_to_start_on_negative_free_space.md")]
#[test]
fn space_between_falls_back_to_start_on_negative_free_space() {
    let rt = drive(
        VW,
        VW,
        ".c{display:flex;flex-direction:row;width:200px;height:40px;\
            justify-content:space-between;gap:0} \
         .i{width:100px;height:10px;flex:0 0 auto;min-width:0}",
        &items(3),
    );
    let x = xs(&rt, 3);
    approx(x[0], 0.0, "flush from the start edge");
    approx(x[1], 100.0, "items do not overlap");
    approx(x[2], 200.0, "overflowing 100px past the end edge");
    assert!(
        x.iter().all(|v| *v >= 0.0),
        "no item has a negative x: {x:?}"
    );
}

#[doc = include_str!("spec_ch4_alignment_gap.doc/a_gutter_is_a_floor_that_justify_content_grows.md")]
#[test]
fn a_gutter_is_a_floor_that_justify_content_grows() {
    let rt = drive(
        VW,
        VW,
        ".c{display:flex;width:400px;height:40px;gap:20px;\
            justify-content:space-between} \
         .i{width:100px;height:10px;flex:0 0 auto}",
        &items(3),
    );
    let x = xs(&rt, 3);
    approx(x[0], 0.0, "first item");
    approx(
        x[1],
        150.0,
        "used gutter 50px = 20px gap + 30px of free space",
    );
    approx(x[2], 300.0, "third item; nothing overflows");
}

#[doc = include_str!("spec_ch4_alignment_gap.doc/gap_normal_is_zero_outside_multicol.md")]
#[test]
fn gap_normal_is_zero_outside_multicol() {
    let rt = drive(
        VW,
        VW,
        ".c{display:flex;width:300px;height:40px;font-size:32px;gap:normal;\
            justify-content:flex-start} \
         .i{width:100px;height:10px;flex:0 0 auto}",
        &items(2),
    );
    let x = xs(&rt, 2);
    approx(x[0], 0.0, "first item");
    approx(x[1], 100.0, "the used gap is 0px, not 1em");
}
