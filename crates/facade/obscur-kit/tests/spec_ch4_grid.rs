#![doc = include_str!("spec_ch4_grid.doc/_module.md")]
#![forbid(unsafe_code)]

#[path = "spec_ch4_support/mod.rs"]
mod support;

use obscur_kit::view::view::View;
use support::{approx, at, drive, el, geom, view};

const VW: f32 = 1000.0;

#[doc = include_str!("spec_ch4_grid.doc/grid.md")]
fn grid(classes: &[&str]) -> View {
    let kids = classes
        .iter()
        .enumerate()
        .map(|(i, c)| el(100 + i as u64, c, vec![]))
        .collect();
    view(vec![el(10, "g", kids)])
}

#[doc = include_str!("spec_ch4_grid.doc/shrink_to_fit_grid.md")]
fn shrink_to_fit_grid(kids: Vec<obscur_kit::view::view::ViewNode>) -> View {
    view(vec![el(9, "wrap", vec![el(10, "g", kids)])])
}

const WRAP: &str = ".wrap{display:flex;width:1000px;height:400px} \
                   .g{flex:0 0 auto}";

#[doc = include_str!("spec_ch4_grid.doc/an_fr_factor_sum_below_one_leaves_space_unfilled.md")]
#[test]
fn an_fr_factor_sum_below_one_leaves_space_unfilled() {
    let rt = drive(
        VW,
        VW,
        ".g{display:grid;width:400px;height:40px;\
            grid-template-columns:0.25fr 0.25fr 0.25fr;gap:0} \
         .a,.b,.c{height:10px}",
        &grid(&["a", "b", "c"]),
    );
    let (ax, _, aw, _) = geom(&rt, at(&rt, &[0, 0]));
    let (bx, _, bw, _) = geom(&rt, at(&rt, &[0, 1]));
    let (cx, _, cw, _) = geom(&rt, at(&rt, &[0, 2]));
    approx(aw, 100.0, "0.25fr of 400px");
    approx(bw, 100.0, "0.25fr of 400px");
    approx(cw, 100.0, "0.25fr of 400px");
    approx(ax, 0.0, "column 1 start");
    approx(bx, 100.0, "column 2 start");
    approx(cx, 200.0, "column 3 start; the last 100px stays empty");
}

#[doc = include_str!("spec_ch4_grid.doc/an_fr_track_blown_by_content_restarts_the_fr_algorithm.md")]
#[test]
fn an_fr_track_blown_by_content_restarts_the_fr_algorithm() {
    let v = view(vec![el(
        10,
        "g",
        vec![
            el(100, "a", vec![]),
            el(101, "b", vec![el(200, "inner", vec![])]),
        ],
    )]);
    let rt = drive(
        VW,
        VW,
        ".g{display:grid;width:300px;height:40px;\
            grid-template-columns:1fr 1fr;gap:0} \
         .a{height:10px} .b{height:10px} .inner{width:250px;height:10px}",
        &v,
    );
    let (_, _, aw, _) = geom(&rt, at(&rt, &[0, 0]));
    let (bx, _, bw, _) = geom(&rt, at(&rt, &[0, 1]));
    approx(
        aw,
        50.0,
        "column 1 gets what is left after the inflexible track",
    );
    approx(bw, 250.0, "column 2 is pinned to its base size");
    approx(bx, 50.0, "column 2 starts at 50");
}

#[doc = include_str!("spec_ch4_grid.doc/auto_fill_repetition_count_accounts_for_the_gap.md")]
#[test]
fn auto_fill_repetition_count_accounts_for_the_gap() {
    let rt = drive(
        VW,
        VW,
        ".g{display:grid;width:500px;height:200px;\
            grid-template-columns:repeat(auto-fill,100px);gap:20px} \
         .a,.b,.c,.d,.e{height:10px}",
        &grid(&["a", "b", "c", "d", "e"]),
    );
    for (i, want) in [0.0_f32, 120.0, 240.0, 360.0].into_iter().enumerate() {
        let (x, _, _, _) = geom(&rt, at(&rt, &[0, i]));
        approx(x, want, "column start of the auto-filled track");
    }
    // A fifth item proves there is no fifth column: it wraps to row 2, column 1.
    let (x5, y5, _, _) = geom(&rt, at(&rt, &[0, 4]));
    let (_, y1, _, _) = geom(&rt, at(&rt, &[0, 0]));
    approx(
        x5,
        0.0,
        "the 5th item wrapped: exactly 4 columns were generated",
    );
    assert!(y5 > y1, "the 5th item is on the second row, got y={y5}");
}

#[doc = include_str!("spec_ch4_grid.doc/auto_fill_under_an_indefinite_size_repeats_once.md")]
#[test]
fn auto_fill_under_an_indefinite_size_repeats_once() {
    let rt = drive(
        VW,
        VW,
        &format!(
            "{WRAP} .g{{display:grid;grid-template-columns:repeat(auto-fill,100px)}} \
             .a{{height:10px}}"
        ),
        &shrink_to_fit_grid(vec![el(100, "a", vec![])]),
    );
    let (_, _, gw, _) = geom(&rt, at(&rt, &[0, 0]));
    let (_, _, aw, _) = geom(&rt, at(&rt, &[0, 0, 0]));
    approx(
        gw,
        100.0,
        "one repetition, so the grid's used inline size is 100px",
    );
    approx(aw, 100.0, "the single 100px column");
}

