#![doc = include_str!("grid_placement_css.doc/_module.md")]
#![forbid(unsafe_code)]

use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::geometry::Size;
use obscur_kit::identity::{ComponentId, NodeId, RuntimeId};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::view::view::{CLASS, View, ViewFragment, ViewNode, ViewProperty, ViewValue};

#[doc = include_str!("grid_placement_css.doc/compile.md")]
fn compile(css: &str) -> Stylesheet {
    let src = CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&src).parse();
    StylesheetCompiler::new().compile(&parsed).matchable
}

#[doc = include_str!("grid_placement_css.doc/tag.md")]
fn tag(node: ViewNode, cls: &str) -> ViewNode {
    node.with_prop(ViewProperty::new(CLASS, ViewValue::Text(cls.into())))
}

fn el(id: u64) -> ViewNode {
    ViewNode::element(ComponentId::from_raw(id))
}

#[doc = include_str!("grid_placement_css.doc/CHILD_CLASSES.md")]
const CHILD_CLASSES: [&str; 5] = ["a", "b", "c", "d", "e"];

#[doc = include_str!("grid_placement_css.doc/grid_view.md")]
fn grid_view(n: usize) -> View {
    let kids: Vec<ViewNode> = (0..n)
        .map(|i| tag(el(100 + i as u64), CHILD_CLASSES[i]))
        .collect();
    let container = tag(el(20), "grid").with_children(ViewFragment::from_nodes(kids));
    View::from_fragment(ViewFragment::from_nodes(vec![container]))
}

#[doc = include_str!("grid_placement_css.doc/drive.md")]
fn drive(css: &str, n: usize, width: f32, height: f32) -> (UiRuntime, Vec<NodeId>) {
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(width, height));
    rt.set_custom_value_parser(std::rc::Rc::new(
        obscur_kit::css::compiler::CustomValueCache::new(),
    ));
    let sheet = compile(css);
    let view = grid_view(n);
    let result = rt.render_now(&view, std::slice::from_ref(&sheet));
    assert!(
        result.restyled(),
        "authored grid CSS resolved computed styles"
    );
    assert!(result.relaid_out(), "the styled tree arranged layout boxes");
    let root = rt.tree().root().expect("root after a frame");
    let container = rt.tree().get(root).unwrap().child_nodes().to_vec()[0];
    let kids = rt.tree().get(container).unwrap().child_nodes().to_vec();
    (rt, kids)
}

#[doc = include_str!("grid_placement_css.doc/geom.md")]
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

#[doc = include_str!("grid_placement_css.doc/THREE_COL.md")]
const THREE_COL: &str = ".grid{display:grid;width:300px;grid-template-columns:100px 100px 100px}";

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
//    auto `.b` continues forward from there into col 2 (x=200).
#[test]
fn explicit_column_line_advances_the_sparse_cursor() {
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
    approx(bx, 200.0, "b.x (auto continues to col 2)");
    approx(by, 0.0, "b.y (row 0)");
}

