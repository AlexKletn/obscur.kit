//! **SPECIFICATION-DERIVED** distributed-alignment and gutter cases for
//! obscur.kit §4.1 ("alignment", "gap").
//!
//! Every expected number below was produced by an agent reading the
//! specification — obscur.kit §4 and the W3C css-align-3 text it normatively
//! cites — with **no access to this repository's source**. A failure here is a
//! defect report about the engine, and must not be "corrected" by editing the
//! expected value.
//!
//! Translation note: the deriver's `column-gap` is authored here as the `gap`
//! shorthand, because `column-gap` / `row-gap` are not modelled as properties in
//! this workspace. Every assertion in this file reads only main-axis geometry of
//! a single-line row flex container, which the shorthand sets identically. The
//! `gap: normal` case inherits a caveat from that: if `normal` fails to parse,
//! `gap` falls back to its initial value and the test can pass for the wrong
//! reason — see the test's own comment.
#![forbid(unsafe_code)]

#[path = "spec_ch4_support/mod.rs"]
mod support;

use obscur_kit::view::view::View;
use support::{approx, at, drive, el, geom, view};

const VW: f32 = 1000.0;

/// A `.c` flex container holding `n` items all classed `.i`.
fn items(n: usize) -> View {
    let kids = (0..n).map(|i| el(100 + i as u64, "i", vec![])).collect();
    view(vec![el(10, "c", kids)])
}

/// The x origin of each item, in order.
fn xs(rt: &obscur_kit::runtime::UiRuntime, n: usize) -> Vec<f32> {
    (0..n).map(|i| geom(rt, at(rt, &[0, i])).0).collect()
}

/// css-align-3 §4.3, `space-around`: "The alignment subjects are distributed so
/// that the spacing between any two adjacent alignment subjects is the same, and
/// the spacing before the first and after the last alignment subject is half the
/// size of the other spacing."
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

/// css-align-3 §4.3, `space-evenly`: "The alignment subjects are evenly
/// distributed in the alignment container, with a full-size space on either end.
/// The alignment subjects are distributed so that the spacing between any two
/// adjacent alignment subjects, before the first alignment subject, and after the
/// last alignment subject is the same."
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

/// css-align-3 §4.3: "When space cannot be distributed in this way, these values
/// behave as their fallback alignment." and "The default fallback alignment for
/// this value is safe flex-start."
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

/// css-align-3 §8.1: "Gutters effect a minimum spacing between items: additional
/// spacing may be added by justify-content/align-content. Such additional space
/// effectively increases the size of these gutters."
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

/// css-align-3 §8.1: "The value normal represents a used value of 1em on
/// multi-column containers, and a used value of 0px in all other contexts."
///
/// Caveat recorded honestly: `gap` has no `normal` keyword in this workspace's
/// property model, so a parse failure would also yield 0 and pass this test for
/// the wrong reason. The `font-size: 32px` is retained from the deriver's setup
/// precisely because it makes the *wrong* answer (132) far from the right one.
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
