//! End-to-end: **percentage margins** resolve against the containing block's
//! width on every edge (CSS 2.1 §8.3), and a flex item's **`auto` margins** take
//! the line's free space before `justify-content` sees it (CSS Flexbox §8.1).
//!
//! Both were silently inert: a `%` margin was folded to `0` by the CSS bridge, and
//! a flex `auto` margin was read as `0` by the algorithm.
//!
//! The container here is deliberately **not square** (300×100). A square one
//! cannot tell the §8.3 rule from the obvious-but-wrong reading — `margin-top:
//! 10%` is 30 either way — so every block-axis assertion below would have passed
//! against an implementation that resolved each edge against its own axis.

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

/// The `.item`'s arranged `(x, y, width)` under `page` / `item` declarations.
fn item_box(page_css: &str, item_css: &str) -> (f32, f32, f32) {
    let sheet = compile(&format!(".page {{ {page_css} }}\n.item {{ {item_css} }}"));
    let component = ComponentId::from_raw(0x00C0_0001);
    let item = ViewNode::element(component)
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("item".into())));
    let page = ViewNode::element(component)
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("page".into())))
        .with_children(ViewFragment::from_nodes(vec![item]));
    let view = View::from_fragment(ViewFragment::from_nodes(vec![page]));

    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(600.0, 600.0));
    runtime.render_now(&view, std::slice::from_ref(&sheet));
    let root = runtime.tree().root().expect("a root");
    let page_id = runtime.tree().get(root).expect("live").children()[0];
    let item_id: NodeId = runtime.tree().get(page_id).expect("live").children()[0];
    let layout = runtime.layout_boxes().get(&item_id).expect("laid out");
    (layout.origin().x, layout.origin().y, layout.size().width)
}

/// A 300×100 block container.
const BLOCK: &str = "width: 300px; height: 100px";

#[test]
fn a_percentage_inline_margin_resolves_against_the_containing_block_width() {
    let (x, _, width) = item_box(BLOCK, "margin-left: 10%; height: 10px");
    assert!((x - 30.0).abs() < 1e-3, "10% of 300 = 30, got {x}");
    assert!(
        (width - 270.0).abs() < 1e-3,
        "and the margin takes from the offer, so `auto` width fills 270, got {width}"
    );
}

#[test]
fn a_percentage_block_margin_resolves_against_the_width_not_the_height() {
    // THE §8.3 rule, and the reason this container is 300 wide and 100 tall.
    // Against the width: 10% of 300 = 30. Against its own axis (the obvious but
    // wrong reading): 10% of 100 = 10. Only one of those can pass.
    let (_, y, _) = item_box(BLOCK, "margin-top: 10%; height: 10px");
    assert!(
        (y - 30.0).abs() < 1e-3,
        "`margin-top: 10%` is 10% of the containing block's WIDTH (30), not its \
         height (10) — CSS 2.1 §8.3. Got y={y}"
    );
}

#[test]
fn the_margin_shorthand_resolves_every_edge_against_the_width() {
    // `margin: 10%` → 30 on all four edges, from the width alone.
    let (x, y, width) = item_box(BLOCK, "margin: 10%; height: 10px");
    assert!((x - 30.0).abs() < 1e-3, "got x={x}");
    assert!((y - 30.0).abs() < 1e-3, "got y={y}");
    assert!(
        (width - 240.0).abs() < 1e-3,
        "300 minus 30 of margin on each side, got {width}"
    );
}

#[test]
fn a_percentage_margin_offsets_a_definite_width_box() {
    let (x, _, width) = item_box(BLOCK, "width: 100px; margin-left: 50%; height: 10px");
    assert!((x - 150.0).abs() < 1e-3, "50% of 300 = 150, got {x}");
    assert!(
        (width - 100.0).abs() < 1e-3,
        "the box keeps its width, got {width}"
    );
}

#[test]
fn a_zero_percentage_and_a_pixel_margin_still_behave() {
    // Regression guard: percentages must not have disturbed the ordinary path.
    let (x, _, _) = item_box(BLOCK, "margin-left: 0%; height: 10px");
    assert!(x.abs() < 1e-3, "got {x}");
    let (px, _, _) = item_box(BLOCK, "margin-left: 40px; height: 10px");
    assert!((px - 40.0).abs() < 1e-3, "got {px}");
}

/// A 300-wide flex row.
const FLEX: &str = "display: flex; width: 300px; height: 50px";

#[test]
fn a_flex_items_auto_margin_pushes_it_to_the_end_of_the_line() {
    let (x, _, width) = item_box(FLEX, "width: 100px; height: 10px; margin-left: auto");
    assert!(
        (width - 100.0).abs() < 1e-3,
        "the item keeps its width, got {width}"
    );
    assert!(
        (x - 200.0).abs() < 1e-3,
        "`margin-left: auto` absorbs all 200 of free space, got {x}"
    );
}

#[test]
fn an_auto_margin_beats_justify_content() {
    // THE §8.1 ordering, and the whole reason the rule exists: the `auto` margin
    // consumes the free space FIRST, so `justify-content` has none left to place
    // with. `flex-start` would otherwise put this item at 0.
    let (x, _, _) = item_box(
        "display: flex; width: 300px; height: 50px; justify-content: flex-start",
        "width: 100px; height: 10px; margin-left: auto",
    );
    assert!(
        (x - 200.0).abs() < 1e-3,
        "the `auto` margin took the free space before `justify-content: flex-start` \
         could place the item at 0. Got x={x}"
    );
}

#[test]
fn two_auto_margins_split_the_free_space_and_centre_a_flex_item() {
    let (x, _, _) = item_box(FLEX, "width: 100px; height: 10px; margin: 0 auto");
    assert!(
        (x - 100.0).abs() < 1e-3,
        "200 of free space split between two `auto` edges centres at 100, got {x}"
    );
}

#[test]
fn justify_content_is_untouched_when_no_margin_is_auto() {
    // The regression guard that matters: `justify-content` is the common case and
    // must be byte-identical when nothing is `auto`.
    let (start, _, _) = item_box(FLEX, "width: 100px; height: 10px");
    assert!(start.abs() < 1e-3, "default flex-start, got {start}");

    let (centre, _, _) = item_box(
        "display: flex; width: 300px; height: 50px; justify-content: center",
        "width: 100px; height: 10px",
    );
    assert!((centre - 100.0).abs() < 1e-3, "got {centre}");

    let (end, _, _) = item_box(
        "display: flex; width: 300px; height: 50px; justify-content: flex-end",
        "width: 100px; height: 10px",
    );
    assert!((end - 200.0).abs() < 1e-3, "got {end}");
}

#[test]
fn an_auto_margin_with_no_free_space_reserves_nothing() {
    // The item already fills the line, so there is nothing for `auto` to absorb
    // and it must resolve to zero rather than to some fraction of a negative.
    let (x, _, width) = item_box(FLEX, "width: 300px; height: 10px; margin-left: auto");
    assert!(x.abs() < 1e-3, "no free space, so no offset, got {x}");
    assert!((width - 300.0).abs() < 1e-3, "got {width}");
}

#[test]
fn a_flex_items_percentage_margin_resolves_against_the_containers_width() {
    // Percentages resolve at extraction, before any of the flex arithmetic —
    // against the container's inline size, which for a row is also its main axis.
    let (x, _, _) = item_box(FLEX, "width: 100px; height: 10px; margin-left: 10%");
    assert!((x - 30.0).abs() < 1e-3, "10% of 300 = 30, got {x}");
}
