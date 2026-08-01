#![doc = include_str!("position_css.doc/_module.md")]
#![forbid(unsafe_code)]

use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::geometry::{Rect, Size};
use obscur_kit::identity::{ComponentId, NodeId, RuntimeId};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::scene::command::PaintCommand;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::view::view::{CLASS, View, ViewFragment, ViewNode, ViewProperty, ViewValue};

#[doc = include_str!("position_css.doc/VIEWPORT.md")]
const VIEWPORT: f32 = 300.0;

#[doc = include_str!("position_css.doc/compile.md")]
fn compile(css: &str) -> Stylesheet {
    let src = CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&src).parse();
    StylesheetCompiler::new().compile(&parsed).matchable
}

#[doc = include_str!("position_css.doc/tag.md")]
fn tag(node: ViewNode, cls: &str) -> ViewNode {
    node.with_prop(ViewProperty::new(CLASS, ViewValue::Text(cls.into())))
}

fn el(id: u64) -> ViewNode {
    ViewNode::element(ComponentId::from_raw(id))
}

#[doc = include_str!("position_css.doc/view.md")]
fn view() -> View {
    let leaf = tag(el(23), "leaf");
    let a = tag(el(21), "a").with_children(ViewFragment::from_nodes(vec![leaf]));
    let b = tag(el(22), "b");
    let container = tag(el(20), "row").with_children(ViewFragment::from_nodes(vec![a, b]));
    View::from_fragment(ViewFragment::from_nodes(vec![container]))
}

#[doc = include_str!("position_css.doc/drive.md")]
fn drive(css: &str) -> UiRuntime {
    drive_view(css, &view())
}

#[doc = include_str!("position_css.doc/drive_view.md")]
fn drive_view(css: &str, view: &View) -> UiRuntime {
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(VIEWPORT, VIEWPORT));
    let sheet = compile(css);
    let result = rt.render_now(view, std::slice::from_ref(&sheet));
    assert!(result.restyled(), "authored CSS resolved computed styles");
    assert!(result.relaid_out(), "the styled tree arranged layout boxes");
    rt
}

#[doc = include_str!("position_css.doc/box_size.md")]
fn box_size(rt: &UiRuntime, node: NodeId) -> (f32, f32) {
    let b = rt
        .layout_boxes()
        .get(&node)
        .copied()
        .expect("node has a box");
    (b.size().width, b.size().height)
}

#[doc = include_str!("position_css.doc/container_node.md")]
fn container_node(rt: &UiRuntime) -> NodeId {
    let root = rt.tree().root().expect("root after a frame");
    rt.tree().get(root).unwrap().child_nodes().to_vec()[0]
}

#[doc = include_str!("position_css.doc/children.md")]
fn children(rt: &UiRuntime) -> (NodeId, NodeId) {
    let container = container_node(rt);
    let kids = rt.tree().get(container).unwrap().child_nodes().to_vec();
    (kids[0], kids[1])
}

#[doc = include_str!("position_css.doc/leaf_node.md")]
fn leaf_node(rt: &UiRuntime, a: NodeId) -> NodeId {
    rt.tree().get(a).unwrap().child_nodes().to_vec()[0]
}

