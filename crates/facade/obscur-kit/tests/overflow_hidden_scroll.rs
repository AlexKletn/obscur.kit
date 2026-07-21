//! `overflow: hidden` clips, but the user cannot scroll it.
//!
//! CSS Overflow 3: `hidden` DOES establish a scroll container — it clips, it has a
//! scrollport, and it can be scrolled programmatically. What it must not do is answer
//! the user: no scrollbar, and the wheel passes through to whatever can scroll behind
//! it. Conflating the two turns every clipping box into a wheel target, so a row that
//! merely clips a long file name swallows the scroll meant for the list around it.
#![forbid(unsafe_code)]

use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::geometry::{Point, Size};
use obscur_kit::identity::{ComponentId, RuntimeId};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::view::view::{CLASS, View, ViewFragment, ViewNode, ViewProperty, ViewValue};

fn compile(css: &str) -> Stylesheet {
    let src = CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&src).parse();
    StylesheetCompiler::new().compile(&parsed).matchable
}

fn el(id: u64, class: &str, children: Vec<ViewNode>) -> ViewNode {
    ViewNode::element(ComponentId::from_raw(id))
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text(class.into())))
        .with_children(ViewFragment::from_nodes(children))
}

/// A scrollable outer list whose rows each clip with `overflow: hidden`.
fn view() -> View {
    let rows: Vec<ViewNode> = (0..40)
        .map(|i| el(0xA100 + i, "row", vec![el(0xA200 + i, "wide", vec![])]))
        .collect();
    View::single(el(0xA000, "list", rows))
}

const CSS: &str = ".list { width: 200px; height: 100px; overflow: scroll }\
 .row { width: 200px; height: 20px; overflow: hidden }\
 .wide { width: 200px; height: 200px }";

#[test]
fn the_wheel_passes_through_a_hidden_box_to_the_scroller_behind_it() {
    let sheet = compile(CSS);
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(200.0, 100.0));
    let _ = rt.render_now(&view(), std::slice::from_ref(&sheet));

    // A point over the first row — which clips, so it is a scroll container in the
    // CSS sense, but must not take the wheel.
    let target = rt.handle_scroll(40.0, Point::new(50.0, 10.0));

    let root = rt.tree().root().expect("a root");
    let list = rt.tree().get(root).expect("live").children()[0];
    assert_eq!(
        target,
        Some(list),
        "the wheel over a `hidden` row must scroll the `scroll` list behind it"
    );
    assert!(
        rt.scroll_top(list) > 0.0,
        "and the list must actually have moved"
    );
}

#[test]
fn a_hidden_box_still_clips() {
    // The other half: refusing the wheel must not make `hidden` stop clipping.
    let sheet = compile(CSS);
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(200.0, 100.0));
    let _ = rt.render_now(&view(), std::slice::from_ref(&sheet));

    let root = rt.tree().root().expect("a root");
    let list = rt.tree().get(root).expect("live").children()[0];
    let row = rt.tree().get(list).expect("live").children()[0];
    let wide = rt.tree().get(row).expect("live").children()[0];

    let row_box = rt.layout_boxes().get(&row).expect("arranged");
    let wide_box = rt.layout_boxes().get(&wide).expect("arranged");
    assert!(
        wide_box.size().height > row_box.size().height,
        "the child really does overflow its clipping parent vertically — which is \
         what would make the row a scroll candidate at all"
    );
}
