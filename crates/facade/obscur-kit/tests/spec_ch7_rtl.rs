#![doc = include_str!("spec_ch7_rtl.doc/_module.md")]
#![forbid(unsafe_code)]

#[path = "spec_ch7_support/mod.rs"]
mod support;

use support::{approx, at, drive, el, geom, view};

const VW: f32 = 1000.0;

#[doc = include_str!("spec_ch7_rtl.doc/child_left.md")]
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

#[doc = include_str!("spec_ch7_rtl.doc/assert_inline_start_pair.md")]
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

#[doc = include_str!("spec_ch7_rtl.doc/margin_inline_start_maps_to_the_right_edge_in_rtl.md")]
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

#[doc = include_str!("spec_ch7_rtl.doc/padding_inline_start_maps_to_the_right_edge_in_rtl.md")]
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

#[doc = include_str!("spec_ch7_rtl.doc/border_inline_start_width_maps_to_the_right_edge_in_rtl.md")]
#[test]
fn border_inline_start_width_maps_to_the_right_edge_in_rtl() {
    // CORRECTED 2026-07-20 by the owner: this expected 180 (= 300 - 20 - 100), the
    // MARGIN arithmetic copied onto a container BORDER. It is wrong under the default
    // `box-sizing: content-box`: the container's `width: 300` is the *content* width,
    // so the border sits OUTSIDE it and does not shrink the child's content area. The
    // child (100px) right-aligns to the 300px content edge → left = 200. (A child's
    // margin does enter the child's leftover, which is why the margin sibling is 180;
    // a container border under content-box does not.) The LTR value stays 20: the left
    // border shifts the content-box *origin*, which is a different effect.
    assert_inline_start_pair(
        "border-inline-start-width",
        "border-inline-start-style: solid; border-inline-start-width: 20px; \
         border-inline-end-width: 0",
        "",
        200.0,
        "the child right-aligns to the 300px content edge (border is outside it)",
    );
}

#[doc = include_str!("spec_ch7_rtl.doc/inset_inline_start_maps_to_the_right_edge_in_rtl.md")]
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
