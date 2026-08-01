#![doc = include_str!("percentage_in_padded_parent.doc/_module.md")]
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

#[doc = include_str!("percentage_in_padded_parent.doc/view.md")]
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

#[doc = include_str!("percentage_in_padded_parent.doc/a_percentage_resolves_against_the_content_box_at_every_depth.md")]
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

#[doc = include_str!("percentage_in_padded_parent.doc/a_fixed_width_parent_was_never_affected.md")]
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

#[doc = include_str!("percentage_in_padded_parent.doc/a_half_width_child_halves_the_content_box.md")]
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

#[doc = include_str!("percentage_in_padded_parent.doc/the_flex_path_resolves_a_percentage_against_the_content_box.md")]
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

#[doc = include_str!("percentage_in_padded_parent.doc/a_border_is_subtracted_from_the_basis_too.md")]
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
