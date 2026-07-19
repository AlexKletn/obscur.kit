//! **SPECIFICATION-DERIVED** RTL cases for obscur.kit §7.3.
//!
//! The mapping is taken verbatim from `specs/w3c/css-writing-modes-4.txt` lines
//! 1510-1513: "inline-start / The side from which text of the inline base
//! direction would start. For boxes with a used direction value of ltr, this
//! means the line-left side. For boxes with a used direction value of rtl, this
//! means the line-right side."
//!
//! Written by an agent with **no access to this repository's source**. The four
//! cases below are the one derived case (RTL-01) applied to each of the four
//! logical properties it names; every expected number is the case's own
//! (`180` in rtl, `20` in ltr).
#![forbid(unsafe_code)]

#[path = "spec_ch7_support/mod.rs"]
mod support;

use support::{approx, at, drive, el, geom, view};

const VW: f32 = 1000.0;

/// The child's left edge, for a 300px content-box container in `direction` whose
/// styling adds `container_decl` and whose single 100px child adds `child_decl`.
fn child_left(direction: &str, container_decl: &str, child_decl: &str) -> f32 {
    let css = format!(
        ".c {{ width: 300px; height: 60px; box-sizing: content-box; \
               direction: {direction}; {container_decl} }} \
         .i {{ width: 100px; height: 20px; {child_decl} }}"
    );
    let v = view(vec![el(1, "c", vec![el(2, "i", vec![])])]);
    let rt = drive(VW, VW, &css, &v);
    geom(&rt, at(&rt, &[0, 0])).0
}

/// Asserts the RTL-01 pair: `180` under `direction: rtl`, `20` under
/// `direction: ltr`.
fn assert_inline_start_pair(what: &str, container_decl: &str, child_decl: &str) {
    approx(
        child_left("rtl", container_decl, child_decl),
        180.0,
        &format!("{what}, rtl: the child's left edge is at 300 - 20 - 100 = 180"),
    );
    approx(
        child_left("ltr", container_decl, child_decl),
        20.0,
        &format!("{what}, ltr: the child's left edge is at 20"),
    );
}

/// §7.3: "RTL behavior relies on:" / "- logical CSS properties;" /
/// "- layout-aware direction;".
#[test]
fn margin_inline_start_maps_to_the_right_edge_in_rtl() {
    assert_inline_start_pair(
        "margin-inline-start",
        "",
        "margin-inline-start: 20px; margin-inline-end: 0",
    );
}

/// §7.3: "- logical CSS properties;" — the same mapping for
/// `padding-inline-start`.
#[test]
fn padding_inline_start_maps_to_the_right_edge_in_rtl() {
    assert_inline_start_pair(
        "padding-inline-start",
        "padding-inline-start: 20px; padding-inline-end: 0",
        "",
    );
}

/// §7.3: "- logical CSS properties;" — the same mapping for
/// `border-inline-start-width`.
#[test]
fn border_inline_start_width_maps_to_the_right_edge_in_rtl() {
    assert_inline_start_pair(
        "border-inline-start-width",
        "border-inline-start-style: solid; border-inline-start-width: 20px; \
         border-inline-end-width: 0",
        "",
    );
}

/// §7.3: "- logical CSS properties;" — the same mapping for
/// `inset-inline-start`.
#[test]
fn inset_inline_start_maps_to_the_right_edge_in_rtl() {
    assert_inline_start_pair(
        "inset-inline-start",
        "",
        "position: relative; inset-inline-start: 20px",
    );
}
