#![doc = include_str!("overflow_axis_longhands.doc/_module.md")]
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

#[doc = include_str!("overflow_axis_longhands.doc/view.md")]
fn view() -> View {
    let rows: Vec<ViewNode> = (0..40).map(|i| el(0xB100 + i, "row", vec![])).collect();
    View::single(el(0xB000, "list", rows))
}

#[doc = include_str!("overflow_axis_longhands.doc/scrolled_by_a_wheel.md")]
fn scrolled_by_a_wheel(css: &str) -> (f32, bool) {
    let sheet = compile(css);
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(200.0, 100.0));
    let _ = rt.render_now(&view(), std::slice::from_ref(&sheet));

    let root = rt.tree().root().expect("a root");
    let list = rt.tree().get(root).expect("live").children()[0];
    let taken = rt.handle_scroll(40.0, Point::new(50.0, 10.0));
    (rt.scroll_top(list), taken == Some(list))
}

const ROWS: &str = " .row { width: 200px; height: 20px }";

#[test]
fn overflow_y_alone_scrolls_the_box() {
    // The declaration an author writes to scroll one direction. It used to name
    // no property at all — `overflow` was the only overflow id — so the whole
    // declaration was recorded as unsupported and the box never became a scroll
    // container: `handle_scroll` answered `None` over a list that visibly
    // overflowed.
    let (top, took_the_wheel) = scrolled_by_a_wheel(&format!(
        ".list {{ width: 200px; height: 100px; overflow-y: auto }}{ROWS}"
    ));
    assert!(
        took_the_wheel,
        "`overflow-y: auto` must make the box the wheel's target"
    );
    assert!(top > 0.0, "and the box must actually have moved");
}

#[test]
fn overflow_x_alone_carries_the_block_axis_with_it() {
    // css-overflow-3 §3: one axis stated as `hidden` computes an unstated
    // `visible` sibling to `auto`, which is why a box that only hides its
    // horizontal overflow still scrolls vertically everywhere else.
    let (top, took_the_wheel) = scrolled_by_a_wheel(&format!(
        ".list {{ width: 200px; height: 100px; overflow-x: hidden }}{ROWS}"
    ));
    assert!(
        took_the_wheel,
        "`overflow-x: hidden` leaves the block axis `auto`, which scrolls"
    );
    assert!(top > 0.0);
}

#[test]
fn the_shorthand_and_the_longhands_agree_and_cascade() {
    // The shorthand is now an expansion over the two longhands, so it must still
    // scroll on its own …
    let (shorthand_top, shorthand_took) = scrolled_by_a_wheel(&format!(
        ".list {{ width: 200px; height: 100px; overflow: auto }}{ROWS}"
    ));
    assert!(shorthand_took);
    assert!(shorthand_top > 0.0);

    // … and a longhand after it in the same block must win on that axis alone,
    // which is the whole point of storing the pair rather than the shorthand.
    let (top, took_the_wheel) = scrolled_by_a_wheel(&format!(
        ".list {{ width: 200px; height: 100px; overflow: hidden; overflow-y: auto }}{ROWS}"
    ));
    assert!(
        took_the_wheel,
        "the later `overflow-y: auto` must override the shorthand's block axis"
    );
    assert_eq!(top, shorthand_top);

    // The reverse order loses: the shorthand sets both axes, so it overrides the
    // earlier longhand and the box is `hidden` — a scroll container, but not one
    // the wheel may move.
    let (_, took_the_wheel) = scrolled_by_a_wheel(&format!(
        ".list {{ width: 200px; height: 100px; overflow-y: auto; overflow: hidden }}{ROWS}"
    ));
    assert!(
        !took_the_wheel,
        "a shorthand after a longhand resets both axes"
    );
}
