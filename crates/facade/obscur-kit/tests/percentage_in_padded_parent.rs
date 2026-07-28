//! A percentage size resolves against its **containing block's content box** —
//! including inside a parent whose own width is a percentage (CSS 2.1 §10.2).
//!
//! The defect this pins: a box arranged at a decided size re-read its own
//! authored `width: 100%` and resolved it against the constraint it had just
//! been handed — its own border box — so it came back one padding wider, and the
//! error compounded one nesting level at a time until the content walked out of
//! the window. A fixed `px` width resolves to the same number against any basis,
//! which is exactly why the bug hid: only percentages diverged, and only under a
//! padded percentage-width ancestor.
//!
//! The fix states the containing block explicitly (`BoxConstraints::percent_basis`)
//! rather than letting a decided box re-derive it — Servo's
//! `SizeConstraint::Definite`, in its first half.
#![forbid(unsafe_code)]

use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::geometry::Size;
use obscur_kit::identity::{ComponentId, NodeId, RuntimeId};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::view::view::{CLASS, View, ViewFragment, ViewNode, ViewProperty, ViewValue};

const VIEWPORT: Size = Size {
    width: 1000.0,
    height: 600.0,
};

fn compile(css: &str) -> Stylesheet {
    let src = CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&src).parse();
    StylesheetCompiler::new().compile(&parsed).matchable
}

fn el(class: &str, kids: Vec<ViewNode>) -> ViewNode {
    ViewNode::element(ComponentId::from_raw(80))
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text(class.into())))
        .with_children(ViewFragment::from_nodes(kids))
}

/// `outer > inner > deep`, so a compounding error shows up as growth per level.
fn view() -> View {
    View::from_fragment(ViewFragment::from_nodes(vec![el(
        "outer",
        vec![el("inner", vec![el("deep", vec![])])],
    )]))
}

fn widths(css: &str) -> Vec<f32> {
    let sheet = compile(css);
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), VIEWPORT);
    let _ = rt.render_now(&view(), std::slice::from_ref(&sheet));

    let mut out = Vec::new();
    let mut node: NodeId = rt.tree().root().expect("a root after a frame");
    loop {
        let kids = rt.tree().get(node).expect("live").child_nodes().to_vec();
        let Some(&child) = kids.first() else { break };
        out.push(
            rt.layout_boxes()
                .get(&child)
                .expect("an arranged box")
                .size()
                .width,
        );
        node = child;
    }
    out
}

/// A percentage child of a padded percentage parent gets the parent's **content**
/// width, and a third level gets the second's — no growth per level.
#[test]
fn a_percentage_resolves_against_the_content_box_at_every_depth() {
    let widths = widths(
        ".outer { width: 100%; padding-left: 20px; padding-right: 20px; height: 400px } \
         .inner { width: 100%; padding-left: 10px; padding-right: 10px; height: 200px } \
         .deep  { width: 100%; height: 50px }",
    );

    // `.outer`: 100% of the 1000 viewport is its *content* width (content-box
    // sizing), so its border box is 1040 — it overflows, which is what CSS says a
    // padded `width: 100%` does, and what the box must keep doing.
    assert!((widths[0] - 1040.0).abs() < 0.01, "outer {}", widths[0]);
    // `.inner`: 100% of `.outer`'s content box (1000) plus its own 20 of padding.
    // Before the fix it read 1040 and returned 1060.
    assert!((widths[1] - 1020.0).abs() < 0.01, "inner {}", widths[1]);
    // `.deep`: 100% of `.inner`'s content box (1000). Before the fix, 1060.
    assert!((widths[2] - 1000.0).abs() < 0.01, "deep {}", widths[2]);
}

/// The same shape with a **fixed** parent width, which was always correct. It is
/// here because it is the reason the bug survived: whichever basis a `px` width
/// is resolved against it answers the same, so only percentages ever diverged.
#[test]
fn a_fixed_width_parent_was_never_affected() {
    let widths = widths(
        ".outer { width: 800px; padding-left: 20px; padding-right: 20px; height: 400px } \
         .inner { width: 100%; padding-left: 10px; padding-right: 10px; height: 200px } \
         .deep  { width: 100%; height: 50px }",
    );

    assert!((widths[0] - 840.0).abs() < 0.01, "outer {}", widths[0]);
    assert!((widths[1] - 820.0).abs() < 0.01, "inner {}", widths[1]);
    assert!((widths[2] - 800.0).abs() < 0.01, "deep {}", widths[2]);
}

/// A fractional percentage takes its share of the content box, where an
/// off-by-a-padding is easier to mistake for rounding.
#[test]
fn a_half_width_child_halves_the_content_box() {
    let widths = widths(
        ".outer { width: 100%; padding-left: 50px; padding-right: 50px; height: 400px } \
         .inner { width: 50%; height: 200px } \
         .deep  { width: 100%; height: 50px }",
    );

    assert!((widths[0] - 1100.0).abs() < 0.01, "outer {}", widths[0]);
    assert!((widths[1] - 500.0).abs() < 0.01, "inner {}", widths[1]);
    assert!((widths[2] - 500.0).abs() < 0.01, "deep {}", widths[2]);
}

/// Flex has its own sizing code and had the same defect. Measured on a
/// **fixed-width** flex parent, which isolates the percentage rule from a
/// separate flex bug: a flex container clamps its own border box to the space
/// available instead of overflowing it, so `width: 100%` + padding comes out
/// 1000 where the block path correctly gives 1040. That is not this fix's
/// business and is deliberately not exercised here.
#[test]
fn the_flex_path_resolves_a_percentage_against_the_content_box() {
    let widths = widths(
        ".outer { width: 800px; padding-left: 20px; padding-right: 20px; height: 400px; \
                  display: flex; flex-direction: column } \
         .inner { width: 100%; padding-left: 10px; padding-right: 10px; height: 200px; \
                  display: flex; flex-direction: column } \
         .deep  { width: 100%; height: 50px }",
    );

    assert!((widths[0] - 840.0).abs() < 0.01, "outer {}", widths[0]);
    assert!((widths[1] - 820.0).abs() < 0.01, "inner {}", widths[1]);
    assert!((widths[2] - 800.0).abs() < 0.01, "deep {}", widths[2]);
}

/// Borders count toward the content box exactly as padding does.
#[test]
fn a_border_is_subtracted_from_the_basis_too() {
    let widths = widths(
        ".outer { width: 100%; border-left-width: 15px; border-right-width: 15px; \
                  border-style: solid; height: 400px } \
         .inner { width: 100%; height: 200px } \
         .deep  { width: 100%; height: 50px }",
    );

    assert!((widths[0] - 1030.0).abs() < 0.01, "outer {}", widths[0]);
    assert!((widths[1] - 1000.0).abs() < 0.01, "inner {}", widths[1]);
    assert!((widths[2] - 1000.0).abs() < 0.01, "deep {}", widths[2]);
}