// 4. explicit row: `.a{grid-row: 2}` with `grid-template-rows: 50px 60px` puts
//    `.a` on row 1 at y=50; the row-1 track is 60px tall.
//
//    `.a` states no `height`, which is what lets its height report the track at
//    all: `stretch` sizes a box only "When the box's computed width/height (as
//    appropriate to the axis) is auto" (`specs/w3c/css-align-3.txt:1377`). It used
//    to carry a `height: 10px` and be asserted at 60 anyway — the row was read
//    through an item that should have kept its 10.
#[test]
fn explicit_row_line_places_on_second_row() {
    let (rt, k) = drive(
        ".grid{display:grid;width:300px;grid-template-columns:100px 100px 100px;\
         grid-template-rows:50px 60px} .a{grid-row:2}",
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
//    `.d` follow the cursor row-major (a→col0, c→row1 col0, d→row1 col1).
#[test]
fn mixed_explicit_and_auto_items_follow_the_sparse_cursor() {
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
    approx(cx, 0.0, "c.x (auto wraps → col 0)");
    approx(cy, 10.0, "c.y (row 1)");
    approx(dx, 100.0, "d.x (auto → col 1)");
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
    approx(
        aw,
        300.0,
        "a.width (line 1 → -1 spans all three 100px columns)",
    );
}

// 8. A negative single line starts an implicit column after the explicit grid.
#[test]
fn negative_single_line_places_after_the_last_explicit_column() {
    let (rt, k) = drive(
        &format!("{THREE_COL} .grid{{grid-auto-columns:40px}} .a{{grid-column:-1;height:10px}}"),
        1,
        400.0,
        200.0,
    );
    let (ax, _, aw, _) = geom(&rt, k[0]);
    approx(ax, 300.0, "a.x starts at the final explicit line");
    approx(aw, 40.0, "a.width follows its implicit column");
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
    // Two 100px columns; two implicit rows sized by the 10px children `.b`/`.c`.
    // `.x` states no `height` on purpose: only a box whose computed height is
    // `auto` is stretched (`specs/w3c/css-align-3.txt:1377`), and its height is
    // this test's evidence that area `a` spans both rows. It carried a
    // `height: 10px` and was asserted at 20 regardless, which read the span through
    // a box that should have kept its 10.
    let css = ".grid{display:grid;width:200px;grid-template-columns:100px 100px;\
               grid-template-areas:\"a b\" \"a c\"} \
               .a{grid-area:a} .b{grid-area:b;height:10px} \
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

// 12. named grid lines: the container names its column lines
//     `[a] 100px [b] 100px [c] 100px [d]` (a=1, b=2, c=3, d=4). A child
//     `.a{grid-column:a / c}` resolves, in the runtime, against the parent's
//     line-name map to line 1 / line 3 → columns 0-1 (x=0, width 200).
#[test]
fn named_grid_lines_place_child_by_line_name() {
    let css = ".grid{display:grid;width:300px;\
               grid-template-columns:[a] 100px [b] 100px [c] 100px [d]} \
               .a{grid-column:a / c;height:10px} .b{height:10px}";
    let (rt, k) = drive(css, 2, 400.0, 200.0);
    let (ax, ay, aw, _) = geom(&rt, k[0]);
    let (bx, _, bw, _) = geom(&rt, k[1]);
    approx(ax, 0.0, "a.x (line a=1 → col 0)");
    approx(ay, 0.0, "a.y (row 0)");
    approx(aw, 200.0, "a.width (lines a→c span cols 0-1 = 100+100)");
    approx(bx, 200.0, "b.x (auto flows to the free col 2)");
    approx(bw, 100.0, "b.width (single col)");
}

// ---- Grid item alignment within a cell (`justify-self` / `align-self` and the
//      `place-self` shorthand), end-to-end through the real pipeline. ----

#[doc = include_str!("grid_placement_css.doc/ONE_CELL.md")]
const ONE_CELL: &str =
    ".grid{display:grid;width:100px;grid-template-columns:100px;grid-template-rows:100px}";

// A `40×30` item in a `100×100` cell with `justify-self:center` (inline/x) and
// `align-self:end` (block/y): x centered at (100-40)/2 = 30, y at the cell end
// 100-30 = 70, keeping its natural 40×30 size (no stretch to the cell).
#[test]
fn justify_and_align_self_position_item_within_cell() {
    let (rt, k) = drive(
        &format!("{ONE_CELL} .a{{width:40px;height:30px;justify-self:center;align-self:end}}"),
        1,
        400.0,
        200.0,
    );
    let (ax, ay, aw, ah) = geom(&rt, k[0]);
    approx(ax, 30.0, "a.x (justify-self:center → (100-40)/2)");
    approx(ay, 70.0, "a.y (align-self:end → 100-30)");
    approx(aw, 40.0, "a.width (natural, not stretched)");
    approx(ah, 30.0, "a.height (natural, not stretched)");
}

// The `place-self: <align> <justify>` shorthand: `end center` → `align-self:end`
// (block/y = 70) + `justify-self:center` (inline/x = 30), the same geometry as
// the two longhands, proving the shorthand expansion reaches layout end-to-end.
#[test]
fn place_self_shorthand_positions_item_within_cell() {
    let (rt, k) = drive(
        &format!("{ONE_CELL} .a{{width:40px;height:30px;place-self:end center}}"),
        1,
        400.0,
        200.0,
    );
    let (ax, ay, aw, ah) = geom(&rt, k[0]);
    approx(ax, 30.0, "a.x (place-self justify side = center)");
    approx(ay, 70.0, "a.y (place-self align side = end)");
    approx(aw, 40.0, "a.width (natural)");
    approx(ah, 30.0, "a.height (natural)");
}

// A container `justify-items:center` default applies to an item that leaves
// `justify-self` at its `auto` default: x centered (30), while the block axis
// keeps the container's default `stretch` (y=0, height fills the 100px cell).
#[test]
fn container_justify_items_default_centers_auto_item() {
    let (rt, k) = drive(
        // No `height`: the block axis is the one this test wants stretched, and a
        // declared height would take it out of `stretch` altogether
        // (`specs/w3c/css-align-3.txt:1377`). The inline axis is centred by the
        // container regardless of the declared `width`.
        &format!("{ONE_CELL} .grid{{justify-items:center}} .a{{width:40px}}"),
        1,
        400.0,
        200.0,
    );
    let (ax, ay, aw, ah) = geom(&rt, k[0]);
    approx(ax, 30.0, "a.x (container justify-items:center)");
    approx(ay, 0.0, "a.y (align-items default stretch → cell top)");
    approx(aw, 40.0, "a.width (natural, inline axis centered)");
    approx(
        ah,
        100.0,
        "a.height (block axis stretches to fill the cell)",
    );
}

// Regression: with no alignment authored, an item whose size is `auto` on both
// axes fills its whole cell (the default `stretch` on both axes).
#[test]
fn default_grid_item_still_fills_its_cell() {
    let (rt, k) = drive(&format!("{ONE_CELL} .a{{}}"), 1, 400.0, 200.0);
    let (ax, ay, aw, ah) = geom(&rt, k[0]);
    approx(ax, 0.0, "a.x (cell origin)");
    approx(ay, 0.0, "a.y (cell origin)");
    approx(aw, 100.0, "a.width (stretched to the 100px cell)");
    approx(ah, 100.0, "a.height (stretched to the 100px cell)");
}

// …and an item that declares a size does NOT, with the very same (absent)
// alignment. `specs/w3c/css-align-3.txt:1377` gives `stretch` a precondition:
// it sets a used size "When the box's computed width/height (as appropriate to
// the axis) is auto and neither of its margins (in the appropriate axis) are
// auto", and "Unless otherwise specified, this value falls back to flex-start
// generally". So the declared 40×30 survives and the box sits at the cell's
// start corner — a real flex-start, position and size together.
//
// This test above used to author `width:40px;height:30px` and assert 100×100:
// the engine stretched a box it was not allowed to stretch, and the author's
// lengths vanished (`docs/external-consumer-audit.md`, 2-4 (b)).
#[test]
fn a_declared_size_survives_the_default_stretch() {
    let (rt, k) = drive(
        &format!("{ONE_CELL} .a{{width:40px;height:30px}}"),
        1,
        400.0,
        200.0,
    );
    let (ax, ay, aw, ah) = geom(&rt, k[0]);
    approx(ax, 0.0, "a.x (flex-start fallback → cell origin)");
    approx(ay, 0.0, "a.y (flex-start fallback → cell origin)");
    approx(aw, 40.0, "a.width (its declared 40px, not the 100px cell)");
    approx(ah, 30.0, "a.height (its declared 30px, not the 100px cell)");
}

// The same box, with `place-self: stretch` written out by hand: identical. The
// spec's condition is on the box's computed size, not on how the alignment value
// was arrived at, so asking for the stretch out loud buys nothing the keyword is
// not defined to give.
#[test]
fn an_authored_stretch_does_not_beat_a_declared_size() {
    let (rt, k) = drive(
        &format!("{ONE_CELL} .a{{width:40px;height:30px;place-self:stretch}}"),
        1,
        400.0,
        200.0,
    );
    let (ax, ay, aw, ah) = geom(&rt, k[0]);
    approx(ax, 0.0, "a.x");
    approx(ay, 0.0, "a.y");
    approx(aw, 40.0, "a.width (authored `stretch` still falls back)");
    approx(ah, 30.0, "a.height (authored `stretch` still falls back)");
}

#[test]
fn template_areas_generate_named_lines_on_both_axes() {
    let css = ".grid{display:grid;width:300px;               grid-template-columns:100px 100px [panel-start] 100px;               grid-template-rows:20px 30px;               grid-template-areas:\". . .\" \". panel panel\"}                .a{grid-column:panel-start / panel-end;grid-row:panel-start / panel-end}";
    let (rt, children) = drive(css, 1, 400.0, 200.0);
    let (x, y, width, height) = geom(&rt, children[0]);
    approx(x, 100.0, "first panel-start line wins");
    approx(y, 20.0, "panel block start");
    approx(width, 200.0, "panel inline span");
    approx(height, 30.0, "panel block span");
}

#[test]
fn template_areas_none_clears_an_earlier_template() {
    let css = ".grid{display:grid;width:200px;grid-template-columns:100px 100px;
               grid-template-areas:\". panel\";grid-template-areas:none}
               .a{grid-area:panel}";
    let (rt, children) = drive(css, 1, 400.0, 200.0);
    let (x, _, _, _) = geom(&rt, children[0]);
    approx(
        x,
        200.0,
        "the name resolves beyond the explicit grid after the area is cleared",
    );
}

#[test]
fn numbered_named_lines_count_distinct_lines_from_either_end() {
    let css = ".grid{display:grid;width:400px;
               grid-template-columns:[panel-start] 100px [panel-start panel-start] 100px 100px [panel-start] 100px;
               grid-template-areas:\". . panel panel\"}
               .a{grid-column:panel-start 2 / span 1}
               .b{grid-column:-2 panel-start / span 1}
               .c{grid-column-start:3 panel-start;grid-column-end:span 1}";
    let (rt, children) = drive(css, 3, 500.0, 200.0);
    approx(geom(&rt, children[0]).0, 100.0, "one occurrence per line");
    approx(
        geom(&rt, children[1]).0,
        200.0,
        "second from end includes generated line",
    );
    approx(
        geom(&rt, children[2]).0,
        200.0,
        "third from start includes generated line",
    );
}

#[test]
fn column_auto_flow_respects_definite_cells_and_moves_along_a_fixed_row() {
    let css = ".grid{display:grid;width:150px;grid-template-columns:50px;
               grid-template-rows:20px 20px;grid-auto-columns:50px;grid-auto-flow:column}
               .a{grid-row:1}
               .b{grid-column:1}
               .c{grid-column:1;grid-row:1}
               .d{grid-row:1}";
    let (rt, children) = drive(css, 4, 400.0, 200.0);
    let positions: Vec<_> = children
        .iter()
        .map(|&node| {
            let (x, y, _, _) = geom(&rt, node);
            (x, y)
        })
        .collect();
    assert_eq!(
        positions,
        [(50.0, 0.0), (0.0, 20.0), (0.0, 0.0), (100.0, 0.0)]
    );
}

#[test]
fn column_dense_can_backfill_a_hole_for_items_locked_to_a_column() {
    fn last_y(flow: &str) -> f32 {
        let css = format!(
            ".grid{{display:grid;width:50px;grid-template-columns:50px;
                   grid-template-rows:repeat(5, 20px);grid-auto-flow:{flow}}}
             .a{{grid-column:1;grid-row:span 2}} .b{{grid-column:1;grid-row:2}}
             .c{{grid-column:1}}"
        );
        let (rt, children) = drive(&css, 3, 400.0, 200.0);
        geom(&rt, children[2]).1
    }
    assert_eq!(last_y("column"), 80.0);
    assert_eq!(last_y("column dense"), 0.0);
}

