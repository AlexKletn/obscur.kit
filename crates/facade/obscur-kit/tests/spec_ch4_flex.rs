#![doc = include_str!("spec_ch4_flex.doc/_module.md")]
#![forbid(unsafe_code)]

#[path = "spec_ch4_support/mod.rs"]
mod support;

use support::{approx, at, drive, el, geom, view};

#[doc = include_str!("spec_ch4_flex.doc/VW.md")]
const VW: f32 = 1000.0;

#[doc = include_str!("spec_ch4_flex.doc/container.md")]
fn container(classes: &[&str]) -> obscur_kit::view::view::View {
    let kids = classes
        .iter()
        .enumerate()
        .map(|(i, c)| el(100 + i as u64, c, vec![]))
        .collect();
    view(vec![el(10, "c", kids)])
}

#[doc = include_str!("spec_ch4_flex.doc/flex_shrink_is_scaled_by_the_flex_base_size.md")]
#[test]
fn flex_shrink_is_scaled_by_the_flex_base_size() {
    let rt = drive(
        VW,
        VW,
        ".c{display:flex;width:100px;height:20px} \
         .a{width:100px;flex:0 1 auto;min-width:0;height:10px} \
         .b{width:200px;flex:0 1 auto;min-width:0;height:10px}",
        &container(&["a", "b"]),
    );
    let (ax, _, aw, _) = geom(&rt, at(&rt, &[0, 0]));
    let (bx, _, bw, _) = geom(&rt, at(&rt, &[0, 1]));
    approx(aw, 33.333, "a width = 100 - 200*(100/300)");
    approx(bw, 66.667, "b width = 200 - 200*(200/300)");
    approx(ax, 0.0, "a.x");
    approx(bx, 33.333, "b.x");
}

#[doc = include_str!("spec_ch4_flex.doc/a_flex_grow_factor_sum_below_one_leaves_space_unfilled.md")]
#[test]
fn a_flex_grow_factor_sum_below_one_leaves_space_unfilled() {
    let rt = drive(
        VW,
        VW,
        ".c{display:flex;width:400px;height:20px} \
         .a{flex:0.5 1 0px;min-width:0;height:10px}",
        &container(&["a"]),
    );
    let (_, _, aw, _) = geom(&rt, at(&rt, &[0, 0]));
    approx(aw, 200.0, "0.5fr of 400px of free space, not all of it");
}

#[doc = include_str!("spec_ch4_flex.doc/a_max_width_violation_refreezes_and_redistributes.md")]
#[test]
fn a_max_width_violation_refreezes_and_redistributes() {
    let rt = drive(
        VW,
        VW,
        ".c{display:flex;width:300px;height:20px} \
         .a{flex:1 1 0px;min-width:0;max-width:50px;height:10px} \
         .b{flex:1 1 0px;min-width:0;height:10px} \
         .c1{flex:1 1 0px;min-width:0;height:10px}",
        &container(&["a", "b", "c1"]),
    );
    let (ax, _, aw, _) = geom(&rt, at(&rt, &[0, 0]));
    let (bx, _, bw, _) = geom(&rt, at(&rt, &[0, 1]));
    let (cx, _, cw, _) = geom(&rt, at(&rt, &[0, 2]));
    approx(aw, 50.0, "a clamped to max-width");
    approx(bw, 125.0, "b takes half of the freed space");
    approx(cw, 125.0, "c takes half of the freed space");
    approx(ax, 0.0, "a.x");
    approx(bx, 50.0, "b.x");
    approx(cx, 175.0, "c.x");
}

#[doc = include_str!("spec_ch4_flex.doc/a_min_width_violation_refreezes_and_reflexes.md")]
#[test]
fn a_min_width_violation_refreezes_and_reflexes() {
    let rt = drive(
        VW,
        VW,
        ".c{display:flex;width:100px;height:20px} \
         .a{flex:0 1 100px;min-width:90px;height:10px} \
         .b{flex:0 1 100px;min-width:0;height:10px}",
        &container(&["a", "b"]),
    );
    let (_, _, aw, _) = geom(&rt, at(&rt, &[0, 0]));
    let (bx, _, bw, _) = geom(&rt, at(&rt, &[0, 1]));
    approx(aw, 90.0, "a frozen at its min-width");
    approx(bw, 10.0, "b absorbs the rest of the shrink");
    approx(bx, 90.0, "b.x");
}

#[doc = include_str!("spec_ch4_flex.doc/an_auto_margin_absorbs_the_free_space_before_justify_content.md")]
#[test]
fn an_auto_margin_absorbs_the_free_space_before_justify_content() {
    let rt = drive(
        VW,
        VW,
        ".c{display:flex;width:300px;height:20px;justify-content:center} \
         .a{width:100px;height:10px;margin-left:auto}",
        &container(&["a"]),
    );
    let (ax, _, aw, _) = geom(&rt, at(&rt, &[0, 0]));
    approx(aw, 100.0, "a width");
    approx(
        ax,
        200.0,
        "the auto margin ate all 200px; centring had none left",
    );
}

