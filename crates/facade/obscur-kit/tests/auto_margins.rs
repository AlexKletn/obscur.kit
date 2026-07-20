//! End-to-end: **`auto` margins** absorb leftover inline space, and the
//! **logical axis shorthands** reach layout (spec §4.2, §4.13, CSS 2.1 §10.3.3).
//!
//! `margin: 0 auto` is the single most-written centring idiom in CSS, and it used
//! to do nothing here: the layout model stored margins as four `f32`s, so `auto`
//! was folded to `0` before layout ever saw it. It cannot be folded early —
//! `auto` means *"take what is left over"*, and the leftover is not known until
//! the box has been measured.
//!
//! Everything below asserts a real arranged box. The container is 300px wide and
//! the box 100px, so a centred box sits at exactly 100 — a number that is wrong
//! for every other outcome (0 if `auto` were dropped, 200 if the edges were
//! swapped).

#![forbid(unsafe_code)]

use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::geometry::Size;
use obscur_kit::identity::{ComponentId, NodeId, RuntimeId};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::view::view::{CLASS, View, ViewFragment, ViewNode, ViewProperty, ViewValue};

/// The page is 300 wide; a 100-wide box centres at 100.
const PAGE: f32 = 300.0;
const BOX: f32 = 100.0;
const CENTRED: f32 = (PAGE - BOX) / 2.0;

fn compile(css: &str) -> Stylesheet {
    let src = CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&src).parse();
    StylesheetCompiler::new().compile(&parsed).matchable
}

/// The `.item`'s arranged `(x, width)` under an `.item` declaration block.
fn item_box(item_css: &str) -> (f32, f32) {
    let sheet = compile(&format!(
        ".page {{ width: {PAGE}px; height: 300px; font-size: 16px }}\n.item {{ {item_css} }}"
    ));
    let component = ComponentId::from_raw(0x00C0_0001);
    let item = ViewNode::element(component)
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("item".into())));
    let page = ViewNode::element(component)
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("page".into())))
        .with_children(ViewFragment::from_nodes(vec![item]));
    let view = View::from_fragment(ViewFragment::from_nodes(vec![page]));

    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(PAGE, PAGE));
    runtime.render_now(&view, std::slice::from_ref(&sheet));
    let root = runtime.tree().root().expect("a root");
    let page_id = runtime.tree().get(root).expect("live").child_nodes()[0];
    let item_id: NodeId = runtime.tree().get(page_id).expect("live").child_nodes()[0];
    let layout = runtime.layout_boxes().get(&item_id).expect("laid out");
    (layout.origin().x, layout.size().width)
}

#[test]
fn margin_zero_auto_centres_a_definite_width_box() {
    // THE idiom.
    let (x, width) = item_box("width: 100px; height: 10px; margin: 0 auto");
    assert!(
        (width - BOX).abs() < 1e-3,
        "the box keeps its width, got {width}"
    );
    assert!(
        (x - CENTRED).abs() < 1e-3,
        "a 100px box in a 300px container centres at {CENTRED}, got {x}"
    );
}

#[test]
fn the_longhand_form_centres_identically() {
    // `margin: 0 auto` is a shorthand; the longhands must agree with it, or the
    // shorthand expansion is where the bug is.
    let (shorthand, _) = item_box("width: 100px; height: 10px; margin: 0 auto");
    let (longhand, _) =
        item_box("width: 100px; height: 10px; margin-left: auto; margin-right: auto");
    assert!(
        (shorthand - longhand).abs() < 1e-3,
        "{shorthand} vs {longhand}"
    );
    assert!((longhand - CENTRED).abs() < 1e-3);
}

#[test]
fn a_single_auto_edge_takes_the_whole_leftover() {
    // `margin-left: auto` pushes the box to the far side; `margin-right: auto`
    // leaves it where it was. The pair is asymmetric, which is what proves the
    // leftover is being given to ONE edge rather than split.
    let (left_auto, _) = item_box("width: 100px; height: 10px; margin-left: auto");
    assert!(
        (left_auto - (PAGE - BOX)).abs() < 1e-3,
        "`margin-left: auto` pushes right to {}, got {left_auto}",
        PAGE - BOX
    );

    let (right_auto, _) = item_box("width: 100px; height: 10px; margin-right: auto");
    assert!(
        right_auto.abs() < 1e-3,
        "`margin-right: auto` leaves the box at the start, got {right_auto}"
    );
}

#[test]
fn an_auto_margin_on_an_auto_width_box_resolves_to_zero() {
    // CSS 2.1 §10.3.3: a `width: auto` box has already taken the space, so there
    // is no leftover and its `auto` margins are zero. It fills, it does not centre
    // — a centring implementation that forgot this would halve the box's position.
    let (x, width) = item_box("height: 10px; margin: 0 auto");
    assert!(
        (width - PAGE).abs() < 1e-3,
        "it still fills the container, got {width}"
    );
    assert!(x.abs() < 1e-3, "and starts at the left edge, got {x}");
}

