//! End-to-end verification that **authored layout CSS** reaches the layout
//! engine and produces the expected [`LayoutBox`](obscur_kit::layout::model::LayoutBox)
//! geometry through the *real* live pipeline (spec §4.1/§4.2/§4.6).
//!
//! Every test drives the genuine path — author CSS text, compile it through the
//! real `obscur-css` frontend (`source -> parse -> compile -> matchable
//! Stylesheet`), and render one frame on a [`UiRuntime`] via
//! [`render_now`](UiRuntime::render_now), which runs `reconcile -> match ->
//! cascade -> resolve (ComputedStyles) -> layout (LayoutBoxes)`. The assertions
//! read the *actual* absolute [`LayoutBox`] coordinates the pipeline produced
//! (`runtime.layout_boxes()`), never a hand-built `ComputedStyle`/`LayoutStyle`,
//! so a regression in the resolver, the computed→layout bridge, or the flex/flow
//! algorithm fails the run.
//!
//! The container is laid out under a fixed 300x300 viewport, so all numbers are
//! deterministic. (A flex container's own explicit `width`/`height` is currently
//! stretched to the block containing block by the Phase-1 flow layout, so the
//! container content extent used below is the 300x300 viewport; percentages,
//! justification, and clamping are all verified against that extent.)
#![forbid(unsafe_code)]

use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::geometry::Size;
use obscur_kit::identity::{ComponentId, NodeId, RuntimeId};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::view::view::{CLASS, View, ViewFragment, ViewNode, ViewProperty, ViewValue};

/// The fixed viewport every test renders under, so geometry is deterministic.
const VIEWPORT: f32 = 300.0;

/// Compiles authored CSS text into a runtime-matchable [`Stylesheet`] through
/// the real CSS frontend (`source -> parse -> compile`).
fn compile(css: &str) -> Stylesheet {
    let src = CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&src).parse();
    StylesheetCompiler::new().compile(&parsed).matchable
}

/// Tags a view node with a `class` attribute so authored selectors match it.
fn tag(node: ViewNode, cls: &str) -> ViewNode {
    node.with_prop(ViewProperty::new(CLASS, ViewValue::Text(cls.into())))
}

fn el(id: u64) -> ViewNode {
    ViewNode::element(ComponentId::from_raw(id))
}

/// A `.row` container holding two children classed `.a` and `.b`.
fn two_child_view() -> View {
    let a = tag(el(21), "a");
    let b = tag(el(22), "b");
    let container = tag(el(20), "row").with_children(ViewFragment::from_nodes(vec![a, b]));
    View::from_fragment(ViewFragment::from_nodes(vec![container]))
}

/// Renders one real frame for `css` + `view` and returns the driven runtime, so
/// the caller can read the produced layout boxes.
fn drive(css: &str, view: &View) -> UiRuntime {
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(VIEWPORT, VIEWPORT));
    let sheet = compile(css);
    let result = rt.render_now(view, std::slice::from_ref(&sheet));
    assert!(result.restyled(), "authored CSS resolved computed styles");
    assert!(result.relaid_out(), "the styled tree arranged layout boxes");
    rt
}

/// The `.a` / `.b` child [`NodeId`]s of the single `.row` container, in order.
fn children(rt: &UiRuntime) -> (NodeId, NodeId) {
    let root = rt.tree().root().expect("root after a frame");
    let container = rt.tree().get(root).unwrap().children().to_vec()[0];
    let kids = rt.tree().get(container).unwrap().children().to_vec();
    (kids[0], kids[1])
}

/// The absolute origin `(x, y)` and size `(w, h)` of a node's layout box.
fn geom(rt: &UiRuntime, node: NodeId) -> (f32, f32, f32, f32) {
    let b = rt.layout_boxes().get(&node).copied().expect("node has a box");
    (b.origin().x, b.origin().y, b.size().width, b.size().height)
}

fn approx(actual: f32, expected: f32, what: &str) {
    assert!(
        (actual - expected).abs() < 1e-3,
        "{what}: expected {expected}, got {actual}"
    );
}

// 1. flex row + gap: `.a` at x=0 w=20, `.b` at x=30 (=20+gap) w=30, same y.
#[test]
fn flex_row_with_gap_places_items_along_x() {
    let rt = drive(
        ".row{display:flex;flex-direction:row;gap:10px} \
         .a{width:20px;height:8px} .b{width:30px;height:8px}",
        &two_child_view(),
    );
    let (a, b) = children(&rt);
    let (ax, ay, aw, _) = geom(&rt, a);
    let (bx, by, bw, _) = geom(&rt, b);
    approx(ax, 0.0, "a.x");
    approx(aw, 20.0, "a.width");
    approx(bx, 30.0, "b.x (20 + 10 gap)");
    approx(bw, 30.0, "b.width");
    approx(ay, by, "a and b share a y");
    approx(ay, 0.0, "row items start at y=0");
}