#[doc = include_str!("spec_ch4_flex.doc/auto_margins_are_zero_while_flexible_lengths_resolve.md")]
#[test]
fn auto_margins_are_zero_while_flexible_lengths_resolve() {
    let rt = drive(
        VW,
        VW,
        ".c{display:flex;width:300px;height:20px} \
         .a{flex:1 1 0px;min-width:0;height:10px;margin-left:auto}",
        &container(&["a"]),
    );
    let (ax, _, aw, _) = geom(&rt, at(&rt, &[0, 0]));
    approx(aw, 300.0, "flexing consumed the free space first");
    approx(ax, 0.0, "so the auto margin used value is 0");
}

#[doc = include_str!("spec_ch4_flex.doc/an_overflowing_item_ignores_its_auto_margins.md")]
#[test]
fn an_overflowing_item_ignores_its_auto_margins() {
    let rt = drive(
        VW,
        VW,
        ".c{display:flex;flex-direction:row;width:100px;height:20px} \
         .a{width:200px;flex:0 0 auto;min-width:0;height:10px;\
            margin-left:auto;margin-right:auto}",
        &container(&["a"]),
    );
    let (ax, _, aw, _) = geom(&rt, at(&rt, &[0, 0]));
    approx(aw, 200.0, "a width");
    approx(
        ax,
        0.0,
        "overflow goes out the end edge, not half out the start",
    );
}

#[doc = include_str!("spec_ch4_flex.doc/the_automatic_minimum_size_depends_on_overflow.md")]
#[test]
fn the_automatic_minimum_size_depends_on_overflow() {
    // `.inner` gives `.a` a min-content main size of exactly 80px.
    let v = view(vec![el(
        10,
        "c",
        vec![el(100, "a", vec![el(200, "inner", vec![])])],
    )]);
    let base = ".c{display:flex;width:50px;height:40px} \
                .a{flex:0 1 200px;height:20px} \
                .inner{width:80px;height:10px}";

    let visible = drive(VW, VW, &format!("{base} .a{{overflow:visible}}"), &v);
    let (_, _, w_visible, _) = geom(&visible, at(&visible, &[0, 0]));
    approx(
        w_visible,
        80.0,
        "non-scrollable: content-based minimum applies",
    );

    let hidden = drive(VW, VW, &format!("{base} .a{{overflow:hidden}}"), &v);
    let (_, _, w_hidden, _) = geom(&hidden, at(&hidden, &[0, 0]));
    approx(
        w_hidden,
        50.0,
        "scroll container: automatic minimum is zero",
    );
}

#[doc = include_str!("spec_ch4_flex.doc/the_specified_size_suggestion_caps_the_content_based_minimum.md")]
#[test]
fn the_specified_size_suggestion_caps_the_content_based_minimum() {
    let v = view(vec![el(
        10,
        "c",
        vec![el(100, "a", vec![el(200, "inner", vec![])])],
    )]);
    let rt = drive(
        VW,
        VW,
        ".c{display:flex;width:10px;height:40px} \
         .a{width:30px;flex:0 1 200px;overflow:visible;height:20px} \
         .inner{width:80px;height:10px}",
        &v,
    );
    let (_, _, aw, _) = geom(&rt, at(&rt, &[0, 0]));
    approx(
        aw,
        30.0,
        "content minimum 80px capped by the author's width 30px",
    );
}

#[doc = include_str!("spec_ch4_flex.doc/align_content_stretch_grows_lines_by_equal_amounts.md")]
#[test]
fn align_content_stretch_grows_lines_by_equal_amounts() {
    let rt = drive(
        VW,
        VW,
        ".c{display:flex;flex-wrap:wrap;width:100px;height:300px;\
            align-content:stretch;align-items:flex-start} \
         .a{width:80px;height:40px} \
         .b{width:80px;height:60px}",
        &container(&["a", "b"]),
    );
    let (_, ay, _, _) = geom(&rt, at(&rt, &[0, 0]));
    let (_, by, _, _) = geom(&rt, at(&rt, &[0, 1]));
    approx(ay, 0.0, "line 1 starts at the top");
    approx(
        by,
        140.0,
        "line 1 is 40+100=140 tall, so line 2 starts at 140",
    );
}

#[doc = include_str!("spec_ch4_flex.doc/a_single_flex_line_takes_the_containers_inner_cross_size.md")]
#[test]
fn a_single_flex_line_takes_the_containers_inner_cross_size() {
    let v = view(vec![el(
        10,
        "c",
        vec![
            el(100, "a", vec![]),
            el(101, "b", vec![el(200, "inner", vec![])]),
        ],
    )]);
    let rt = drive(
        VW,
        VW,
        ".c{display:flex;flex-wrap:nowrap;width:200px;height:200px;padding:0} \
         .a{width:10px;height:50px;align-self:flex-start} \
         .b{width:10px;align-self:stretch} \
         .inner{width:5px;height:20px}",
        &v,
    );
    let (_, by, _, bh) = geom(&rt, at(&rt, &[0, 1]));
    approx(by, 0.0, "b starts at the container's cross start");
    approx(bh, 200.0, "b stretches to the container's inner cross size");
}
