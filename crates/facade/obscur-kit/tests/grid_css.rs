//! End-to-end verification that **authored CSS grid** reaches the layout engine
//! and produces the expected [`LayoutBox`](obscur_kit::layout::model::LayoutBox)
//! geometry through the *real* live pipeline (spec §4.1/§4.2).
//!
//! Every test drives the genuine path — author CSS text (`display: grid` with
//! `grid-template-columns`/`-rows`, `gap`, `repeat(…)`, `minmax(…)`), compile it
//! through the real `obscur-css` frontend (`source -> parse -> compile ->
//! matchable Stylesheet`), and render one frame on a [`UiRuntime`] via
//! [`render_now`](UiRuntime::render_now), which runs `reconcile -> match ->
//! cascade -> resolve (ComputedStyles) -> grid-templates -> layout
//! (LayoutBoxes)`. The assertions read the *actual* absolute [`LayoutBox`]
//! coordinates the pipeline produced (`runtime.layout_boxes()`), never a
//! hand-built `GridTemplate`/`LayoutStyle`, so a regression anywhere in the chain
//! (CSS lowering → resolver → `grid_template_for` conversion →
//! `GridLayout::with_template` dispatch → the track-sizing algorithm) fails the
//! run.
//!
//! # Container sizing
//!
//! A grid container honours its own explicit `width`/`height`: [`GridLayout`]
//! sizes its tracks (and distributes `fr` space) against the authored extent, and
//! the container box *is* that size. A container with no explicit size fills the
//! available extent its parent flow hands down (the viewport) — the
//! fill-available default. Tests that author an explicit container size render
//! under a larger viewport and assert the container is its authored size;
//! `auto_placement_wraps_to_second_row` keeps a viewport-driven (auto) case to
//! prove the fill-available default still holds.
//!
//! # Proven vs still limited
//!
//! Proven end-to-end here: fixed / percentage / `fr` / `auto` / `minmax` column
//! tracks, `fr` free-space distribution, `gap` on both axes, `repeat(N, …)`
//! expansion, `repeat(auto-fill, …)` sized from the container extent, row-major
//! auto-placement into implicit rows, and explicit fixed row tracks. Percentage
//! tracks resolve against the container's own content extent inside the algorithm
//! (`30%` of a `400px` container → `120px`). Still out of scope (documented in
//! steps 1-2, not exercised here): line-based placement / spanning, dense packing,
//! named lines, and alignment of an item within its track.
#![forbid(unsafe_code)]

use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::geometry::Size;
use obscur_kit::identity::{ComponentId, NodeId, RuntimeId};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::view::view::{CLASS, View, ViewFragment, ViewNode, ViewProperty, ViewValue};

/// Compiles authored CSS text into a runtime-matchable [`Stylesheet`] through the
/// real CSS frontend (`source -> parse -> compile`).
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

/// The class each grid child carries, in order.
const CHILD_CLASSES: [&str; 5] = ["a", "b", "c", "d", "e"];

/// A `.grid` container holding `n` children classed `.a`, `.b`, `.c`, … in order.
fn grid_view(n: usize) -> View {
    let kids: Vec<ViewNode> = (0..n)
        .map(|i| tag(el(100 + i as u64), CHILD_CLASSES[i]))
        .collect();
    let container = tag(el(20), "grid").with_children(ViewFragment::from_nodes(kids));
    View::from_fragment(ViewFragment::from_nodes(vec![container]))
}

/// Renders one real frame for `css` + a `.grid` container of `n` children under a
/// `width` x `height` viewport (the available extent an auto-sized container
/// fills), and returns each child's [`NodeId`] in authored order plus the driven
/// runtime.
fn drive(css: &str, n: usize, width: f32, height: f32) -> (UiRuntime, Vec<NodeId>) {
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(width, height));
    let sheet = compile(css);
    let view = grid_view(n);
    let result = rt.render_now(&view, std::slice::from_ref(&sheet));
    assert!(result.restyled(), "authored grid CSS resolved computed styles");
    assert!(result.relaid_out(), "the styled tree arranged layout boxes");
    let root = rt.tree().root().expect("root after a frame");
    let container = rt.tree().get(root).unwrap().children().to_vec()[0];
    let kids = rt.tree().get(container).unwrap().children().to_vec();
    (rt, kids)
}