// 2. flex column: children stack vertically; `.b` below `.a` by height+gap.
#[test]
fn flex_column_stacks_items_along_y() {
    let rt = drive(
        ".row{display:flex;flex-direction:column;gap:10px} \
         .a{width:20px;height:8px} .b{width:30px;height:8px}",
        &two_child_view(),
    );
    let (a, b) = children(&rt);
    let (ax, ay, _, ah) = geom(&rt, a);
    let (bx, by, _, _) = geom(&rt, b);
    approx(ay, 0.0, "a.y");
    approx(ah, 8.0, "a.height");
    approx(by, 18.0, "b.y (8 height + 10 gap)");
    approx(ax, bx, "a and b share an x (stacked, not side by side)");
}

// 3. padding: container `padding:5px` shifts the children origin by 5 in x and y.
#[test]
fn container_padding_shifts_children_origin() {
    let rt = drive(
        ".row{display:flex;flex-direction:row;padding:5px} \
         .a{width:20px;height:8px} .b{width:30px;height:8px}",
        &two_child_view(),
    );
    let (a, b) = children(&rt);
    let (ax, ay, _, _) = geom(&rt, a);
    let (bx, by, _, _) = geom(&rt, b);
    approx(ax, 5.0, "a.x shifted by padding");
    approx(ay, 5.0, "a.y shifted by padding");
    approx(bx, 25.0, "b.x = 5 padding + 20 a.width");
    approx(by, 5.0, "b.y shifted by padding");
}

// 4. margin: a child `margin-left:7px` shifts it right by 7 (and pushes `.b`).
#[test]
fn child_margin_left_shifts_it_right() {
    let rt = drive(
        ".row{display:flex;flex-direction:row} \
         .a{width:20px;height:8px;margin-left:7px} .b{width:30px;height:8px}",
        &two_child_view(),
    );
    let (a, b) = children(&rt);
    let (ax, _, aw, _) = geom(&rt, a);
    let (bx, _, _, _) = geom(&rt, b);
    approx(ax, 7.0, "a.x shifted right by margin-left");
    approx(aw, 20.0, "a keeps its width");
    approx(bx, 27.0, "b.x = 7 margin + 20 a.width");
}

// 5. percentage width: `.a{width:50%}` is half the container content width.
#[test]
fn percentage_width_resolves_against_container() {
    let rt = drive(
        ".row{display:flex;flex-direction:row} \
         .a{width:50%;height:8px} .b{width:30px;height:8px}",
        &two_child_view(),
    );
    let (a, _) = children(&rt);
    let (_, _, aw, _) = geom(&rt, a);
    // The flex container content extent is the 300px viewport; 50% => 150.
    approx(aw, VIEWPORT * 0.5, "a.width = 50% of the 300px container");
}

// 6. justify-content: `flex-end` and `center` shift the children block.
#[test]
fn justify_content_shifts_children_block() {
    // flex-end: the two items (20 + 30 = 50 wide) are pushed to the far right.
    let end = drive(
        ".row{display:flex;flex-direction:row;justify-content:flex-end} \
         .a{width:20px;height:8px} .b{width:30px;height:8px}",
        &two_child_view(),
    );
    let (a, b) = children(&end);
    approx(geom(&end, a).0, 250.0, "flex-end a.x = 300 - 50");
    approx(geom(&end, b).0, 270.0, "flex-end b.x = 250 + 20");

    // center: leading = (300 - 50) / 2 = 125.
    let center = drive(
        ".row{display:flex;flex-direction:row;justify-content:center} \
         .a{width:20px;height:8px} .b{width:30px;height:8px}",
        &two_child_view(),
    );
    let (a, b) = children(&center);
    approx(geom(&center, a).0, 125.0, "center a.x = (300 - 50)/2");
    approx(geom(&center, b).0, 145.0, "center b.x = 125 + 20");
}

// 7. min-width / max-width clamp a child's size.
#[test]
fn min_and_max_width_clamp_child_size() {
    // min-width raises a width below it.
    let min = drive(
        ".row{display:flex;flex-direction:row} \
         .a{width:20px;min-width:40px;height:8px} .b{width:30px;height:8px}",
        &two_child_view(),
    );
    let (a, b) = children(&min);
    approx(geom(&min, a).2, 40.0, "min-width raises a.width 20 -> 40");
    approx(geom(&min, b).0, 40.0, "b.x follows the clamped a.width");

    // max-width lowers a width above it.
    let max = drive(
        ".row{display:flex;flex-direction:row} \
         .a{width:50px;max-width:25px;height:8px} .b{width:30px;height:8px}",
        &two_child_view(),
    );
    let (a, b) = children(&max);
    approx(geom(&max, a).2, 25.0, "max-width lowers a.width 50 -> 25");
    approx(geom(&max, b).0, 25.0, "b.x follows the clamped a.width");
}