#[test]
fn row_auto_flow_reserves_definite_cells_and_expands_for_items_locked_to_a_row() {
    let css = ".grid{display:grid;width:150px;grid-template-columns:50px;
               grid-template-rows:20px;grid-auto-columns:50px;grid-auto-rows:20px}
               .a{grid-row:1} .b{grid-column:1}
               .c{grid-column:1;grid-row:1} .d{grid-row:1}";
    let (rt, children) = drive(css, 4, 400.0, 200.0);
    let positions: Vec<_> = children
        .iter()
        .map(|&node| {
            let (x, y, _, _) = geom(&rt, node);
            (x, y)
        })
        .collect();
    assert_eq!(
        positions,
        [(50.0, 0.0), (0.0, 20.0), (0.0, 0.0), (100.0, 0.0)]
    );
}

#[test]
fn fixed_columns_wrap_the_sparse_row_flow_cursor() {
    fn last_y(flow: &str) -> f32 {
        let css = format!(
            ".grid{{display:grid;width:150px;grid-template-columns:repeat(3, 50px);
                   grid-auto-rows:20px;grid-auto-flow:{flow}}}
             .a{{grid-column:3}} .b{{grid-column:1}}"
        );
        let (rt, children) = drive(&css, 2, 400.0, 200.0);
        geom(&rt, children[1]).1
    }
    assert_eq!(last_y("row"), 20.0);
    assert_eq!(last_y("row dense"), 0.0);
}

