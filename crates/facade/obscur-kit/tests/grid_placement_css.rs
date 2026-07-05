//! End-to-end verification that **authored CSS grid line placement** (`grid-column`
//! / `grid-row`, integer lines and `span`) reaches the layout engine and produces
//! the expected [`LayoutBox`](obscur_kit::layout::model::LayoutBox) geometry through
//! the *real* live pipeline (spec §4.13, the positive-line + span subset).
//!
//! Each test drives the genuine path — author CSS text (`display: grid` plus
//! `grid-column` / `grid-row` shorthands on the children), compile it through the
//! real `obscur-css` frontend (`source -> parse -> compile -> matchable
//! Stylesheet`), and render one frame on a [`UiRuntime`] via
//! [`render_now`](UiRuntime::render_now), which runs `reconcile -> match -> cascade
//! -> resolve (ComputedStyles, incl. the four `grid-*-start`/`-end` longhands) ->
//! grid-templates -> layout (LayoutBoxes)`. The assertions read the *actual*
//! absolute [`LayoutBox`] coordinates the pipeline produced
//! (`runtime.layout_boxes()`), never a hand-built `GridPlacement`/`LayoutStyle`, so
//! a regression anywhere in the chain (CSS shorthand→longhand lowering → resolver
//! `GridLineValue` → coordinator `grid_placement_of` → the grid algorithm's
//! two-phase placement) fails the run.
//!
//! Every container authors an explicit `width` so the columns (and thus the placed
//! item x-coordinates) are deterministic regardless of the viewport.
//!
//! # Proven vs still limited
//!
//! Proven end-to-end here: line-based column placement (`grid-column: 1 / 3`),
//! `span` on an auto-placed item (`grid-column: span 2`), a single explicit line
//! (`grid-column: 2`) with auto items flowing around it, explicit row placement
//! (`grid-row: 2`) against fixed row tracks, a span crossing an interior `gap`, a
//! mixed explicit + auto layout where the auto items first-fit around the occupied
//! cell, negative (from-end) lines (`grid-column: 1 / -1` spanning the whole row),
//! the `grid-area` four-line shorthand, and **named-area placement**
//! (`grid-template-areas` on the container + `grid-area: <name>` on the items,
//! resolved in the runtime against the parent's parsed template). Still out of
//! scope (not exercised here): named grid *lines* (`[name]` in a track list).
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

/// The class each grid child carries, in authored order.
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
/// `width` x `height` viewport, returning the driven runtime and each child's
/// [`NodeId`] in authored order.
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

/// The shared three-fixed-column container prefix (`width:300px`, `100px 100px
/// 100px`). Callers append the per-child rules.
const THREE_COL: &str =
    ".grid{display:grid;width:300px;grid-template-columns:100px 100px 100px}";

// 1. column span via lines: `.a{grid-column: 1 / 3}` spans columns 0-1 (x=0,
//    width 200); a following auto `.b` first-fits into the next free cell (col 2).
#[test]
fn column_line_span_places_and_next_auto_flows() {
    let (rt, k) = drive(
        &format!("{THREE_COL} .a{{grid-column:1 / 3;height:10px}} .b{{height:10px}}"),
        2,
        400.0,
        200.0,
    );
    let (ax, ay, aw, _) = geom(&rt, k[0]);
    let (bx, _, bw, _) = geom(&rt, k[1]);
    approx(ax, 0.0, "a.x (line 1 → col 0)");
    approx(ay, 0.0, "a.y (row 0)");
    approx(aw, 200.0, "a.width (cols 0-1 = 100+100)");
    approx(bx, 200.0, "b.x (auto flows to the free col 2)");
    approx(bw, 100.0, "b.width (single col)");
}

// 2. `span` keyword: `.a{grid-column: span 2}` auto-placed occupies two columns
//    (x=0, width 200); the trailing auto `.b` lands in col 2.
#[test]
fn column_span_keyword_occupies_two_columns() {
    let (rt, k) = drive(
        &format!("{THREE_COL} .a{{grid-column:span 2;height:10px}} .b{{height:10px}}"),
        2,
        400.0,
        200.0,
    );
    let (ax, _, aw, _) = geom(&rt, k[0]);
    let (bx, _, _, _) = geom(&rt, k[1]);
    approx(ax, 0.0, "a.x (auto-placed span from col 0)");
    approx(aw, 200.0, "a.width (span 2 = 100+100)");
    approx(bx, 200.0, "b.x (auto flows past the span into col 2)");
}

