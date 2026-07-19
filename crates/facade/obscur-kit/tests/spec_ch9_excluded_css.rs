//! **SPECIFICATION-DERIVED** cases for the CSS features obscur.kit §9.1 declares
//! non-goals and §4.1/§4.13 exclude normatively: float/clear, table layout,
//! multicol, legacy clip/zoom/page-break, browser link/visited/target state, and
//! network loading through `@import`.
//!
//! Every expected value was produced by an agent reading the specification with
//! **no access to this repository's source**. A failure here is a defect report
//! about the engine, and must not be "corrected" by editing the expected value.
#![forbid(unsafe_code)]

#[path = "spec_ch9_support/mod.rs"]
mod support;

use obscur_kit::css::error::UnsupportedFeatureKind;
use support::{approx, assert_color, at, drive, el, geom, unsupported, view};

const VW: f32 = 1000.0;

/// §4.1 Layout models — "Excluded legacy/browser baggage:" / "- float/clear;"
/// (making concrete §9.1 "legacy browser layout behavior").
///
/// NG-FLOAT-01.
#[test]
fn float_left_does_not_place_children_side_by_side() {
    let rt = drive(
        VW,
        VW,
        ".box{width:300px}\
         .f{width:100px;height:50px;float:left}",
        &view(vec![el(
            1,
            "box",
            vec![el(2, "f", vec![]), el(3, "f", vec![])],
        )]),
    );
    let (x1, y1, w1, h1) = geom(&rt, at(&rt, &[0, 0]));
    let (x2, y2, w2, h2) = geom(&rt, at(&rt, &[0, 1]));
    approx(x1, 0.0, "child 1 x");
    approx(y1, 0.0, "child 1 y");
    approx(w1, 100.0, "child 1 width");
    approx(h1, 50.0, "child 1 height");
    approx(x2, 0.0, "child 2 x — floats do not place it beside child 1");
    approx(y2, 50.0, "child 2 y — it stacks below child 1");
    approx(w2, 100.0, "child 2 width");
    approx(h2, 50.0, "child 2 height");
    let (_, _, _, container_h) = geom(&rt, at(&rt, &[0]));
    approx(container_h, 100.0, "container content height");
}

/// §4.13 — "Excluded or restricted:" / "- float/clear;" plus §1.6 "diagnostics
/// include severity, subsystem, stable code, node/resource/window context and
/// recovery action".
///
/// NG-FLOAT-02.
#[test]
fn an_excluded_float_declaration_is_reported_and_its_siblings_still_apply() {
    let css = ".a{float:left;color:rgb(255,0,0)}";
    let reports = unsupported(css);
    assert_eq!(
        reports.len(),
        1,
        "exactly one diagnostic for the excluded `float` property, got {reports:?}"
    );
    let report = &reports[0];
    assert_eq!(report.kind(), UnsupportedFeatureKind::Property);
    assert_eq!(report.name(), "float");
    assert!(
        report.span().end() > report.span().start(),
        "the diagnostic carries the source position of `float`"
    );

    // The rest of the same declaration block still applies.
    let rt = drive(VW, VW, css, &view(vec![el(1, "a", vec![])]));
    assert_color(
        &rt,
        at(&rt, &[0]),
        255,
        0,
        0,
        "`color` in the same rule as an excluded property still computes",
    );
}

/// §4.1 — "Excluded legacy/browser baggage:" / "- table layout as a
/// general-purpose layout engine;".
///
/// NG-TABLE-01.
#[test]
fn display_table_cell_children_stack_and_do_not_form_a_row() {
    let rt = drive(
        VW,
        VW,
        ".box{width:300px}\
         .c{display:table-cell;height:40px}",
        &view(vec![el(
            1,
            "box",
            vec![el(2, "c", vec![]), el(3, "c", vec![])],
        )]),
    );
    let (x1, y1, w1, h1) = geom(&rt, at(&rt, &[0, 0]));
    let (x2, y2, w2, h2) = geom(&rt, at(&rt, &[0, 1]));
    approx(x1, 0.0, "cell 1 x");
    approx(y1, 0.0, "cell 1 y");
    approx(w1, 300.0, "cell 1 width — full container, not a shared row");
    approx(h1, 40.0, "cell 1 height");
    approx(x2, 0.0, "cell 2 x");
    approx(y2, 40.0, "cell 2 y — it stacks below, not beside");
    approx(w2, 300.0, "cell 2 width");
    approx(h2, 40.0, "cell 2 height");
    let (_, _, _, container_h) = geom(&rt, at(&rt, &[0]));
    approx(container_h, 80.0, "container content height");
}

/// §4.13 — "Excluded or restricted:" / "- columns;".
///
/// NG-COLUMNS-01.
#[test]
fn column_count_does_not_fragment_the_flow() {
    let rt = drive(
        VW,
        VW,
        ".box{width:300px;column-count:2;column-gap:20px}\
         .item{height:50px}",
        &view(vec![el(
            1,
            "box",
            vec![
                el(2, "item", vec![]),
                el(3, "item", vec![]),
                el(4, "item", vec![]),
                el(5, "item", vec![]),
            ],
        )]),
    );
    for (index, expected_y) in [0.0_f32, 50.0, 100.0, 150.0].into_iter().enumerate() {
        let (x, y, w, _) = geom(&rt, at(&rt, &[0, index]));
        approx(x, 0.0, "child x — single column flow");
        approx(
            y,
            expected_y,
            "child y — no column fragmentation, no gap leak",
        );
        approx(w, 300.0, "child width — the full container width");
    }
    let (_, _, _, container_h) = geom(&rt, at(&rt, &[0]));
    approx(container_h, 200.0, "container content height");
}

