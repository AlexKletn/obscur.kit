#![doc = include_str!("restyle_invalidation.doc/_module.md")]
#![forbid(unsafe_code)]

use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::geometry::Size;
use obscur_kit::identity::{ComponentId, NodeId, RuntimeId};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::view::view::{CLASS, View, ViewFragment, ViewNode, ViewProperty, ViewValue};

#[doc = include_str!("restyle_invalidation.doc/compile.md")]
fn compile(css: &str) -> Stylesheet {
    let src = CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&src).parse();
    StylesheetCompiler::new().compile(&parsed).matchable
}

#[doc = include_str!("restyle_invalidation.doc/row.md")]
fn row(id: u64) -> ViewNode {
    ViewNode::element(ComponentId::from_raw(id))
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("row".into())))
}

#[doc = include_str!("restyle_invalidation.doc/list.md")]
fn list(count: u64) -> View {
    let rows: Vec<ViewNode> = (0..count).map(|i| row(0x5000 + i)).collect();
    View::single(
        ViewNode::element(ComponentId::from_raw(0x4000))
            .with_prop(ViewProperty::new(CLASS, ViewValue::Text("list".into())))
            .with_children(ViewFragment::from_nodes(rows)),
    )
}

#[doc = include_str!("restyle_invalidation.doc/child.md")]
fn child(rt: &UiRuntime, index: usize) -> NodeId {
    let root = rt.tree().root().expect("a root after a frame");
    let list = rt.tree().get(root).expect("live").children()[0];
    rt.tree().get(list).expect("live").children()[index]
}

#[doc = include_str!("restyle_invalidation.doc/width.md")]
fn width(rt: &UiRuntime, node: NodeId) -> f32 {
    rt.layout_boxes()
        .get(&node)
        .expect("the node was arranged")
        .size()
        .width
}

#[test]
fn appending_a_row_restyles_the_sibling_that_lost_last_child() {
    // `:last-child` is 50px wide, every other row 10px — so the match is readable
    // as geometry.
    let sheet = compile(".row { width: 10px; height: 4px } .row:last-child { width: 50px }");
    let sheets = std::slice::from_ref(&sheet);
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(200.0, 200.0));

    // Two rows: the second one is `:last-child`.
    let _ = rt.render_now(&list(2), sheets);
    assert_eq!(
        width(&rt, child(&rt, 0)),
        10.0,
        "row 0 is not the last child"
    );
    assert_eq!(width(&rt, child(&rt, 1)), 50.0, "row 1 is the last child");

    // Append a third row. Row 1 is now an ordinary row and row 2 is the last —
    // and row 1 sits BEFORE the edit, so a forward-only invalidation misses it.
    let _ = rt.render_now(&list(3), sheets);
    assert_eq!(
        width(&rt, child(&rt, 1)),
        10.0,
        "row 1 lost `:last-child` and must have been restyled, even though the \
         structural edit happened after it"
    );
    assert_eq!(
        width(&rt, child(&rt, 2)),
        50.0,
        "row 2 is the new last child"
    );
}

#[test]
fn removing_a_row_restyles_the_sibling_that_became_last_child() {
    // The mirror case: a removal also shifts the backward count, and the node the
    // edit names is gone by the time the style phase runs.
    let sheet = compile(".row { width: 10px; height: 4px } .row:last-child { width: 50px }");
    let sheets = std::slice::from_ref(&sheet);
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(200.0, 200.0));

    let _ = rt.render_now(&list(3), sheets);
    assert_eq!(width(&rt, child(&rt, 2)), 50.0, "row 2 starts as the last");

    let _ = rt.render_now(&list(2), sheets);
    assert_eq!(
        width(&rt, child(&rt, 1)),
        50.0,
        "row 1 became `:last-child` when row 2 was removed, so it must restyle"
    );
}

#[test]
fn an_in_place_change_still_restyles_the_node_itself() {
    // The non-structural half: no sibling count moves, so the node's own subtree
    // (plus what follows it) is the whole reach — this must keep working.
    let sheet = compile(".row { width: 10px; height: 4px } .wide { width: 80px }");
    let sheets = std::slice::from_ref(&sheet);
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(200.0, 200.0));

    let _ = rt.render_now(&list(3), sheets);
    assert_eq!(width(&rt, child(&rt, 1)), 10.0, "row 1 starts narrow");

    // Same structure, one node's class changed in place.
    let mut rows: Vec<ViewNode> = (0..3).map(|i| row(0x5000 + i)).collect();
    rows[1] = ViewNode::element(ComponentId::from_raw(0x5001))
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("row wide".into())));
    let view = View::single(
        ViewNode::element(ComponentId::from_raw(0x4000))
            .with_prop(ViewProperty::new(CLASS, ViewValue::Text("list".into())))
            .with_children(ViewFragment::from_nodes(rows)),
    );
    let _ = rt.render_now(&view, sheets);
    assert_eq!(
        width(&rt, child(&rt, 1)),
        80.0,
        "the in-place class change restyled the node"
    );
}