#[test]
fn fixed_margins_are_untouched_by_any_of_this() {
    // The regression guard: an ordinary fixed margin is the common case.
    let (x, width) = item_box("width: 100px; height: 10px; margin-left: 40px");
    assert!((x - 40.0).abs() < 1e-3, "got {x}");
    assert!((width - BOX).abs() < 1e-3, "got {width}");

    // And no margin at all still starts at the content edge.
    let (plain, _) = item_box("width: 100px; height: 10px");
    assert!(plain.abs() < 1e-3, "got {plain}");
}

#[test]
fn an_auto_margin_composes_with_a_fixed_one_on_the_other_edge() {
    // `margin-left: auto; margin-right: 20px` → the box sits 20 from the right,
    // i.e. the fixed edge is reserved first and `auto` takes what remains.
    let (x, _) = item_box("width: 100px; height: 10px; margin-left: auto; margin-right: 20px");
    let expected = PAGE - BOX - 20.0;
    assert!(
        (x - expected).abs() < 1e-3,
        "the fixed edge reserves its 20 and `auto` takes the rest: want {expected}, got {x}"
    );
}

#[test]
fn margin_inline_auto_centres_through_the_logical_shorthand() {
    // Both features of this change at once: the `margin-inline` shorthand expands
    // to the two logical longhands, which map to the physical edges, which are
    // `auto`, which centre.
    let (x, _) = item_box("width: 100px; height: 10px; margin-inline: auto");
    assert!(
        (x - CENTRED).abs() < 1e-3,
        "`margin-inline: auto` must centre exactly as `margin: 0 auto` does, got {x}"
    );
}

#[test]
fn the_logical_axis_shorthands_reach_layout_with_two_values() {
    // `margin-inline: <start> <end>` — under ltr, start is the left edge.
    let (x, _) = item_box("width: 100px; height: 10px; margin-inline: 30px 5px");
    assert!(
        (x - 30.0).abs() < 1e-3,
        "start is the left edge under ltr, got {x}"
    );
}

#[test]
fn padding_block_shorthand_insets_both_block_edges() {
    // A different axis and a different property, to catch a table that sent every
    // logical shorthand to the same pair.
    let sheet = compile(
        ".page { width: 300px; height: 300px; padding-block: 25px }\n\
         .item { width: 10px; height: 10px }",
    );
    let component = ComponentId::from_raw(0x00C0_0001);
    let item = ViewNode::element(component)
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("item".into())));
    let page = ViewNode::element(component)
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("page".into())))
        .with_children(ViewFragment::from_nodes(vec![item]));
    let view = View::from_fragment(ViewFragment::from_nodes(vec![page]));

    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(300.0, 300.0));
    runtime.render_now(&view, std::slice::from_ref(&sheet));
    let root = runtime.tree().root().expect("a root");
    let page_id = runtime.tree().get(root).expect("live").child_nodes()[0];
    let item_id: NodeId = runtime.tree().get(page_id).expect("live").child_nodes()[0];
    let origin = runtime
        .layout_boxes()
        .get(&item_id)
        .expect("laid out")
        .origin();

    assert!(
        (origin.y - 25.0).abs() < 1e-3,
        "`padding-block` insets the TOP edge, got y={}",
        origin.y
    );
    assert!(
        origin.x.abs() < 1e-3,
        "and leaves the inline axis alone, got x={}",
        origin.x
    );
}

#[test]
fn fit_content_with_auto_margins_centres_a_shrink_wrapped_box() {
    // The two most recent pieces of §4 meeting: `fit-content` shrinks the box to
    // its content, and `auto` margins centre whatever that turned out to be. The
    // width is not authored anywhere — if intrinsic sizing and the margin solve
    // disagreed about what the used width is, this lands off-centre.
    let sheet = compile(
        ".page { width: 300px; height: 300px; font-size: 16px }\n\
         .item { width: fit-content; height: 10px; margin: 0 auto }",
    );
    let component = ComponentId::from_raw(0x00C0_0001);
    let item = ViewNode::element(component)
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("item".into())))
        .with_children(ViewFragment::from_nodes(vec![ViewNode::text("hi")]));
    let page = ViewNode::element(component)
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("page".into())))
        .with_children(ViewFragment::from_nodes(vec![item]));
    let view = View::from_fragment(ViewFragment::from_nodes(vec![page]));

    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(300.0, 300.0));
    runtime.render_now(&view, std::slice::from_ref(&sheet));
    let root = runtime.tree().root().expect("a root");
    let page_id = runtime.tree().get(root).expect("live").child_nodes()[0];
    let item_id: NodeId = runtime.tree().get(page_id).expect("live").child_nodes()[0];
    let layout = runtime.layout_boxes().get(&item_id).expect("laid out");
    let (x, width) = (layout.origin().x, layout.size().width);

    // "hi" is 2 glyphs at 0.6 x 16px.
    assert!(
        (width - 19.2).abs() < 1e-3,
        "shrink-wrapped to its text, got {width}"
    );
    assert!(
        (x - (PAGE - width) / 2.0).abs() < 1e-3,
        "centred on its shrink-wrapped width: want {}, got {x}",
        (PAGE - width) / 2.0
    );
}