#[doc = include_str!("spec_ch4_grid.doc/an_out_of_range_placement_adds_an_implicit_column.md")]
#[test]
fn an_out_of_range_placement_adds_an_implicit_column() {
    let rt = drive(
        VW,
        VW,
        &format!(
            "{WRAP} .g{{display:grid;grid-template-columns:repeat(5,100px);\
             grid-auto-columns:50px;grid-auto-flow:row;gap:0}} \
             .a{{grid-column:4 / span 3;height:10px}}"
        ),
        &shrink_to_fit_grid(vec![el(100, "a", vec![])]),
    );
    let (_, _, gw, _) = geom(&rt, at(&rt, &[0, 0]));
    let (ax, _, aw, _) = geom(&rt, at(&rt, &[0, 0, 0]));
    approx(ax, 300.0, "the item starts at column 4");
    approx(
        aw,
        250.0,
        "columns 4 and 5 (100 each) plus the implicit 6th (50)",
    );
    approx(gw, 550.0, "5x100 explicit + 1x50 implicit");
}

#[doc = include_str!("spec_ch4_grid.doc/sparse_auto_placement_never_backtracks.md")]
#[test]
fn sparse_auto_placement_never_backtracks() {
    let rt = drive(
        VW,
        VW,
        ".g{display:grid;width:300px;grid-template-columns:repeat(3,100px);\
            grid-auto-rows:50px;grid-auto-flow:row;gap:0} \
         .a{grid-column:span 2} .b{grid-column:span 3} .c{grid-column:span 1}",
        &grid(&["a", "b", "c"]),
    );
    let (ax, ay, aw, _) = geom(&rt, at(&rt, &[0, 0]));
    let (_, by, _, _) = geom(&rt, at(&rt, &[0, 1]));
    let (cx, cy, cw, _) = geom(&rt, at(&rt, &[0, 2]));
    let (_, _, _, gh) = geom(&rt, at(&rt, &[0]));
    approx(ax, 0.0, "a at row 1 column 1");
    approx(ay, 0.0, "a at row 1");
    approx(aw, 200.0, "a spans two columns");
    approx(by, 50.0, "b at row 2");
    approx(cx, 0.0, "c at column 1, NOT backfilled into row 1 column 3");
    approx(cy, 100.0, "c at row 3");
    approx(cw, 100.0, "c spans one column");
    approx(gh, 150.0, "three rows of 50px");
}

#[doc = include_str!("spec_ch4_grid.doc/dense_auto_placement_backfills_the_earlier_hole.md")]
#[test]
fn dense_auto_placement_backfills_the_earlier_hole() {
    let rt = drive(
        VW,
        VW,
        ".g{display:grid;width:300px;grid-template-columns:repeat(3,100px);\
            grid-auto-rows:50px;grid-auto-flow:row dense;gap:0} \
         .a{grid-column:span 2} .b{grid-column:span 3} .c{grid-column:span 1}",
        &grid(&["a", "b", "c"]),
    );
    let (cx, cy, _, _) = geom(&rt, at(&rt, &[0, 2]));
    let (_, _, _, gh) = geom(&rt, at(&rt, &[0]));
    approx(cx, 200.0, "c backfills row 1 column 3");
    approx(cy, 0.0, "c is on row 1");
    approx(gh, 100.0, "two rows of 50px");
}

#[doc = include_str!("spec_ch4_grid.doc/an_auto_placed_span_widens_the_implicit_grid.md")]
#[test]
fn an_auto_placed_span_widens_the_implicit_grid() {
    let rt = drive(
        VW,
        VW,
        &format!(
            "{WRAP} .g{{display:grid;grid-template-columns:100px;\
             grid-auto-columns:40px;grid-auto-flow:row;gap:0}} \
             .a{{grid-column:span 3;height:10px}}"
        ),
        &shrink_to_fit_grid(vec![el(100, "a", vec![])]),
    );
    let (_, _, gw, _) = geom(&rt, at(&rt, &[0, 0]));
    let (ax, _, aw, _) = geom(&rt, at(&rt, &[0, 0, 0]));
    approx(ax, 0.0, "the item starts at column 1");
    approx(aw, 180.0, "100 + 40 + 40");
    approx(gw, 180.0, "the implicit grid grew to three columns");
}

#[doc = include_str!("spec_ch4_grid.doc/an_infinitely_growable_track_absorbs_the_whole_max_content_remainder.md")]
#[test]
fn an_infinitely_growable_track_absorbs_the_whole_max_content_remainder() {
    let v = shrink_to_fit_grid(vec![
        el(100, "i1", vec![el(200, "inner", vec![])]),
        el(101, "i2", vec![support::txt("aaa aaa aa")]),
    ]);
    let rt = drive(
        VW,
        VW,
        &format!(
            "{WRAP} .g{{display:grid;grid-template-columns:auto auto;gap:0}} \
             .i1{{grid-column:1;grid-row:1}} .inner{{width:10px;height:10px}} \
             .i2{{grid-column:1 / span 2;grid-row:2;font-size:16.666667px}}"
        ),
        &v,
    );
    let (_, _, gw, _) = geom(&rt, at(&rt, &[0, 0]));
    let (_, _, w1, _) = geom(&rt, at(&rt, &[0, 0, 0]));
    let (x2, _, w2, _) = geom(&rt, at(&rt, &[0, 0, 1]));
    approx(w1, 10.0, "column 1 is 10px");
    approx(gw, 100.0, "the grid's used inline size is 100px");
    approx(x2, 0.0, "item 2 starts at column 1");
    approx(w2, 100.0, "item 2 spans both columns: 10 + 90");
}
