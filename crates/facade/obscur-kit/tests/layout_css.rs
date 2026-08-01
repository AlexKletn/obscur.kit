#![doc = include_str!("layout_css.doc/_module.md")]
#![forbid(unsafe_code)]

use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::geometry::Size;
use obscur_kit::identity::{ComponentId, NodeId, RuntimeId};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::view::view::{CLASS, View, ViewFragment, ViewNode, ViewProperty, ViewValue};

#[doc = include_str!("layout_css.doc/VIEWPORT.md")]
const VIEWPORT: f32 = 300.0;

#[doc = include_str!("layout_css.doc/compile.md")]
fn compile(css: &str) -> Stylesheet {
    let src = CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&src).parse();
    StylesheetCompiler::new().compile(&parsed).matchable
}

#[doc = include_str!("layout_css.doc/tag.md")]
fn tag(node: ViewNode, cls: &str) -> ViewNode {
    node.with_prop(ViewProperty::new(CLASS, ViewValue::Text(cls.into())))
}

fn el(id: u64) -> ViewNode {
    ViewNode::element(ComponentId::from_raw(id))
}

#[doc = include_str!("layout_css.doc/two_child_view.md")]
fn two_child_view() -> View {
    let a = tag(el(21), "a");
    let b = tag(el(22), "b");
    let container = tag(el(20), "row").with_children(ViewFragment::from_nodes(vec![a, b]));
    View::from_fragment(ViewFragment::from_nodes(vec![container]))
}

#[doc = include_str!("layout_css.doc/drive.md")]
fn drive(css: &str, view: &View) -> UiRuntime {
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(VIEWPORT, VIEWPORT));
    let sheet = compile(css);
    let result = rt.render_now(view, std::slice::from_ref(&sheet));
    assert!(result.restyled(), "authored CSS resolved computed styles");
    assert!(result.relaid_out(), "the styled tree arranged layout boxes");
    rt
}

#[doc = include_str!("layout_css.doc/container_node.md")]
fn container_node(rt: &UiRuntime) -> NodeId {
    let root = rt.tree().root().expect("root after a frame");
    rt.tree().get(root).unwrap().child_nodes().to_vec()[0]
}

#[doc = include_str!("layout_css.doc/children.md")]
fn children(rt: &UiRuntime) -> (NodeId, NodeId) {
    let container = container_node(rt);
    let kids = rt.tree().get(container).unwrap().child_nodes().to_vec();
    (kids[0], kids[1])
}

#[doc = include_str!("layout_css.doc/geom.md")]
fn geom(rt: &UiRuntime, node: NodeId) -> (f32, f32, f32, f32) {
    let b = rt
        .layout_boxes()
        .get(&node)
        .copied()
        .expect("node has a box");
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

// 5. percentage width resolves against the container's OWN explicit width:
//    `.row{width:200px}` + `.a{width:50%}` => 100 (half of 200, not of the 300 viewport).
#[test]
fn percentage_width_resolves_against_explicit_container_width() {
    let rt = drive(
        ".row{display:flex;flex-direction:row;width:200px} \
         .a{width:50%;height:8px} .b{width:30px;height:8px}",
        &two_child_view(),
    );
    // The container is its authored 200px wide, not stretched to the 300 viewport.
    let (_, _, cw, _) = geom(&rt, container_node(&rt));
    approx(cw, 200.0, "container honours its explicit width:200px");
    let (a, _) = children(&rt);
    let (_, _, aw, _) = geom(&rt, a);
    // 50% of the explicit 200px container => 100 (would be 150 against the viewport).
    approx(aw, 100.0, "a.width = 50% of the 200px container");
}

// 6. justify-content resolves against the container's OWN explicit width:
//    `.row{width:200px}` with two items (20 + 30 = 50 wide), remaining = 150.
#[test]
fn justify_content_shifts_children_block_within_explicit_width() {
    // flex-end: the items pack to the right within the authored 200px (not 300).
    let end = drive(
        ".row{display:flex;flex-direction:row;width:200px;justify-content:flex-end} \
         .a{width:20px;height:8px} .b{width:30px;height:8px}",
        &two_child_view(),
    );
    approx(
        geom(&end, container_node(&end)).2,
        200.0,
        "container is 200px wide",
    );
    let (a, b) = children(&end);
    approx(geom(&end, a).0, 150.0, "flex-end a.x = 200 - 50");
    approx(geom(&end, b).0, 170.0, "flex-end b.x = 150 + 20");

    // center: leading = (200 - 50) / 2 = 75.
    let center = drive(
        ".row{display:flex;flex-direction:row;width:200px;justify-content:center} \
         .a{width:20px;height:8px} .b{width:30px;height:8px}",
        &two_child_view(),
    );
    let (a, b) = children(&center);
    approx(geom(&center, a).0, 75.0, "center a.x = (200 - 50)/2");
    approx(geom(&center, b).0, 95.0, "center b.x = 75 + 20");
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
