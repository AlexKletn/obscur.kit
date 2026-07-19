//! **SPECIFICATION-DERIVED** flex-layout cases for obscur.kit §4.1 / §4.13.
//!
//! Read this before touching anything in the file.
//!
//! Every expected number below was produced by an agent reading the
//! specification — obscur.kit §4 and the W3C css-flexbox-1 text it normatively
//! cites — with **no access to this repository's source**. The cases are
//! therefore a statement of what the implementation owes the spec, not a record
//! of what the implementation currently does. A failure here is a defect report
//! about the engine (or, occasionally, about the spec), and must not be
//! "corrected" by editing the expected value.
//!
//! Each test's doc comment carries the quoted normative sentence it encodes.
#![forbid(unsafe_code)]

#[path = "spec_ch4_support/mod.rs"]
mod support;

use support::{approx, at, drive, el, geom, view};

/// Two boxes wide enough that the viewport never constrains the container.
const VW: f32 = 1000.0;

/// A `.c` container holding `n` children classed `.a`, `.b`, `.c1`… in order.
fn container(classes: &[&str]) -> obscur_kit::view::view::View {
    let kids = classes
        .iter()
        .enumerate()
        .map(|(i, c)| el(100 + i as u64, c, vec![]))
        .collect();
    view(vec![el(10, "c", kids)])
}

/// css-flexbox-1 §9.7 Resolving Flexible Lengths:
/// "If using the flex shrink factor / For every unfrozen item on the line,
/// multiply its flex shrink factor by its inner flex base size, and note this as
/// its scaled flex shrink factor. Find the ratio of the item's scaled flex shrink
/// factor to the sum of the scaled flex shrink factors of all unfrozen items on
/// the line. Set the item's target main size to its flex base size minus a
/// fraction of the absolute value of the remaining free space proportional to the
/// ratio."
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

/// css-flexbox-1 §9.7:
/// "Calculate the remaining free space as for initial free space, above. If the
/// sum of the unfrozen flex items' flex factors is less than one, multiply the
/// initial free space by this sum. If the magnitude of this value is less than
/// the magnitude of the remaining free space, use this as the remaining free
/// space."
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

/// css-flexbox-1 §9.7 "Freeze over-flexed items":
/// "The total violation is the sum of the adjustments from the previous step
/// ∑(clamped size - unclamped size). If the total violation is: […] Negative /
/// Freeze all the items with max violations."
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

/// css-flexbox-1 §9.2: "When determining the flex base size, the item's min and
/// max main sizes are ignored (no clamping occurs)." and §9.7: "Positive / Freeze
/// all the items with min violations. […] Return to the start of this loop."
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

/// css-flexbox-1 §9.5 Main-Axis Alignment: "If the remaining free space is
/// positive and at least one main-axis margin on this line is auto, distribute
/// the free space equally among these margins. Otherwise, set all auto margins to
/// zero. / Align the items along the main-axis per justify-content."
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

/// css-flexbox-1 §8.1 Aligning with auto margins: "During calculations of flex
/// bases and flexible lengths, auto margins are treated as 0."
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

/// css-flexbox-1 §8.1: "Overflowing boxes ignore their auto margins and overflow
/// in the end direction." and §9.5: "Otherwise, set all auto margins to zero."
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

/// css-flexbox-1 §4.5: "the used value of a main axis automatic minimum size on a
/// flex item whose computed overflow value is non-scrollable is its content-based
/// minimum size; for scroll containers the automatic minimum size is zero, as
/// usual."
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

/// css-flexbox-1 §4.5: "For non-replaced elements / Use the larger of the content
/// size suggestion and the transferred size suggestion (if one exists), capped by
/// the specified size suggestion (if one exists)." and "specified size suggestion
/// / If the item's preferred main size is definite and not automatic, then the
/// specified size suggestion is that size."
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

/// css-flexbox-1 §9.4, "Handle 'align-content: stretch'": "increase the cross size
/// of each flex line by equal amounts such that the sum of their cross sizes
/// exactly equals the flex container's inner cross size."
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

/// css-flexbox-1 §9.4: "If the flex container is single-line and has a definite
/// cross size, the cross size of the flex line is the flex container's inner cross
/// size."
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