/// The single `.grid` container [`NodeId`].
fn container(rt: &UiRuntime) -> NodeId {
    let root = rt.tree().root().expect("root after a frame");
    rt.tree().get(root).unwrap().children().to_vec()[0]
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

// 1. fixed + fr columns resolve against the container's OWN explicit width:
//    `.grid{width:400px}` + `100px 1fr 2fr` under a *500px* viewport.
//    fr unit = (400 - 100) / (1 + 2) = 100 -> 1fr=100, 2fr=200. The container is
//    its authored 400px, not stretched to the 500px viewport.
#[test]
fn fixed_and_fr_columns_against_explicit_width() {
    let (rt, k) = drive(
        ".grid{display:grid;width:400px;grid-template-columns:100px 1fr 2fr} \
         .a{height:10px} .b{height:10px} .c{height:10px}",
        3,
        500.0,
        200.0,
    );
    approx(geom(&rt, container(&rt)).2, 400.0, "container is its authored 400px");
    let (ax, _, aw, _) = geom(&rt, k[0]);
    let (bx, _, bw, _) = geom(&rt, k[1]);
    let (cx, _, cw, _) = geom(&rt, k[2]);
    approx(ax, 0.0, "a.x");
    approx(aw, 100.0, "a.width (100px fixed)");
    approx(bx, 100.0, "b.x");
    approx(bw, 100.0, "b.width (1fr against 400, not 500)");
    approx(cx, 200.0, "c.x");
    approx(cw, 200.0, "c.width (2fr against 400, not 500)");
}

// 2. repeat + gap: `repeat(2, 1fr); gap: 10px` on a 300px container, 2 children.
//    column = (300 - 10 gap) / 2 = 145; b starts at 145 + 10 gap = 155.
#[test]
fn repeat_columns_with_gap() {
    let (rt, k) = drive(
        ".grid{display:grid;grid-template-columns:repeat(2, 1fr);gap:10px} \
         .a{height:10px} .b{height:10px}",
        2,
        300.0,
        200.0,
    );
    let (ax, _, aw, _) = geom(&rt, k[0]);
    let (bx, _, bw, _) = geom(&rt, k[1]);
    approx(ax, 0.0, "a.x");
    approx(aw, 145.0, "a.width ((300-10)/2)");
    approx(bx, 155.0, "b.x (145 + 10 gap)");
    approx(bw, 145.0, "b.width ((300-10)/2)");
}

// 3. auto-placement into rows: `1fr 1fr` with 3 children -> c wraps to row 1.
//    a(col0,row0) b(col1,row0) c(col0,row1); row 0 is 10px tall so c.y = 10.
#[test]
fn auto_placement_wraps_to_second_row() {
    let (rt, k) = drive(
        ".grid{display:grid;grid-template-columns:1fr 1fr} \
         .a{height:10px} .b{height:10px} .c{height:10px}",
        3,
        300.0,
        200.0,
    );
    let (ax, ay, _, _) = geom(&rt, k[0]);
    let (bx, by, _, _) = geom(&rt, k[1]);
    let (cx, cy, _, _) = geom(&rt, k[2]);
    approx(ax, 0.0, "a.x (col0)");
    approx(ay, 0.0, "a.y (row0)");
    approx(bx, 150.0, "b.x (col1)");
    approx(by, 0.0, "b.y (row0)");
    approx(cx, 0.0, "c.x (col0, wrapped)");
    assert!(cy > 0.0, "c wraps to a second row: c.y = {cy} > 0");
    approx(cy, 10.0, "c.y (row1, below the 10px row0)");
}

// 4. explicit rows under an explicit container height: `.grid{height:200px}`,
//    `grid-template-rows: 50px 80px`, `grid-template-columns: 1fr`, 2 children ->
//    single column, a in the 50px row, b in the 80px row below it, and the
//    container is its authored 200px tall (not the 130px its tracks occupy nor the
//    400px viewport).
#[test]
fn explicit_fixed_rows_and_container_height() {
    let (rt, k) = drive(
        ".grid{display:grid;height:200px;grid-template-columns:1fr;grid-template-rows:50px 80px} \
         .a{height:10px} .b{height:10px}",
        2,
        300.0,
        400.0,
    );
    approx(geom(&rt, container(&rt)).3, 200.0, "container is its authored 200px tall");
    let (_, ay, aw, ah) = geom(&rt, k[0]);
    let (_, by, _, bh) = geom(&rt, k[1]);
    approx(ay, 0.0, "a.y (row0 at top)");
    approx(ah, 50.0, "a.height (50px row track)");
    approx(aw, 300.0, "a.width (single 1fr column)");
    approx(by, 50.0, "b.y (row1, below the 50px row0)");
    approx(bh, 80.0, "b.height (80px row track)");
}

// 5. auto column: `auto 1fr` where a has an intrinsic (fixed) width -> the auto
//    column sizes to a's content (80), the fr column takes the rest (220).
#[test]
fn auto_column_sizes_to_content() {
    let (rt, k) = drive(
        ".grid{display:grid;grid-template-columns:auto 1fr} \
         .a{width:80px;height:10px} .b{height:10px}",
        2,
        300.0,
        200.0,
    );
    let (ax, _, aw, _) = geom(&rt, k[0]);
    let (bx, _, bw, _) = geom(&rt, k[1]);
    approx(ax, 0.0, "a.x");
    approx(aw, 80.0, "a.width (auto column sized to 80px content)");
    approx(bx, 80.0, "b.x (after the auto column)");
    approx(bw, 220.0, "b.width (1fr = 300 - 80)");
}

// 6. minmax floor: `minmax(120px, 1fr) 1fr` on a 100px container. A plain `1fr`
//    would give each column 50px (< 120), but the minmax min floors column 0 at
//    120 even when the container cannot afford it; column 1 collapses to 0 (no
//    free space left over to distribute).
#[test]
fn minmax_floors_first_column() {
    let (rt, k) = drive(
        ".grid{display:grid;grid-template-columns:minmax(120px, 1fr) 1fr} \
         .a{height:10px} .b{height:10px}",
        2,
        100.0,
        200.0,
    );
    let (ax, _, aw, _) = geom(&rt, k[0]);
    let (bx, _, bw, _) = geom(&rt, k[1]);
    approx(ax, 0.0, "a.x");
    approx(aw, 120.0, "a.width floored at the 120px minmax minimum");
    approx(bx, 120.0, "b.x (after the floored first column)");
    approx(bw, 0.0, "b.width collapses (no free space after the floor)");
}

// 7. `repeat(auto-fill, 100px)` on a 350px container: the algorithm fits 3 whole
//    100px columns (3*100 = 300 <= 350; a 4th overflows), so the 3 children land at
//    x = 0 / 100 / 200 — the repetition count computed from the container size, not
//    a single repetition.
#[test]
fn auto_fill_columns_from_container_width() {
    let (rt, k) = drive(
        ".grid{display:grid;width:350px;grid-template-columns:repeat(auto-fill,100px)} \
         .a{height:10px} .b{height:10px} .c{height:10px}",
        3,
        500.0,
        200.0,
    );
    let (ax, _, aw, _) = geom(&rt, k[0]);
    let (bx, _, bw, _) = geom(&rt, k[1]);
    let (cx, _, _, _) = geom(&rt, k[2]);
    approx(ax, 0.0, "a.x (col0)");
    approx(aw, 100.0, "a.width (100px auto-fill track)");
    approx(bx, 100.0, "b.x (col1)");
    approx(bw, 100.0, "b.width (100px auto-fill track)");
    approx(cx, 200.0, "c.x (col2 — 3 columns fit the 350px container)");
}

// 8. percentage + fr columns resolve against the container's OWN content width:
//    `.grid{width:400px}` + `30% 1fr`. 30% of 400 = 120; the 1fr track takes the
//    280px remainder. This is the case that previously degraded to `auto` because
//    the percent basis was unknown at the runtime->layout conversion step; it now
//    resolves inside the algorithm, which knows the container content size.
#[test]
fn percent_and_fr_columns_against_explicit_width() {
    let (rt, k) = drive(
        ".grid{display:grid;width:400px;grid-template-columns:30% 1fr} \
         .a{height:10px} .b{height:10px}",
        2,
        500.0,
        200.0,
    );
    let (ax, _, aw, _) = geom(&rt, k[0]);
    let (bx, _, bw, _) = geom(&rt, k[1]);
    approx(ax, 0.0, "a.x");
    approx(aw, 120.0, "a.width (30% of the 400px container)");
    approx(bx, 120.0, "b.x (after the 30% track)");
    approx(bw, 280.0, "b.width (1fr takes the 280px remainder)");
}