/// §4.13 — "Excluded or restricted:" / "- legacy clip/zoom/page-break features;".
///
/// NG-ZOOM-01.
#[test]
fn zoom_neither_scales_layout_nor_becomes_a_transform() {
    let rt = drive(
        VW,
        VW,
        ".box{width:400px}\
         .z{width:100px;height:100px;background-color:rgb(0,0,255);zoom:2}",
        &view(vec![el(1, "box", vec![el(2, "z", vec![])])]),
    );
    let (x, y, w, h) = geom(&rt, at(&rt, &[0, 0]));
    approx(x, 0.0, "border box x");
    approx(y, 0.0, "border box y");
    approx(w, 100.0, "border box width — `zoom` does not scale layout");
    approx(h, 100.0, "border box height");

    let painted: Vec<_> = rt
        .snapshot()
        .display_list()
        .commands()
        .iter()
        .filter_map(|c| match c {
            obscur_kit::scene::command::PaintCommand::PushTransform(t) => Some(*t),
            _ => None,
        })
        .collect();
    assert!(
        painted
            .iter()
            .all(|t| *t == obscur_kit::geometry::AffineTransform::IDENTITY),
        "`zoom` must not be aliased onto a transform: {painted:?}"
    );

    let (_, _, _, parent_h) = geom(&rt, at(&rt, &[0]));
    approx(parent_h, 100.0, "parent content height — not 200");

    let reports = unsupported(".z{zoom:2}");
    assert_eq!(
        reports.len(),
        1,
        "the excluded `zoom` property is reported, got {reports:?}"
    );
    assert_eq!(reports[0].name(), "zoom");
}

/// §4.13 — "Excluded or restricted:" / "- browser link/visited/target state;".
///
/// NG-LINKSTATE-01.
#[test]
fn visited_and_target_selectors_are_excluded_not_merely_inert() {
    let css = ".item:visited{color:rgb(255,0,0)}\
               .item:target{color:rgb(0,255,0)}\
               .item{color:rgb(0,0,255)}";
    let reports = unsupported(css);
    assert_eq!(
        reports.len(),
        2,
        "one diagnostic each for `:visited` and `:target`, got {reports:?}"
    );
    assert!(
        reports.iter().any(|r| r.name() == ":visited"),
        "a diagnostic names `:visited`: {reports:?}"
    );
    assert!(
        reports.iter().any(|r| r.name() == ":target"),
        "a diagnostic names `:target`: {reports:?}"
    );
    assert!(
        reports
            .iter()
            .all(|r| r.kind() == UnsupportedFeatureKind::Selector),
        "both are reported as unsupported *selectors*: {reports:?}"
    );

    let rt = drive(VW, VW, css, &view(vec![el(1, "item", vec![])]));
    assert_color(
        &rt,
        at(&rt, &[0]),
        0,
        0,
        255,
        "the plain `.item` rule in the same sheet still applies",
    );
}

/// §4.13 — "Excluded or restricted:" / "- arbitrary network loading via `url()`
/// and `@import`;" and §5.15 "The kit does not perform arbitrary network access
/// by itself."
///
/// NG-IMPORT-01.
#[test]
fn an_import_at_rule_is_rejected_and_the_rest_of_the_sheet_survives() {
    let css = "@import url(\"https://example.invalid/theme.css\");\n\
               .a{color:rgb(255,0,0)}";
    let reports = unsupported(css);
    assert_eq!(
        reports.len(),
        1,
        "exactly one diagnostic for the rejected `@import`, got {reports:?}"
    );
    assert_eq!(reports[0].name(), "@import");
    assert_eq!(reports[0].kind(), UnsupportedFeatureKind::AtRule);

    let rt = drive(VW, VW, css, &view(vec![el(1, "a", vec![])]));
    assert_color(
        &rt,
        at(&rt, &[0]),
        255,
        0,
        0,
        "an unknown at-rule is not fatal: `.a` still computes",
    );
}

/// §5.6 Offscreen output — "Paged document/print rendering is not a core goal."
/// and §4.13 "- print/page fragmentation;".
///
/// NG-PRINT-01 (the `page-break-after` half; the paged-export half is recorded as
/// unexpressible — the kit exposes no offscreen paged-export entry point at all).
#[test]
fn page_break_after_is_reported_and_moves_nothing() {
    let css = ".box{width:300px}\
               .a{height:500px;page-break-after:always}\
               .b{height:100px}";
    let reports = unsupported(css);
    assert_eq!(
        reports.len(),
        1,
        "the excluded `page-break-after` property is reported, got {reports:?}"
    );
    assert_eq!(reports[0].name(), "page-break-after");

    let rt = drive(
        VW,
        VW,
        css,
        &view(vec![el(
            1,
            "box",
            vec![el(2, "a", vec![]), el(3, "b", vec![])],
        )]),
    );
    let (_, y, _, _) = geom(&rt, at(&rt, &[0, 1]));
    approx(
        y,
        500.0,
        "the following child is not pushed to a page boundary",
    );
}
