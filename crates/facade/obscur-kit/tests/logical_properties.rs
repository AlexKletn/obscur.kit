//! End-to-end: **logical box-model properties** authored in CSS reach layout, and
//! follow the computed `direction` (spec §4.13 "Supported: logical properties").
//!
//! `inline-start` is the LEFT edge under `ltr` and the RIGHT edge under `rtl` —
//! from one declaration. That is the whole reason the family exists, and it is
//! checked here against real arranged boxes rather than computed values, so a
//! mapping that resolved but never moved anything would fail.

#![forbid(unsafe_code)]

use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::geometry::Size;
use obscur_kit::identity::{ComponentId, NodeId, RuntimeId};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::view::view::{CLASS, View, ViewFragment, ViewNode, ViewProperty, ViewValue};

fn compile(css: &str) -> Stylesheet {
    let src = CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&src).parse();
    StylesheetCompiler::new().compile(&parsed).matchable
}

/// A `.page` box containing one `.item`.
fn view() -> View {
    let container = ComponentId::from_raw(0x00C0_0001);
    let item = ViewNode::element(container)
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("item".into())));
    let page = ViewNode::element(container)
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("page".into())))
        .with_children(ViewFragment::from_nodes(vec![item]));
    View::from_fragment(ViewFragment::from_nodes(vec![page]))
}

/// The `.item`'s arranged box origin after rendering `css`.
fn item_origin(css: &str) -> (f32, f32) {
    let sheet = compile(css);
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(300.0, 300.0));
    runtime.render_now(&view(), std::slice::from_ref(&sheet));

    let root = runtime.tree().root().expect("a root");
    let page = runtime.tree().get(root).expect("live").children()[0];
    let item: NodeId = runtime.tree().get(page).expect("live").children()[0];
    let layout = runtime.layout_boxes().get(&item).expect("the item is laid out");
    (layout.origin().x, layout.origin().y)
}

const PAGE: &str = ".page { width: 300px; height: 300px }";
const ITEM: &str = ".item { width: 50px; height: 50px }";

#[test]
fn padding_inline_start_is_the_left_edge_under_ltr() {
    // The parent's inline-start padding insets its child from the left.
    let (x, _) = item_origin(&format!(
        ".page {{ width: 300px; height: 300px; padding-inline-start: 40px }}\n{ITEM}"
    ));
    assert!((x - 40.0).abs() < 1e-3, "ltr: inline-start is the left edge, got x={x}");
}

#[test]
fn padding_inline_start_becomes_the_right_edge_under_rtl() {
    // The SAME declaration, with only `direction` added: the padding moves to the
    // other side, so the child is no longer inset from the left.
    let (rtl_x, _) = item_origin(&format!(
        ".page {{ width: 300px; height: 300px; direction: rtl; padding-inline-start: 40px }}\n{ITEM}"
    ));
    assert!(
        rtl_x.abs() < 1e-3,
        "rtl: inline-start is the RIGHT edge, so nothing insets the left, got x={rtl_x}"
    );
}

#[test]
fn padding_block_start_is_the_top_edge_in_either_direction() {
    // No writing-mode is modelled, so the block axis never flips.
    for direction in ["ltr", "rtl"] {
        let (_, y) = item_origin(&format!(
            ".page {{ width: 300px; height: 300px; direction: {direction}; \
              padding-block-start: 25px }}\n{ITEM}"
        ));
        assert!((y - 25.0).abs() < 1e-3, "{direction}: block-start is the top, got y={y}");
    }
}

#[test]
fn inset_inline_start_positions_from_the_leading_edge() {
    let (x, _) = item_origin(&format!(
        "{PAGE}\n.page {{ position: relative }}\n\
         .item {{ width: 50px; height: 50px; position: absolute; inset-inline-start: 40px }}"
    ));
    assert!((x - 40.0).abs() < 1e-3, "ltr: inset-inline-start is `left`, got x={x}");

    // Under rtl it is `right`: 300 − 40 − 50 = 210.
    let (rtl_x, _) = item_origin(&format!(
        "{PAGE}\n.page {{ position: relative; direction: rtl }}\n\
         .item {{ width: 50px; height: 50px; position: absolute; inset-inline-start: 40px }}"
    ));
    assert!(
        (rtl_x - 210.0).abs() < 1e-3,
        "rtl: inset-inline-start is `right`, got x={rtl_x}"
    );
}

#[test]
fn a_later_logical_declaration_beats_an_earlier_physical_one_end_to_end() {
    // The cascade rule, through the real parser: two forms of one box edge compete
    // by ordinary source order, not by which form was used.
    let (x, _) = item_origin(&format!(
        ".page {{ width: 300px; height: 300px; padding-left: 10px; \
          padding-inline-start: 40px }}\n{ITEM}"
    ));
    assert!((x - 40.0).abs() < 1e-3, "the later declaration wins, got x={x}");

    let (x, _) = item_origin(&format!(
        ".page {{ width: 300px; height: 300px; padding-inline-start: 40px; \
          padding-left: 10px }}\n{ITEM}"
    ));
    assert!((x - 10.0).abs() < 1e-3, "and the other way round, got x={x}");
}

#[test]
fn margin_inline_start_is_the_left_edge_under_ltr_and_the_right_under_rtl() {
    // The family's headline case, through the real parser and layout: ONE
    // declaration, two sides. (Horizontal margins reach flow layout as of the
    // §10.3.3 fix — before it this could not have been asserted at all.)
    let (ltr, _) = item_origin(&format!(
        "{PAGE}\n.item {{ width: 50px; height: 50px; margin-inline-start: 40px }}"
    ));
    assert!((ltr - 40.0).abs() < 1e-3, "ltr: inline-start insets from the left, got x={ltr}");

    let (rtl, _) = item_origin(&format!(
        ".page {{ width: 300px; height: 300px; direction: rtl }}\n\
         .item {{ width: 50px; height: 50px; margin-inline-start: 40px }}"
    ));
    assert!(
        rtl.abs() < 1e-3,
        "rtl: inline-start is the RIGHT edge, so nothing insets the left, got x={rtl}"
    );
}
