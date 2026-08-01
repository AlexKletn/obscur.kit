#![doc = include_str!("percentage_against_indefinite.doc/_module.md")]
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
    ViewNode::element(ComponentId::from_raw(83))
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text(class.into())))
        .with_children(ViewFragment::from_nodes(kids))
}

#[doc = include_str!("percentage_against_indefinite.doc/view.md")]
fn view() -> View {
    View::from_fragment(ViewFragment::from_nodes(vec![el(
        "outer",
        vec![el("first", vec![]), el("second", vec![])],
    )]))
}

#[doc = include_str!("percentage_against_indefinite.doc/sibling_heights.md")]
fn sibling_heights(css: &str) -> (f32, f32) {
    let sheet = compile(css);
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), VIEWPORT);
    let _ = rt.render_now(&view(), std::slice::from_ref(&sheet));

    let root = rt.tree().root().expect("a root after a frame");
    let outer = *rt
        .tree()
        .get(root)
        .expect("live")
        .child_nodes()
        .first()
        .expect("the .outer node");
    let kids = rt.tree().get(outer).expect("live").child_nodes().to_vec();
    let height = |node: NodeId| {
        rt.layout_boxes()
            .get(&node)
            .expect("an arranged box")
            .size()
            .height
    };
    (height(kids[0]), height(kids[1]))
}

#[doc = include_str!("percentage_against_indefinite.doc/a_definite_parent_still_resolves_both_siblings_against_itself.md")]
#[test]
fn a_definite_parent_still_resolves_both_siblings_against_itself() {
    let heights = sibling_heights(
        ".outer { width: 400px; height: 300px } \
         .first, .second { width: 100%; height: 50% }",
    );
    assert!(
        (heights.0 - 150.0).abs() < 0.01 && (heights.1 - 150.0).abs() < 0.01,
        "both 50% of 300: {heights:?}"
    );
}

#[doc = include_str!("percentage_against_indefinite.doc/an_indefinite_parent_makes_a_percentage_behave_as_auto.md")]
#[test]
fn an_indefinite_parent_makes_a_percentage_behave_as_auto() {
    let heights = sibling_heights(
        ".outer { width: 400px } \
         .first, .second { width: 100%; height: 50% }",
    );
    assert!(
        heights.0.abs() < 0.01 && heights.1.abs() < 0.01,
        "both behave as auto over empty content: {heights:?}"
    );
}

#[doc = include_str!("percentage_against_indefinite.doc/a_percentage_and_auto_agree_under_an_indefinite_parent.md")]
#[test]
fn a_percentage_and_auto_agree_under_an_indefinite_parent() {
    let as_percent = sibling_heights(
        ".outer { width: 400px } \
         .first, .second { width: 100%; height: 50%; min-height: 40px }",
    );
    let as_auto = sibling_heights(
        ".outer { width: 400px } \
         .first, .second { width: 100%; height: auto; min-height: 40px }",
    );
    assert_eq!(as_percent, as_auto, "the two spellings must agree");
    assert!(
        (as_percent.0 - 40.0).abs() < 0.01 && (as_percent.1 - 40.0).abs() < 0.01,
        "and the agreed answer is the floor, not the offer: {as_percent:?}"
    );
}