#[test]
fn numbered_grid_lines_resolve_names_from_numeric_repeats() {
    let css = ".grid{display:grid;grid-template-columns:repeat(3,[start] 40px [end]);gap:5px}
               .a{grid-column:start 3 / end 3;height:10px}";
    let (rt, children) = drive(css, 1, 400.0, 200.0);
    assert_eq!(geom(&rt, children[0]), (90.0, 0.0, 40.0, 10.0));
}

#[test]
fn automatic_repeat_names_and_trailing_lines_follow_the_expanded_track_count() {
    let css = ".grid{display:grid;width:100%;
               grid-template-columns:repeat(auto-fill,[slot] 40px [slot]) [tail] 20px}
               .a{grid-column:slot 3;grid-row:1;height:10px}
               .b{grid-column:tail;grid-row:1;height:10px}";
    let (mut rt, children) = drive(css, 2, 170.0, 200.0);
    assert_eq!(geom(&rt, children[0]), (80.0, 0.0, 40.0, 10.0));
    assert_eq!(geom(&rt, children[1]), (120.0, 0.0, 20.0, 10.0));

    rt.set_viewport(Size::new(210.0, 200.0));
    rt.render_now(&grid_view(2), &[compile(css)]);
    assert_eq!(geom(&rt, children[0]), (80.0, 0.0, 40.0, 10.0));
    assert_eq!(geom(&rt, children[1]), (160.0, 0.0, 20.0, 10.0));
}

