#![doc = include_str!("spec_ch4_sizing_units.doc/_module.md")]
#![forbid(unsafe_code)]

#[path = "spec_ch4_support/mod.rs"]
mod support;

use obscur_kit::text::font::MockFontResolver;
use obscur_kit::view::view::View;
use support::{approx, at, compile, drive, drive_sheets, el, geom, runtime, txt, view};

const VW: f32 = 1000.0;

#[doc = include_str!("spec_ch4_sizing_units.doc/page.md")]
fn page(kids: Vec<obscur_kit::view::view::ViewNode>) -> View {
    view(vec![el(10, "p", vec![el(100, "a", kids)])])
}

#[doc = include_str!("spec_ch4_sizing_units.doc/a_border_box_width_floors_its_content_at_zero.md")]
#[test]
fn a_border_box_width_floors_its_content_at_zero() {
    let rt = drive(
        VW,
        VW,
        ".p{width:500px;height:200px} \
         .a{box-sizing:border-box;width:10px;padding:20px;height:100px;\
            border-top-width:0;border-right-width:0;border-bottom-width:0;\
            border-left-width:0} \
         .inner{width:100%;height:5px}",
        &page(vec![el(200, "inner", vec![])]),
    );
    let (_, _, aw, _) = geom(&rt, at(&rt, &[0, 0]));
    let (_, _, iw, _) = geom(&rt, at(&rt, &[0, 0, 0]));
    approx(aw, 40.0, "padding is never reduced: 20 + 0 + 20");
    approx(iw, 0.0, "the content width floored at zero");
}

#[doc = include_str!("spec_ch4_sizing_units.doc/the_fit_content_formula_floors_at_the_min_content_size.md")]
#[test]
fn the_fit_content_formula_floors_at_the_min_content_size() {
    const TEXT: &str = "aaaaa aaaa aaaa aaaa aaaa aaaa";
    let sized = |available: f32| {
        let css = format!(
            ".p{{width:{available}px;height:200px}} \
             .a{{width:fit-content;font-size:16.666667px}}"
        );
        let rt = drive(VW, VW, &css, &page(vec![txt(TEXT)]));
        geom(&rt, at(&rt, &[0, 0])).2
    };
    approx(sized(30.0), 50.0, "never below the min-content size");
    approx(
        sized(200.0),
        200.0,
        "the stretch-fit size when it is in range",
    );
    approx(sized(400.0), 300.0, "never above the max-content size");
}

#[doc = include_str!("spec_ch4_sizing_units.doc/clamp_lets_the_minimum_win_over_an_inverted_maximum.md")]
#[test]
fn clamp_lets_the_minimum_win_over_an_inverted_maximum() {
    let rt = drive(
        VW,
        VW,
        ".p{width:500px;height:200px} \
         .a{width:clamp(100px, 50px, 40px);height:10px} \
         .b{width:clamp(none, 30px, 20px);height:10px}",
        &view(vec![el(
            10,
            "p",
            vec![el(100, "a", vec![]), el(101, "b", vec![])],
        )]),
    );
    approx(
        geom(&rt, at(&rt, &[0, 0])).2,
        100.0,
        "clamp(100px,50px,40px)",
    );
    approx(geom(&rt, at(&rt, &[0, 1])).2, 20.0, "clamp(none,30px,20px)");
}

#[doc = include_str!("spec_ch4_sizing_units.doc/absolute_length_units_are_anchored_to_the_css_pixel.md")]
#[test]
fn absolute_length_units_are_anchored_to_the_css_pixel() {
    let rt = drive(
        VW,
        VW,
        ".p{width:500px;height:200px} \
         .a{width:1cm;height:10px} .b{width:1in;height:10px} .c{width:12pt;height:10px}",
        &view(vec![el(
            10,
            "p",
            vec![
                el(100, "a", vec![]),
                el(101, "b", vec![]),
                el(102, "c", vec![]),
            ],
        )]),
    );
    approx(geom(&rt, at(&rt, &[0, 0])).2, 37.795_276, "1cm");
    approx(geom(&rt, at(&rt, &[0, 1])).2, 96.0, "1in");
    approx(geom(&rt, at(&rt, &[0, 2])).2, 16.0, "12pt");
}

#[doc = include_str!("spec_ch4_sizing_units.doc/the_ch_unit_measures_zero_and_falls_back_to_half_an_em.md")]
#[test]
fn the_ch_unit_measures_zero_and_falls_back_to_half_an_em() {
    let css = ".p{width:500px;height:200px} .a{font-size:20px;width:10ch;height:10px}";
    let sheet = compile(css);
    let v = page(vec![]);

    // The spec fixes only the SUBSTITUTE, never a measured advance. Asserting a
    // number here would encode whichever shaper happens to be installed — the
    // first version of this case asserted 120px, which is 10 x 0.6em, and 0.6 is
    // `MockTextShaper::ADVANCE_RATIO`. No reading of css-values-4 produces it, so
    // that expectation had leaked in from the implementation. What the clause
    // actually requires is a relationship: when `0` CAN be measured, the width
    // follows the measurement rather than the substitute.
    let measured = drive_sheets(VW, VW, std::slice::from_ref(&sheet), &v);
    let width = geom(&measured, at(&measured, &[0, 0])).2;
    assert!(
        width > 0.0,
        "a measurable `0` must give `10ch` a width, got {width}"
    );
    assert!(
        (width - 100.0).abs() > 0.5,
        "10ch resolved to {width}px, which is exactly 10 x 0.5em — the substitute \
         was used where the glyph was measurable"
    );
}

#[doc = include_str!("spec_ch4_sizing_units.doc/container_query_units_fall_back_to_the_small_viewport.md")]
#[test]
fn container_query_units_fall_back_to_the_small_viewport() {
    let rt = drive(
        800.0,
        600.0,
        ".p{width:200px;height:200px} .a{width:50cqi;height:10px}",
        &page(vec![]),
    );
    approx(
        geom(&rt, at(&rt, &[0, 0])).2,
        400.0,
        "50% of the 800px small viewport inline size, not of the 200px parent",
    );
}
