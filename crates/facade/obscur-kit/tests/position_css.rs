//! End-to-end verification that authored `position` / inset CSS reaches the
//! layout engine and offsets a `position: relative` element (and its subtree)
//! without moving its siblings (spec §4.1).
//!
//! Like the sibling `layout_css` suite, every test drives the genuine pipeline —
//! author CSS text, compile it through the real `obscur-css` frontend, and render
//! one frame on a [`UiRuntime`] via [`render_now`](UiRuntime::render_now)
//! (`reconcile -> match -> cascade -> resolve -> layout`). The assertions read the
//! actual absolute [`LayoutBox`](obscur_kit::layout::model::LayoutBox) coordinates
//! the pipeline produced, so a regression anywhere from the CSS lowering through
//! the computed→layout bridge to `arrange` fails the run.
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

/// Compiles authored CSS text into a runtime-matchable [`Stylesheet`].
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

/// A `.row` container holding a `.a` child (which itself holds a `.leaf`
/// grandchild) followed by a `.b` sibling.
fn view() -> View {
    let leaf = tag(el(23), "leaf");
    let a = tag(el(21), "a").with_children(ViewFragment::from_nodes(vec![leaf]));
    let b = tag(el(22), "b");
    let container = tag(el(20), "row").with_children(ViewFragment::from_nodes(vec![a, b]));
    View::from_fragment(ViewFragment::from_nodes(vec![container]))
}

/// Renders one real frame for `css` + the view and returns the driven runtime.
fn drive(css: &str) -> UiRuntime {
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(VIEWPORT, VIEWPORT));
    let sheet = compile(css);
    let result = rt.render_now(&view(), std::slice::from_ref(&sheet));
    assert!(result.restyled(), "authored CSS resolved computed styles");
    assert!(result.relaid_out(), "the styled tree arranged layout boxes");
    rt
}

/// The `.row` container [`NodeId`].
fn container_node(rt: &UiRuntime) -> NodeId {
    let root = rt.tree().root().expect("root after a frame");
    rt.tree().get(root).unwrap().children().to_vec()[0]
}

/// The `.a` and `.b` child [`NodeId`]s of the container, in order.
fn children(rt: &UiRuntime) -> (NodeId, NodeId) {
    let container = container_node(rt);
    let kids = rt.tree().get(container).unwrap().children().to_vec();
    (kids[0], kids[1])
}

/// The `.leaf` grandchild [`NodeId`] under `.a`.
fn leaf_node(rt: &UiRuntime, a: NodeId) -> NodeId {
    rt.tree().get(a).unwrap().children().to_vec()[0]
}

/// The absolute origin `(x, y)` of a node's layout box.
fn origin(rt: &UiRuntime, node: NodeId) -> (f32, f32) {
    let b = rt.layout_boxes().get(&node).copied().expect("node has a box");
    (b.origin().x, b.origin().y)
}

fn approx(actual: f32, expected: f32, what: &str) {
    assert!(
        (actual - expected).abs() < 1e-3,
        "{what}: expected {expected}, got {actual}"
    );
}

// A `position: relative; left: 20px; top: 10px` child shifts by (20, 10); its
// subtree shifts with it; the following sibling is unmoved.
#[test]
fn relative_child_offsets_itself_and_subtree_leaving_sibling_put() {
    let rt = drive(
        ".row{display:flex;flex-direction:row} \
         .a{position:relative;left:20px;top:10px;width:20px;height:8px} \
         .leaf{width:6px;height:6px} \
         .b{width:30px;height:8px}",
    );
    let (a, b) = children(&rt);
    let leaf = leaf_node(&rt, a);

    // `.a` flow origin is (0, 0); the insets shift it to (20, 10).
    let (ax, ay) = origin(&rt, a);
    approx(ax, 20.0, "a.x shifted by left:20px");
    approx(ay, 10.0, "a.y shifted by top:10px");

    // The `.leaf` grandchild (flow-placed at a's content origin) rides along.
    let (lx, ly) = origin(&rt, leaf);
    approx(lx, 20.0, "leaf.x shifts with the relative subtree");
    approx(ly, 10.0, "leaf.y shifts with the relative subtree");

    // `.b` follows a's UNCHANGED flow box (x = a.width = 20), so it is unmoved.
    let (bx, by) = origin(&rt, b);
    approx(bx, 20.0, "b.x = a.width; unaffected by a's relative offset");
    approx(by, 0.0, "b.y unaffected by a's relative offset");
}

// The `inset` shorthand drives the same offset: `inset: 10px 0 0 20px` =>
// top=10, right=0, bottom=0, left=20 (left wins) -> shift (20, 10).
#[test]
fn inset_shorthand_drives_the_relative_offset() {
    let rt = drive(
        ".row{display:flex;flex-direction:row} \
         .a{position:relative;inset:10px 0 0 20px;width:20px;height:8px} \
         .leaf{width:6px;height:6px} \
         .b{width:30px;height:8px}",
    );
    let (a, b) = children(&rt);
    approx(origin(&rt, a).0, 20.0, "a.x from inset left=20");
    approx(origin(&rt, a).1, 10.0, "a.y from inset top=10");
    approx(origin(&rt, b).0, 20.0, "b.x unmoved");
}

// A `position: static` (default) element ignores its insets entirely.
#[test]
fn static_child_ignores_insets() {
    let rt = drive(
        ".row{display:flex;flex-direction:row} \
         .a{left:20px;top:10px;width:20px;height:8px} \
         .leaf{width:6px;height:6px} \
         .b{width:30px;height:8px}",
    );
    let (a, _b) = children(&rt);
    approx(origin(&rt, a).0, 0.0, "static a.x ignores left");
    approx(origin(&rt, a).1, 0.0, "static a.y ignores top");
}