#[test]
fn auto_fit_resolves_named_lines_before_collapsing_empty_tracks() {
    let css = ".grid{display:grid;width:200px;
               grid-template-columns:repeat(auto-fit,[slot] 40px)}
               .a{grid-column:-2 slot;grid-row:1;height:10px}
               .b{grid-column:slot;grid-row:1;height:10px}";
    let (rt, children) = drive(css, 2, 400.0, 200.0);
    assert_eq!(geom(&rt, children[0]), (40.0, 0.0, 40.0, 10.0));
    assert_eq!(geom(&rt, children[1]), (0.0, 0.0, 40.0, 10.0));
}

#[test]
fn numbered_missing_matches_extend_the_grid_on_both_sides() {
    let css = ".grid{display:grid;
               grid-template-columns:[mark] 100px;grid-auto-columns:20px 30px}
               .a{grid-column:-3 mark;grid-row:1;height:10px}
               .b{grid-column:1;grid-row:1;height:10px}
               .c{grid-column:mark 3;grid-row:1;height:10px}";
    let (rt, children) = drive(css, 3, 400.0, 200.0);
    assert_eq!(geom(&rt, children[0]), (0.0, 0.0, 20.0, 10.0));
    assert_eq!(geom(&rt, children[1]), (50.0, 0.0, 100.0, 10.0));
    assert_eq!(geom(&rt, children[2]), (200.0, 0.0, 20.0, 10.0));
}

#[test]
fn numbered_names_count_implicit_rows_after_expanding_auto_repeat() {
    let css = ".grid{display:grid;width:50px;height:80px;grid-template-columns:50px;
               grid-template-rows:repeat(auto-fill,[mark] 40px);grid-auto-rows:20px}
               .a{grid-row:mark 3;grid-column:1}
               .b{grid-row:-3 mark;grid-column:1}";
    let (rt, children) = drive(css, 2, 400.0, 200.0);
    assert_eq!(geom(&rt, children[0]), (0.0, 120.0, 50.0, 20.0));
    assert_eq!(geom(&rt, children[1]), (0.0, 0.0, 50.0, 20.0));
}

