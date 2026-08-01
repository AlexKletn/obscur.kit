#![doc = include_str!("percentage_in_flex_item.doc/_module.md")]
#![forbid(unsafe_code)]

use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::geometry::{Rect, Size};
use obscur_kit::identity::{ComponentId, NodeId, RuntimeId};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::view::view::{CLASS, View, ViewFragment, ViewNode, ViewProperty, ViewValue};

#[doc = include_str!("percentage_in_flex_item.doc/VIEWPORT.md")]
const VIEWPORT: Size = Size {
    width: 480.0,
    height: 240.0,
};

#[doc = include_str!("percentage_in_flex_item.doc/SHEET.md")]
const SHEET: &str = ".app { width: 400px; height: 200px; display: flex; \
                     align-items: flex-start } \
                     .item { width: 50%; display: flex; flex-wrap: wrap } \
                     .kid { width: 60px; height: 60px }";

fn compile(css: &str) -> Stylesheet {
    let src = CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&src).parse();
    StylesheetCompiler::new().compile(&parsed).matchable
}

fn el(class: &str, kids: Vec<ViewNode>) -> ViewNode {
    ViewNode::element(ComponentId::from_raw(82))
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text(class.into())))
        .with_children(ViewFragment::from_nodes(kids))
}

#[doc = include_str!("percentage_in_flex_item.doc/boxes.md")]
fn boxes() -> (Rect, Vec<Rect>) {
    let sheet = compile(SHEET);
    let view = View::from_fragment(ViewFragment::from_nodes(vec![el(
        "app",
        vec![el("item", vec![el("kid", vec![]), el("kid", vec![])])],
    )]));
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), VIEWPORT);
    let _ = rt.render_now(&view, std::slice::from_ref(&sheet));

    let root: NodeId = rt.tree().root().expect("a root after a frame");
    let app = rt.tree().get(root).expect("live").child_nodes()[0];
    let item = rt.tree().get(app).expect("live").child_nodes()[0];
    let kids: Vec<Rect> = rt
        .tree()
        .get(item)
        .expect("live")
        .child_nodes()
        .iter()
        .map(|kid| rt.layout_boxes().get(kid).expect("an arranged box").bounds)
        .collect();
    (
        rt.layout_boxes()
            .get(&item)
            .expect("an arranged box")
            .bounds,
        kids,
    )
}

#[doc = include_str!("percentage_in_flex_item.doc/a_percentage_item_measures_its_content_at_the_width_it_is_laid_out_at.md")]
#[test]
fn a_percentage_item_measures_its_content_at_the_width_it_is_laid_out_at() {
    let (item, _) = boxes();
    assert!((item.size.width - 200.0).abs() < 0.01, "item {item:?}");
    assert!((item.size.height - 60.0).abs() < 0.01, "item {item:?}");
}

#[doc = include_str!("percentage_in_flex_item.doc/the_items_arranged_content_agrees_with_the_height_it_reported.md")]
#[test]
fn the_items_arranged_content_agrees_with_the_height_it_reported() {
    let (item, kids) = boxes();
    assert_eq!(kids.len(), 2, "two children");
    assert!(
        (kids[0].origin.y - kids[1].origin.y).abs() < 0.01,
        "both boxes sit on one line: {kids:?}"
    );
    assert!(
        (kids[1].origin.x - kids[0].origin.x - 60.0).abs() < 0.01,
        "the second box follows the first along the line: {kids:?}"
    );
    assert!(
        (item.size.height - kids[0].size.height).abs() < 0.01,
        "the item is exactly as tall as the one line it holds: {item:?}"
    );
}