// 3. explicit single line: `.a{grid-column: 2}` locks `.a` to col 1 (x=100); the
//    auto `.b` flows around it into col 0 (x=0).
#[test]
fn explicit_column_line_and_auto_flows_around() {
    let (rt, k) = drive(
        &format!("{THREE_COL} .a{{grid-column:2;height:10px}} .b{{height:10px}}"),
        2,
        400.0,
        200.0,
    );
    let (ax, ay, _, _) = geom(&rt, k[0]);
    let (bx, by, _, _) = geom(&rt, k[1]);
    approx(ax, 100.0, "a.x (line 2 → col 1)");
    approx(ay, 0.0, "a.y (row 0)");
    approx(bx, 0.0, "b.x (auto flows into the free col 0)");
    approx(by, 0.0, "b.y (row 0)");
}

// 4. explicit row: `.a{grid-row: 2}` with `grid-template-rows: 50px 60px` puts
//    `.a` on row 1 at y=50; the row-1 track is 60px tall.
#[test]
fn explicit_row_line_places_on_second_row() {
    let (rt, k) = drive(
        ".grid{display:grid;width:300px;grid-template-columns:100px 100px 100px;\
         grid-template-rows:50px 60px} .a{grid-row:2;height:10px}",
        1,
        400.0,
        300.0,
    );
    let (ax, ay, _, ah) = geom(&rt, k[0]);
    approx(ax, 0.0, "a.x (auto column → col 0)");
    approx(ay, 50.0, "a.y (row 1, below the 50px row 0)");
    approx(ah, 60.0, "a.height (60px row-1 track)");
}

// 5. span across a gap: `gap:10px` with `.a{grid-column: 1 / 3}` → the span covers
//    the two 100px columns plus the one interior 10px gap = 210.
#[test]
fn column_span_includes_interior_gap() {
    let (rt, k) = drive(
        ".grid{display:grid;width:340px;grid-template-columns:100px 100px 100px;gap:10px}\
         .a{grid-column:1 / 3;height:10px}",
        1,
        400.0,
        200.0,
    );
    let (ax, _, aw, _) = geom(&rt, k[0]);
    approx(ax, 0.0, "a.x (col 0)");
    approx(aw, 210.0, "a.width (100 + 10 gap + 100)");
}

// 6. mixed explicit + auto: `.b{grid-column: 3}` locks col 2; the auto `.a`, `.c`,
//    `.d` first-fit around it row-major (a→col0, c→col1, d→row1 col0).
#[test]
fn mixed_explicit_and_auto_fill_remaining_cells() {
    let (rt, k) = drive(
        &format!(
            "{THREE_COL} .a{{height:10px}} .b{{grid-column:3;height:10px}} \
             .c{{height:10px}} .d{{height:10px}}"
        ),
        4,
        400.0,
        200.0,
    );
    let (ax, ay, _, _) = geom(&rt, k[0]);
    let (bx, by, _, _) = geom(&rt, k[1]);
    let (cx, cy, _, _) = geom(&rt, k[2]);
    let (dx, dy, _, _) = geom(&rt, k[3]);
    approx(bx, 200.0, "b.x (explicit line 3 → col 2)");
    approx(by, 0.0, "b.y (row 0)");
    approx(ax, 0.0, "a.x (auto → col 0)");
    approx(ay, 0.0, "a.y (row 0)");
    approx(cx, 100.0, "c.x (auto → col 1, skipping the occupied col 2)");
    approx(cy, 0.0, "c.y (row 0)");
    approx(dx, 0.0, "d.x (auto wraps → col 0)");
    approx(dy, 10.0, "d.y (row 1, below the 10px row 0)");
}

// 7. negative end line: `.a{grid-column: 1 / -1}` spans from line 1 to the LAST
//    line (line 4 in a 3-track grid) → the whole row (x=0, width 300).
#[test]
fn negative_end_line_spans_all_columns() {
    let (rt, k) = drive(
        &format!("{THREE_COL} .a{{grid-column:1 / -1;height:10px}}"),
        1,
        400.0,
        200.0,
    );
    let (ax, ay, aw, _) = geom(&rt, k[0]);
    approx(ax, 0.0, "a.x (line 1 → col 0)");
    approx(ay, 0.0, "a.y (row 0)");
    approx(aw, 300.0, "a.width (line 1 → -1 spans all three 100px columns)");
}

// 8. negative single line: `.a{grid-column: -1}` lands on the LAST column (x=200).
#[test]
fn negative_single_line_places_in_last_column() {
    let (rt, k) = drive(
        &format!("{THREE_COL} .a{{grid-column:-1;height:10px}}"),
        1,
        400.0,
        200.0,
    );
    let (ax, _, aw, _) = geom(&rt, k[0]);
    approx(ax, 200.0, "a.x (-1 → last column, index 2)");
    approx(aw, 100.0, "a.width (single column)");
}