#[test]
fn a_bare_column_name_prefers_the_area_boundaries_over_a_matching_line() {
    let css = ".grid{display:grid;grid-template-columns:[panel] 20px 30px 40px;
               grid-template-areas:\". panel panel\"}
               .a{grid-column:panel;grid-row:1;height:10px}";
    let (rt, children) = drive(css, 1, 400.0, 200.0);
    assert_eq!(geom(&rt, children[0]), (20.0, 0.0, 70.0, 10.0));
}

#[test]
fn bare_names_resolve_preferred_repeat_edges_and_unknown_implicit_lines() {
    let css = ".grid{display:grid;width:180px;
               grid-template-columns:20px repeat(auto-fill,[panel-start] 40px [panel-end]);
               grid-auto-columns:15px}
               .a{grid-column:panel;grid-row:1;height:10px}
               .b{grid-column:missing;grid-row:1;height:10px}";
    let (rt, children) = drive(css, 2, 400.0, 200.0);
    assert_eq!(geom(&rt, children[0]), (20.0, 0.0, 40.0, 10.0));
    assert_eq!(geom(&rt, children[1]), (195.0, 0.0, 15.0, 10.0));
}

#[test]
fn named_grid_area_edges_obey_later_longhands() {
    let css = ".grid{display:grid;grid-template-columns:30px 40px 50px;
               grid-template-rows:20px;grid-template-areas:\"panel panel .\"}
               .a{grid-area:panel;grid-column-start:3;grid-column-end:4}";
    let (rt, children) = drive(css, 1, 400.0, 200.0);
    assert_eq!(geom(&rt, children[0]), (70.0, 0.0, 50.0, 20.0));
}

#[test]
fn reversed_grid_lines_span_the_same_tracks_as_forward_lines() {
    let css = ".grid{display:grid;grid-template-columns:30px 40px 50px;
               grid-template-rows:20px 30px 40px;gap:5px}
               .a{grid-column:4 / 2;grid-row:-1 / 2}
               .b{grid-column:2 / 4;grid-row:2 / -1}";
    let (rt, children) = drive(css, 2, 400.0, 200.0);
    assert_eq!(geom(&rt, children[0]), (35.0, 25.0, 95.0, 75.0));
    assert_eq!(geom(&rt, children[1]), geom(&rt, children[0]));
}

#[test]
fn numeric_placement_can_add_columns_before_the_first_explicit_line() {
    let css = ".grid{display:grid;grid-template-columns:50px 60px;grid-auto-columns:20px 30px}
               .a{grid-column:-5;grid-row:1;height:10px}
               .b{grid-column:1;grid-row:1;height:10px}
               .c{grid-column:span 2 / 1;grid-row:2;height:10px}";
    let (rt, children) = drive(css, 3, 400.0, 200.0);
    assert_eq!(geom(&rt, children[0]), (0.0, 0.0, 20.0, 10.0));
    assert_eq!(geom(&rt, children[1]), (50.0, 0.0, 50.0, 10.0));
    assert_eq!(geom(&rt, children[2]), (0.0, 10.0, 50.0, 10.0));
}

#[test]
fn grid_auto_placement_uses_order_groups_and_source_order_for_ties() {
    let css = ".grid{display:grid;grid-template-columns:repeat(3,40px);grid-auto-rows:20px}
               .a{order:2} .b{order:-1} .d{--priority:-1;order:var(--priority)} .e{order:2}";
    let (rt, children) = drive(css, 5, 400.0, 200.0);
    let positions: Vec<_> = children
        .iter()
        .map(|&node| {
            let (x, y, _, _) = geom(&rt, node);
            (x, y)
        })
        .collect();
    assert_eq!(
        positions,
        [
            (0.0, 20.0),
            (0.0, 0.0),
            (80.0, 0.0),
            (40.0, 0.0),
            (40.0, 20.0)
        ]
    );
}

#[test]
fn grid_order_also_applies_to_items_locked_to_a_row() {
    let css = ".grid{display:grid;grid-template-columns:repeat(2,40px);grid-auto-rows:20px}
               .a,.b{grid-row:1} .b{order:-1}";
    let (rt, children) = drive(css, 2, 400.0, 200.0);
    assert_eq!(geom(&rt, children[0]).0, 40.0);
    assert_eq!(geom(&rt, children[1]).0, 0.0);
}
