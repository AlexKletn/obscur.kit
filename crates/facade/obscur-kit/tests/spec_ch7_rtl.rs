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

/// Asserts the RTL-01 pair: `rtl_expected` under `direction: rtl`, `20` under
/// `direction: ltr`.
fn assert_inline_start_pair(
    what: &str,
    container_decl: &str,
    child_decl: &str,
    rtl_expected: f32,
    rtl_why: &str,
) {
    approx(
        child_left("rtl", container_decl, child_decl),
        rtl_expected,
        &format!("{what}, rtl: the child's left edge is at {rtl_why}"),
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
        180.0,
        "300 - 20 - 100 = 180",
    );
}

/// §7.3: "- logical CSS properties;" — the same mapping for
/// `padding-inline-start`.
///
/// EXPECTED VALUE REPAIRED 2026-07-19: this asserted `180` in rtl, copied from
/// the sibling margin case, and 180 is arithmetically impossible here. The
/// padding is declared on the CONTAINER, which is `box-sizing: content-box;
/// width: 300px` — padding sits OUTSIDE that 300px content box and does not
/// shrink it. In rtl, `padding-inline-start` maps to `padding-right`
/// (css-writing-modes-4.txt:1510-1513), so the content box is still x ∈ [0, 300]
/// and a 100px child flush against its line-right edge has its left edge at
/// **200**, not 180. The ltr half (20) was and is correct: `padding-left: 20px`
/// starts the content box at 20.
///
/// The case is expected to keep failing, but now for one honest reason instead
/// of two: measured 0, i.e. ltr placement. That is the same defect the register
/// already carries as `margin_inline_start_maps_to_the_right_edge_in_rtl` —
/// block-level over-constrained resolution ignores `direction` — and this case
/// is a second symptom of it, not a second gap.
#[test]
fn padding_inline_start_maps_to_the_right_edge_in_rtl() {
    assert_inline_start_pair(
        "padding-inline-start",
        "padding-inline-start: 20px; padding-inline-end: 0",
        "",
        200.0,
        "the line-right edge of the unshrunk 300px content box, 300 - 100 = 200",
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
        180.0,
        "300 - 20 - 100 = 180",
    );
}

/// §7.3: "- logical CSS properties;" — the same mapping for
/// `inset-inline-start`.
///
/// TRIAGE OVERRULED 2026-07-19, and left failing. Triage held that the measured
/// `-20` is the correct answer and the expected `180` wrong. It is not. Two
/// steps compose here and only the second is right.
///
/// Step 1, the static position: a 100px block child in a 300px `direction: rtl`
/// content box sits flush against the line-right edge, at x = 200. This runtime
/// puts it at 0.
///
/// Step 2, the relative shift: in rtl `inset-inline-start` maps to `right`, and
/// `left` is auto, so by css-position-3.txt:792-795 ("If only one is auto, its
/// used value becomes the negation of the other, and the box is shifted by the
/// specified amount") the used `left` is -20 — the box moves 20px left of its
/// static position. That step is implemented correctly, which is what triage
/// observed.
///
/// 200 - 20 = **180**, exactly the expected value. The measured -20 is
/// 0 - 20: correct relative positioning applied to a wrong static position. The
/// case is a second symptom of the same registered defect as
/// `margin_inline_start_maps_to_the_right_edge_in_rtl`, not a bad test, so
/// nothing here is weakened.
#[test]
fn inset_inline_start_maps_to_the_right_edge_in_rtl() {
    assert_inline_start_pair(
        "inset-inline-start",
        "",
        "position: relative; inset-inline-start: 20px",
        180.0,
        "the static 200 shifted 20px line-inward: 200 - 20 = 180",
    );
}