#[doc = include_str!("position_css.doc/origin.md")]
fn origin(rt: &UiRuntime, node: NodeId) -> (f32, f32) {
    let b = rt
        .layout_boxes()
        .get(&node)
        .copied()
        .expect("node has a box");
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

// Scenario (a): an `position: absolute` sibling is out of flow — it neither
// moves the in-flow `.a` nor contributes to the container's content height, and
// is placed at its insets within its containing block.
#[test]
fn absolute_sibling_is_out_of_flow_and_positioned_by_insets() {
    let a = tag(el(21), "a");
    let b = tag(el(22), "b");
    let container = tag(el(20), "row").with_children(ViewFragment::from_nodes(vec![a, b]));
    let view = View::from_fragment(ViewFragment::from_nodes(vec![container]));

    let rt = drive_view(
        ".row{width:300px} \
         .a{width:300px;height:50px} \
         .b{position:absolute;top:10px;left:20px;width:40px;height:30px}",
        &view,
    );

    let container = container_node(&rt);
    let (a, b) = children(&rt);

    // `.a` keeps its flow origin and size, unaffected by `.b`.
    approx(origin(&rt, a).0, 0.0, "a.x unaffected by absolute b");
    approx(origin(&rt, a).1, 0.0, "a.y unaffected by absolute b");
    approx(box_size(&rt, a).1, 50.0, "a keeps its own height");

    // The auto-height container reflects only `.a` (50), not `.b`.
    approx(
        box_size(&rt, container).1,
        50.0,
        "container height is only a's 50",
    );

    // `.b` is placed at (left, top) = (20, 10) with its explicit 40x30 size.
    approx(origin(&rt, b).0, 20.0, "b.x = left inset");
    approx(origin(&rt, b).1, 10.0, "b.y = top inset");
    approx(box_size(&rt, b).0, 40.0, "b.width explicit");
    approx(box_size(&rt, b).1, 30.0, "b.height explicit");
}

// Scenario (b): an absolute element resolves against the padding box of a
// `position: relative` ancestor (not the root); `right: 0; bottom: 0` pins it to
// that ancestor's far corner.
#[test]
fn absolute_resolves_against_positioned_ancestor() {
    let abs = tag(el(42), "abs");
    let rel = tag(el(41), "rel").with_children(ViewFragment::from_nodes(vec![abs]));
    let outer = tag(el(40), "outer").with_children(ViewFragment::from_nodes(vec![rel]));
    let view = View::from_fragment(ViewFragment::from_nodes(vec![outer]));

    let rt = drive_view(
        ".outer{width:300px;height:300px} \
         .rel{position:relative;left:100px;top:100px;width:200px;height:200px} \
         .abs{position:absolute;right:0px;bottom:0px;width:20px;height:20px}",
        &view,
    );

    let outer = container_node(&rt);
    let rel = rt.tree().get(outer).unwrap().child_nodes().to_vec()[0];
    let abs = rt.tree().get(rel).unwrap().child_nodes().to_vec()[0];

    // `.rel` is offset into the flow by its insets (100, 100).
    approx(origin(&rt, rel).0, 100.0, "rel.x from left:100");
    approx(origin(&rt, rel).1, 100.0, "rel.y from top:100");

    // `.abs` sits flush against `.rel`'s far corner: (100+200-20, 100+200-20).
    approx(origin(&rt, abs).0, 280.0, "abs.x = rel.right - width");
    approx(origin(&rt, abs).1, 280.0, "abs.y = rel.bottom - height");
    approx(box_size(&rt, abs).0, 20.0, "abs.width explicit");
    approx(box_size(&rt, abs).1, 20.0, "abs.height explicit");
}

// End-to-end: a `position: sticky; top: 10px` child in a block-flow container is
// scroll-clamped by the test-driven viewport scroll offset. With scroll (0, 0) it
// sits at its static flow position; after `set_viewport_scroll(0, 100)` its used
// top sticks to the scrollport edge (100 + 10 = 110), its subtree rides along, and
// the following sibling — which stays IN flow — is unmoved. This drives the full
// pipeline twice on one runtime, engaging the sticky clamp only via the scroll
// input the second frame.
#[test]
fn sticky_child_clamps_on_scroll_leaving_sibling_in_flow() {
    // A block-flow container: a 50px `.spacer`, then a `position:sticky; top:10px`
    // `.a` (holding a `.leaf`), then a `.b` sibling. `.a`'s static flow top is 50,
    // which is BELOW its 10px inset, so at scroll 0 it is at its static position.
    let leaf = tag(el(23), "leaf");
    let spacer = tag(el(24), "spacer");
    let a = tag(el(21), "a").with_children(ViewFragment::from_nodes(vec![leaf]));
    let b = tag(el(22), "b");
    let container = tag(el(20), "row").with_children(ViewFragment::from_nodes(vec![spacer, a, b]));
    let view = View::from_fragment(ViewFragment::from_nodes(vec![container]));

    let css = ".row{width:300px} \
               .spacer{width:300px;height:50px} \
               .a{position:sticky;top:10px;width:300px;height:40px} \
               .leaf{width:20px;height:20px} \
               .b{width:300px;height:40px}";
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(VIEWPORT, VIEWPORT));
    let sheet = compile(css);

    let kids = |rt: &UiRuntime| {
        let c = container_node(rt);
        rt.tree().get(c).unwrap().child_nodes().to_vec()
    };

    // Frame 1: no scroll — the sticky child lays out at its static flow position.
    let r1 = rt.render_now(&view, std::slice::from_ref(&sheet));
    assert!(r1.relaid_out(), "first frame arranged boxes");
    let k = kids(&rt);
    let (a_node, b_node) = (k[1], k[2]);
    let leaf_node = rt.tree().get(a_node).unwrap().child_nodes().to_vec()[0];
    approx(
        origin(&rt, a_node).1,
        50.0,
        "scroll 0: sticky a at static y=50",
    );
    approx(
        origin(&rt, leaf_node).1,
        50.0,
        "scroll 0: leaf at a's content origin",
    );
    approx(
        origin(&rt, b_node).1,
        90.0,
        "scroll 0: sibling b below a (flow)",
    );

    // Frame 2: scroll the viewport 100px down — the sticky child sticks to the
    // scrollport top plus its `top:10px` inset (used top 110), within its CB.
    rt.set_viewport_scroll(0.0, 100.0);
    let r2 = rt.render_now(&view, std::slice::from_ref(&sheet));
    assert!(r2.relaid_out(), "second frame re-arranged boxes");
    approx(
        origin(&rt, a_node).1,
        110.0,
        "scroll 100: a sticks at scrollport+inset (110)",
    );
    approx(
        origin(&rt, leaf_node).1,
        110.0,
        "scroll 100: the sticky subtree rides along",
    );
    approx(
        origin(&rt, b_node).1,
        90.0,
        "scroll 100: sibling b unmoved (stays in flow)",
    );
    approx(
        origin(&rt, a_node).0,
        0.0,
        "no horizontal inset → a.x unchanged",
    );
}