// 10. `grid-auto-flow: dense` backfill: two `span 2` items in a 3-column grid land
//     on rows 0 and 1, leaving a hole at (row0, col2). A trailing 1x1 auto item
//     backfills that hole under `dense` (row 0, x=200, y=0) — whereas the default
//     sparse flow leaves it on row 1 (y=10). The two flows place `.c` at different
//     y-coordinates, proving dense packing reached the engine end-to-end.
#[test]
fn dense_auto_flow_backfills_earlier_hole() {
    // Three fixed 100px columns; implicit auto rows sized by the 10px children.
    // `extra` injects `grid-auto-flow` into the container rule (empty for sparse).
    let css = |extra: &str| {
        format!(
            ".grid{{display:grid;width:300px;\
             grid-template-columns:100px 100px 100px{extra}}} \
             .a{{grid-column:span 2;height:10px}} \
             .b{{grid-column:span 2;height:10px}} .c{{height:10px}}"
        )
    };

    // Dense: `.c` backfills the hole at (row 0, col 2).
    let (rt, k) = drive(&css(";grid-auto-flow:dense"), 3, 400.0, 200.0);
    let (cx, cy, cw, _) = geom(&rt, k[2]);
    approx(cx, 200.0, "c.x (backfilled col 2)");
    approx(cy, 0.0, "c.y (dense backfills row 0's hole)");
    approx(cw, 100.0, "c.width (single col)");

    // Sparse (default): `.c` does NOT backfill — it stays on row 1.
    let (rt_s, k_s) = drive(&css(""), 3, 400.0, 200.0);
    let (sx, sy, _, _) = geom(&rt_s, k_s[2]);
    approx(sx, 200.0, "c.x (col 2 on row 1)");
    approx(sy, 10.0, "c.y (sparse leaves it on row 1)");

    // The whole point: the flows differ on the y-coordinate.
    assert!(
        (cy - sy).abs() > 1e-3,
        "dense backfill (y={cy}) must differ from sparse (y={sy})"
    );
}

// 9. `grid-area` shorthand: `.a{grid-area: 1 / 1 / 2 / -1}` → row 1, from column
//    line 1 to the last column line → the whole first row (x=0, width 300).
#[test]
fn grid_area_shorthand_spans_row_via_lines() {
    let (rt, k) = drive(
        ".grid{display:grid;width:300px;grid-template-columns:100px 100px 100px;\
         grid-template-rows:20px 20px} .a{grid-area:1 / 1 / 2 / -1;height:10px}",
        1,
        400.0,
        200.0,
    );
    let (ax, ay, aw, _) = geom(&rt, k[0]);
    approx(ax, 0.0, "a.x (column line 1 → col 0)");
    approx(ay, 0.0, "a.y (row line 1 → row 0)");
    approx(aw, 300.0, "a.width (col 1 → -1 spans all three columns)");
}

// 11. `grid-template-areas` + `grid-area: <name>`: a 2x2 grid whose template is
//     `"a b" "a c"` — area `a` fills column 0 of BOTH rows, `b` is (col1,row0),
//     `c` is (col1,row1). Children `.x{grid-area:a}` `.y{grid-area:b}`
//     `.z{grid-area:c}` resolve, in the runtime, against the container's parsed
//     template to concrete placements: `.x` at col 0 spanning both rows (x=0, two
//     rows tall), `.y` at (col1,row0), `.z` at (col1,row1).
#[test]
fn named_grid_areas_place_children_by_name() {
    // Two 100px columns; two implicit rows sized by the 10px children.
    let css = ".grid{display:grid;width:200px;grid-template-columns:100px 100px;\
               grid-template-areas:\"a b\" \"a c\"} \
               .a{grid-area:a;height:10px} .b{grid-area:b;height:10px} \
               .c{grid-area:c;height:10px}";
    let (rt, k) = drive(css, 3, 400.0, 200.0);
    let (xx, xy, xw, xh) = geom(&rt, k[0]);
    let (yx, yy, yw, _) = geom(&rt, k[1]);
    let (zx, zy, zw, _) = geom(&rt, k[2]);

    // `.x` (area `a`): column 0, spanning both rows → x=0, width 100, two 10px rows.
    approx(xx, 0.0, "x.x (area a → col 0)");
    approx(xy, 0.0, "x.y (area a starts on row 0)");
    approx(xw, 100.0, "x.width (single column 0)");
    approx(xh, 20.0, "x.height (area a spans both 10px rows)");
    // `.y` (area `b`): col 1, row 0.
    approx(yx, 100.0, "y.x (area b → col 1)");
    approx(yy, 0.0, "y.y (area b → row 0)");
    approx(yw, 100.0, "y.width (single column 1)");
    // `.z` (area `c`): col 1, row 1 (below the 10px row 0).
    approx(zx, 100.0, "z.x (area c → col 1)");
    approx(zy, 10.0, "z.y (area c → row 1)");
    approx(zw, 100.0, "z.width (single column 1)");
}