// End-to-end: two overlapping `position: absolute` boxes with `background-color`
// and different `z-index` paint in STACKING order, not tree order. `.boxa` is
// FIRST in the tree but carries the HIGHER `z-index` (5) and `.boxb` is second
// with the lower `z-index` (1); the composed display list must emit `.boxb`'s
// fill first and `.boxa`'s fill LAST (higher z on top), overriding document
// order. This drives the genuine pipeline through to the scene's paint commands.
#[test]
fn z_index_orders_overlapping_absolute_fills_by_stacking() {
    let boxa = tag(el(31), "boxa");
    let boxb = tag(el(32), "boxb");
    let wrap = tag(el(30), "wrap").with_children(ViewFragment::from_nodes(vec![boxa, boxb]));
    let view = View::from_fragment(ViewFragment::from_nodes(vec![wrap]));

    let rt = drive_view(
        ".wrap{position:relative;width:100px;height:100px} \
         .boxa{position:absolute;top:0px;left:0px;width:40px;height:40px;\
               background-color:#ff0000;z-index:5} \
         .boxb{position:absolute;top:20px;left:20px;width:40px;height:40px;\
               background-color:#0000ff;z-index:1}",
        &view,
    );

    let wrap = container_node(&rt);
    let kids = rt.tree().get(wrap).unwrap().child_nodes().to_vec();
    let (node_a, node_b) = (kids[0], kids[1]);
    let bounds_a = rt
        .layout_boxes()
        .get(&node_a)
        .expect("boxa laid out")
        .bounds;
    let bounds_b = rt
        .layout_boxes()
        .get(&node_b)
        .expect("boxb laid out")
        .bounds;

    // Locate each node's background fill in the composed command stream by its
    // arranged rectangle.
    let snapshot = rt.snapshot();
    let commands = snapshot.display_list().commands();
    let fill_index = |target: Rect| {
        commands.iter().position(|c| match c {
            PaintCommand::DrawPath { path, .. } => path.bounds() == Some(target),
            _ => false,
        })
    };
    let index_a = fill_index(bounds_a).expect("boxa background fill emitted");
    let index_b = fill_index(bounds_b).expect("boxb background fill emitted");

    // `.boxa` has the higher z-index, so its fill is emitted AFTER `.boxb`'s
    // despite being earlier in the tree — the paint order reflects z, not tree.
    assert!(
        index_a > index_b,
        "higher z-index (.boxa, z=5) must paint after lower (.boxb, z=1): \
         boxa at {index_a}, boxb at {index_b}",
    );
}
