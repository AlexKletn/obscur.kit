#![doc = include_str!("grid_css.doc/_module.md")]
#![forbid(unsafe_code)]

use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::geometry::Size;
use obscur_kit::identity::{ComponentId, NodeId, RuntimeId};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::view::view::{CLASS, View, ViewFragment, ViewNode, ViewProperty, ViewValue};

#[doc = include_str!("grid_css.doc/compile.md")]
fn compile(css: &str) -> Stylesheet {
    let src = CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&src).parse();
    StylesheetCompiler::new().compile(&parsed).matchable
}

#[doc = include_str!("grid_css.doc/tag.md")]
fn tag(node: ViewNode, cls: &str) -> ViewNode {
    node.with_prop(ViewProperty::new(CLASS, ViewValue::Text(cls.into())))
}

fn el(id: u64) -> ViewNode {
    ViewNode::element(ComponentId::from_raw(id))
}

#[doc = include_str!("grid_css.doc/CHILD_CLASSES.md")]
const CHILD_CLASSES: [&str; 5] = ["a", "b", "c", "d", "e"];

#[doc = include_str!("grid_css.doc/grid_view.md")]
fn grid_view(n: usize) -> View {
    let kids: Vec<ViewNode> = (0..n)
        .map(|i| tag(el(100 + i as u64), CHILD_CLASSES[i]))
        .collect();
    let container = tag(el(20), "grid").with_children(ViewFragment::from_nodes(kids));
    View::from_fragment(ViewFragment::from_nodes(vec![container]))
}

#[doc = include_str!("grid_css.doc/drive.md")]
fn drive(css: &str, n: usize, width: f32, height: f32) -> (UiRuntime, Vec<NodeId>) {
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(width, height));
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

#[doc = include_str!("grid_css.doc/container.md")]
fn container(rt: &UiRuntime) -> NodeId {
    let root = rt.tree().root().expect("root after a frame");
    rt.tree().get(root).unwrap().child_nodes().to_vec()[0]
}

#[doc = include_str!("grid_css.doc/geom.md")]
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
    approx(
        geom(&rt, container(&rt)).2,
        400.0,
        "container is its authored 400px",
    );
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
//
//    The two children deliberately differ. `.a` declares `height: 10px`, so its
//    block axis is NOT stretched — `stretch` sets a used size only "When the box's
//    computed width/height (as appropriate to the axis) is auto", and otherwise
//    "falls back to flex-start" (`specs/w3c/css-align-3.txt:1377`): `.a` keeps its
//    10 and sits at the row's start. `.b` leaves `height` alone and so does fill
//    its 80px row. Between them the row block is pinned twice over: `b.y = 50` is
//    row 0's height measured from the *outside* (a 10px-tall child cannot fake it)
//    and `b.height = 80` is row 1's, measured from the inside.
//
//    `.a` asserted 50 here until the `stretch` precondition was implemented, which
//    is what an item filling a track it was never told to fill looks like from the
//    outside (`docs/external-consumer-audit.md`, 2-4 (b)).
#[test]
fn explicit_fixed_rows_and_container_height() {
    let (rt, k) = drive(
        ".grid{display:grid;height:200px;grid-template-columns:1fr;grid-template-rows:50px 80px} \
         .a{height:10px}",
        2,
        300.0,
        400.0,
    );
    approx(
        geom(&rt, container(&rt)).3,
        200.0,
        "container is its authored 200px tall",
    );
    let (_, ay, aw, ah) = geom(&rt, k[0]);
    let (_, by, _, bh) = geom(&rt, k[1]);
    approx(ay, 0.0, "a.y (row0 at top)");
    approx(ah, 10.0, "a.height (its declared 10px, not the 50px row)");
    approx(aw, 300.0, "a.width (single 1fr column)");
    approx(by, 50.0, "b.y (row1, below the 50px row0)");
    approx(
        bh,
        80.0,
        "b.height (`height:auto` does stretch: the 80px row)",
    );
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

// 9. `grid-auto-rows: 40px` sizes the IMPLICIT wrapped rows: `1fr 1fr` columns
//    with 4 children wraps c,d to row 1, and that implicit row is a fixed 40px
//    (NOT sized to the 10px children), so c.y = d.y = 40.
//
//    The children keep their declared `height: 10px` — a declared size takes the
//    block axis out of `stretch` and falls back to flex-start, so each child is
//    10 tall at the top of its 40px row (`specs/w3c/css-align-3.txt:1377`). That
//    makes the row's size a *stronger* claim than it was when the children filled
//    it: `c.y = 40` is the height of row 0 seen from outside it, and only the
//    authored `grid-auto-rows` can put it there — the 10px children certainly
//    cannot. `d` is the one child left with an `auto` height, so the stretched
//    case is still covered and its 40 reads the row from the inside.
#[test]
fn grid_auto_rows_sizes_wrapped_rows() {
    let (rt, k) = drive(
        ".grid{display:grid;grid-template-columns:1fr 1fr;grid-auto-rows:40px} \
         .a{height:10px} .b{height:10px} .c{height:10px}",
        4,
        300.0,
        200.0,
    );
    let (_, ay, _, _) = geom(&rt, k[0]);
    let (_, by, _, _) = geom(&rt, k[1]);
    let (_, cy, _, ch) = geom(&rt, k[2]);
    let (_, dy, _, dh) = geom(&rt, k[3]);
    approx(ay, 0.0, "a.y (row0)");
    approx(by, 0.0, "b.y (row0)");
    approx(
        cy,
        40.0,
        "c.y (row1 — the 40px implicit auto-row, not the 10px content)",
    );
    approx(dy, 40.0, "d.y (row1)");
    approx(ch, 10.0, "c.height (its declared 10px, at the row's start)");
    approx(dh, 40.0, "d.height (`height:auto` fills the 40px row)");
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

// 10. `justify-content: center` distributes the leftover inline free space: two
//     fixed `50px` columns (100px total) in a `200px` container leave a 100px
//     leftover, so the column block is centered — the children land at x = 50/100
//     (not 0/50). The tracks keep their 50px size; only their positions shift.
#[test]
fn justify_content_center_centers_the_column_block() {
    let (rt, k) = drive(
        ".grid{display:grid;width:200px;grid-template-columns:50px 50px;justify-content:center} \
         .a{height:10px} .b{height:10px}",
        2,
        400.0,
        200.0,
    );
    approx(
        geom(&rt, container(&rt)).2,
        200.0,
        "container is its authored 200px",
    );
    let (ax, _, aw, _) = geom(&rt, k[0]);
    let (bx, _, bw, _) = geom(&rt, k[1]);
    approx(aw, 50.0, "a.width (50px fixed track keeps its size)");
    approx(bw, 50.0, "b.width (50px fixed track keeps its size)");
    approx(ax, 50.0, "a.x (centered: (200 - 100) / 2)");
    approx(bx, 100.0, "b.x (second column, after the centered first)");
}

// 11. `align-content: end` distributes the leftover block free space: two fixed
//     `50px` rows (100px total) in a `200px`-tall container leave a 100px leftover,
//     so the whole row block is pushed to the bottom — the stacked children land at
//     y = 100/150 (not 0/50).
//
//     Both children leave `height` at `auto` and so genuinely fill their rows;
//     `align-content` distributes free space between *tracks*, which is a separate
//     question from how an item is sized inside one, and this test is about the
//     former. (`.a` carried a `height: 10px` that it was nonetheless asserted to
//     stretch off — see `explicit_fixed_rows_and_container_height` for what that
//     was hiding.)
#[test]
fn align_content_end_pushes_the_row_block_to_the_bottom() {
    let (rt, k) = drive(
        ".grid{display:grid;height:200px;grid-template-columns:1fr;\
         grid-template-rows:50px 50px;align-content:end}",
        2,
        300.0,
        400.0,
    );
    approx(
        geom(&rt, container(&rt)).3,
        200.0,
        "container is its authored 200px tall",
    );
    let (_, ay, _, ah) = geom(&rt, k[0]);
    let (_, by, _, _) = geom(&rt, k[1]);
    approx(ah, 50.0, "a.height (50px fixed row keeps its size)");
    approx(ay, 100.0, "a.y (rows pushed to the bottom: 200 - 100)");
    approx(by, 150.0, "b.y (second row, after the pushed first)");
}

// --- `docs/layer-consumer-audit.md` 2-I: leaving room around a thing in a cell.

// 14. The audit's first way, and the one an author reaches for: padding on the
//     grid container. Under `box-sizing: border-box` the shell keeps the 60×60 it
//     was offered and the padding comes out of the track, so the item sits two
//     pixels inside all four edges — 56×56 at (2, 2).
//
//     The `y`/`height` half is what did not work. With no row template the single
//     implicit row is `auto`, and `specs/w3c/css-grid-2.txt:4180` stretches an
//     `auto` row to a container with a definite height; the item was nonetheless
//     handed back its own measured height, so a component came out as tall as one
//     line of its own text inside a shell that had already made room for it.
#[test]
fn a_padded_border_box_grid_insets_the_item_on_both_axes() {
    let (rt, k) = drive(
        ".grid{display:grid;width:100%;height:100%;box-sizing:border-box;\
         padding:2px;grid-template-columns:1fr}",
        1,
        60.0,
        60.0,
    );
    let (cx, cy, cw, ch) = geom(&rt, container(&rt));
    approx(cw, 60.0, "shell keeps the width it was offered");
    approx(ch, 60.0, "shell keeps the height it was offered");
    approx(cx, 0.0, "shell.x");
    approx(cy, 0.0, "shell.y");
    let (ax, ay, aw, ah) = geom(&rt, k[0]);
    approx(ax, 2.0, "a.x (the left padding)");
    approx(ay, 2.0, "a.y (the top padding)");
    approx(aw, 56.0, "a.width (60 less both paddings)");
    approx(
        ah,
        56.0,
        "a.height — the stretched `auto` row, not the item's own",
    );
}

// 15. The same shell under the DEFAULT `content-box`, where `width: 100%` is 100%
//     of the content: the border box is the offer plus the padding, and the shell
//     overflows its parent by two pixels on each edge exactly as it would in a
//     browser. The item still fills its cell and no longer reaches past the box
//     its own container reports — grid used to clamp the report to 60 while
//     sizing and placing the track at 60 from x = 2.
#[test]
fn a_content_box_grid_reports_the_padding_it_added() {
    let (rt, k) = drive(
        ".grid{display:grid;width:100%;height:100%;padding:2px;\
         grid-template-columns:1fr}",
        1,
        60.0,
        60.0,
    );
    let (_, _, cw, ch) = geom(&rt, container(&rt));
    approx(cw, 64.0, "shell border box = 60 content + 2 + 2");
    approx(ch, 64.0, "…on the block axis too");
    let (ax, ay, aw, ah) = geom(&rt, k[0]);
    approx(ax, 2.0, "a.x");
    approx(ay, 2.0, "a.y");
    approx(
        aw,
        60.0,
        "a.width (its track), inside the 64 the shell reports",
    );
    approx(ah, 60.0, "a.height (its stretched row)");
}

// 16. The audit's second way: state the margin on the thing in the cell. Grid
//     dropped a grid item's margins outright, so `margin: 2px` moved nothing and
//     the item's margin box hung two pixels outside the grid on every side
//     (`specs/w3c/css-grid-2.txt:3323`).
#[test]
fn a_margin_on_a_grid_item_leaves_room_on_all_four_sides() {
    let (rt, k) = drive(
        ".grid{display:grid;width:60px;height:60px;grid-template-columns:1fr} \
         .a{margin:2px}",
        1,
        200.0,
        200.0,
    );
    let (ax, ay, aw, ah) = geom(&rt, k[0]);
    approx(ax, 2.0, "a.x (the left margin)");
    approx(ay, 2.0, "a.y (the top margin)");
    approx(aw, 56.0, "a.width (the cell less both margins)");
    approx(ah, 56.0, "a.height (the cell less both margins)");
}

#[test]
fn empty_explicit_rows_still_size_the_grid() {
    let css = ".grid{display:grid;width:100px;grid-template-columns:100px;
               grid-template-rows:20px 30px 40px;gap:5px;padding:2px}";
    for count in [0, 1] {
        let (rt, _) = drive(css, count, 400.0, 200.0);
        let (_, _, _, height) = geom(&rt, container(&rt));
        assert_eq!(height, 104.0, "grid with {count} items");
    }
}

#[test]
fn align_content_accounts_for_unoccupied_explicit_rows() {
    let (rt, children) = drive(
        ".grid{display:grid;height:200px;grid-template-columns:100px;
               grid-template-rows:50px 50px;align-content:end}",
        1,
        300.0,
        400.0,
    );
    let (_, y, _, height) = geom(&rt, children[0]);
    assert_eq!((y, height), (100.0, 50.0));
}

#[test]
fn empty_explicit_columns_still_size_the_grid() {
    let css = ".grid{display:grid;grid-template-columns:20px 30px 40px;
               grid-template-rows:20px;gap:5px;padding:2px}";
    for count in [0, 1] {
        let (rt, _) = drive(css, count, 400.0, 200.0);
        let (_, _, width, _) = geom(&rt, container(&rt));
        assert_eq!(width, 104.0, "grid with {count} items");
    }
}

#[test]
fn empty_auto_width_grid_counts_fixed_tracks_without_stretching_fr_tracks() {
    let (rt, _) = drive(
        ".grid{display:grid;grid-template-columns:40px 1fr;gap:5px}",
        0,
        400.0,
        200.0,
    );
    assert_eq!(geom(&rt, container(&rt)).2, 45.0);
}

#[test]
fn collapsed_autofit_columns_contribute_neither_width_nor_gaps() {
    let (rt, _) = drive(
        ".grid{display:grid;grid-template-columns:40px repeat(auto-fit, 50px);gap:5px}",
        0,
        400.0,
        200.0,
    );
    assert_eq!(geom(&rt, container(&rt)).2, 40.0);
}

#[test]
fn percentage_columns_in_an_auto_grid_resolve_from_content_under_a_finite_offer() {
    let (rt, children) = drive(
        ".grid{display:grid;grid-template-columns:50% 100px}
         .a{width:40px;height:10px}",
        2,
        500.0,
        200.0,
    );
    assert_eq!(geom(&rt, container(&rt)).2, 140.0);
    assert_eq!(geom(&rt, children[1]).0, 70.0);
}

#[test]
fn autofit_collapse_releases_space_to_a_flexible_column() {
    let (rt, children) = drive(
        ".grid{display:grid;width:300px;gap:10px;
               grid-template-columns:repeat(auto-fit, 50px) minmax(0px, 1fr)}
         .b{grid-column:-2;grid-row:1}",
        2,
        400.0,
        200.0,
    );
    let (_, _, first_width, _) = geom(&rt, children[0]);
    let (flex_x, _, flex_width, _) = geom(&rt, children[1]);
    assert_eq!((first_width, flex_x, flex_width), (50.0, 60.0, 240.0));
}

#[test]
fn autofit_collapse_releases_space_to_a_flexible_row() {
    let (rt, children) = drive(
        ".grid{display:grid;width:100px;height:300px;gap:10px;
               grid-template-rows:repeat(auto-fit, 50px) minmax(0px, 1fr)}
         .b{grid-row:-2;grid-column:1}",
        2,
        400.0,
        400.0,
    );
    let (_, _, _, first_height) = geom(&rt, children[0]);
    let (_, flex_y, _, flex_height) = geom(&rt, children[1]);
    assert_eq!((first_height, flex_y, flex_height), (50.0, 60.0, 240.0));
}

#[test]
fn auto_fill_counts_minmax_columns_using_the_fixed_maximum() {
    let (rt, children) = drive(
        ".grid{display:grid;width:350px;grid-template-columns:repeat(auto-fill,minmax(40px,100px))}
         .a,.b,.c,.d{height:10px}",
        4,
        400.0,
        200.0,
    );
    let (x, y, width, _) = geom(&rt, children[3]);
    assert_eq!((x, y, width), (0.0, 10.0, 100.0));
}

#[test]
fn auto_fill_uses_a_minimum_without_a_preferred_or_maximum_width() {
    let (rt, children) = drive(
        ".grid{display:grid;min-width:205px;grid-template-columns:repeat(auto-fill,100px);gap:10px}",
        3,
        500.0,
        200.0,
    );
    assert_eq!(geom(&rt, container(&rt)).2, 210.0);
    let (x, _, _, _) = geom(&rt, children[2]);
    assert_eq!(x, 0.0);
    assert_eq!(geom(&rt, children[1]).0, 110.0);
}

#[test]
fn auto_fill_uses_an_authored_maximum_before_a_minimum() {
    let (rt, children) = drive(
        ".grid{display:grid;min-width:205px;max-width:290px;
               grid-template-columns:repeat(auto-fill,100px);gap:10px}",
        3,
        500.0,
        200.0,
    );
    assert_eq!(geom(&rt, children[1]).0, 110.0);
    assert_eq!(geom(&rt, children[2]).0, 0.0);
}

#[test]
fn auto_fill_creates_enough_empty_rows_to_reach_a_minimum_height() {
    let (rt, _) = drive(
        ".grid{display:grid;min-height:240px;grid-template-rows:repeat(auto-fill,100px);gap:10px}",
        0,
        500.0,
        400.0,
    );
    assert_eq!(geom(&rt, container(&rt)).3, 320.0);
}

#[test]
fn implicit_flexible_columns_follow_the_pattern_from_the_first_column() {
    let (rt, children) = drive(
        ".grid{display:grid;width:300px;grid-template-rows:20px;
               grid-auto-columns:1fr 2fr;grid-auto-flow:column}",
        2,
        400.0,
        200.0,
    );
    let (_, _, first_width, _) = geom(&rt, children[0]);
    let (x, _, width, _) = geom(&rt, children[1]);
    assert_eq!((first_width, x, width), (100.0, 100.0, 200.0));
}

#[test]
fn implicit_flexible_rows_share_the_available_height() {
    let (rt, children) = drive(
        ".grid{display:grid;height:300px;grid-template-columns:100px;
               grid-auto-rows:1fr 2fr}",
        2,
        400.0,
        400.0,
    );
    let (_, _, _, first_height) = geom(&rt, children[0]);
    let (_, y, _, height) = geom(&rt, children[1]);
    assert_eq!((first_height, y, height), (100.0, 100.0, 200.0));
}

#[test]
fn grid_start_packs_auto_tracks_and_normal_or_stretch_expand_them() {
    for (alignment, expected) in [
        ("", (200.0, 100.0)),
        (
            "justify-content:normal;align-content:normal",
            (200.0, 100.0),
        ),
        (
            "justify-content:stretch;align-content:stretch",
            (200.0, 100.0),
        ),
        ("justify-content:start;align-content:start", (30.0, 20.0)),
    ] {
        let css = format!(
            ".grid{{display:grid;width:200px;height:100px;
                    grid-template-columns:auto;grid-template-rows:auto;{alignment}}}
             .a,.b{{grid-column:1;grid-row:1}} .a{{width:30px;height:20px}}"
        );
        let (rt, children) = drive(&css, 2, 400.0, 200.0);
        let (_, _, width, height) = geom(&rt, children[1]);
        assert_eq!((width, height), expected, "{alignment}");
    }
}

#[test]
fn grid_template_shorthand_sets_rows_columns_areas_and_line_names() {
    let (rt, children) = drive(
        ".grid{display:grid;grid-template:[top] \"a b\" 20px [middle] \"a c\" 30px / 40px 60px}
         .a{grid-area:a} .b{grid-area:b} .c{grid-column:2;grid-row:middle}",
        3,
        400.0,
        200.0,
    );
    assert_eq!(geom(&rt, children[0]), (0.0, 0.0, 40.0, 50.0));
    assert_eq!(geom(&rt, children[1]), (40.0, 0.0, 60.0, 20.0));
    assert_eq!(geom(&rt, children[2]), (40.0, 20.0, 60.0, 30.0));
}

#[test]
fn grid_template_areas_establish_columns_when_column_sizes_are_omitted() {
    let (rt, children) = drive(
        ".grid{display:grid;grid-template:\". .\";grid-auto-columns:40px 60px}
         .a,.b{height:10px}",
        2,
        400.0,
        200.0,
    );
    assert_eq!(geom(&rt, children[1]), (40.0, 0.0, 60.0, 10.0));
}

#[test]
fn grid_template_track_form_resets_an_earlier_area_template() {
    let (rt, children) = drive(
        ".grid{display:grid;grid-template-areas:\". panel\";
               grid-template:20px / 40px 60px}
         .a{grid-area:panel}",
        1,
        400.0,
        200.0,
    );
    assert_eq!(geom(&rt, children[0]), (100.0, 20.0, 0.0, 0.0));
}

#[test]
fn grid_shorthand_sets_implicit_rows_and_preserves_gap() {
    let (rt, children) = drive(
        ".grid{display:grid;grid-auto-flow:column;gap:5px;
               grid:auto-flow 20px 30px / 50px}",
        2,
        400.0,
        200.0,
    );
    assert_eq!(geom(&rt, children[0]), (0.0, 0.0, 50.0, 20.0));
    assert_eq!(geom(&rt, children[1]), (0.0, 25.0, 50.0, 30.0));
}

#[test]
fn grid_shorthand_sets_dense_column_flow_and_implicit_columns() {
    let (rt, children) = drive(
        ".grid{display:grid;grid:20px 20px / dense auto-flow 30px 50px}
         .a{grid-row:2}",
        3,
        400.0,
        200.0,
    );
    assert_eq!(geom(&rt, children[0]), (0.0, 20.0, 30.0, 20.0));
    assert_eq!(geom(&rt, children[1]), (0.0, 0.0, 30.0, 20.0));
    assert_eq!(geom(&rt, children[2]), (30.0, 0.0, 50.0, 20.0));
}

#[test]
fn grid_shorthand_template_form_resets_earlier_implicit_properties() {
    let (rt, children) = drive(
        ".grid{display:grid;grid-auto-flow:column;grid-auto-rows:90px;grid:20px / 40px}
         .b{height:10px}",
        2,
        400.0,
        200.0,
    );
    assert_eq!(geom(&rt, children[1]), (0.0, 20.0, 40.0, 10.0));
}

#[test]
fn fit_content_track_preserves_the_item_minimum_and_does_not_stretch() {
    let (rt, children) = drive(
        ".grid { display: grid; width: 200px; grid-template-columns: fit-content(25%) 1fr; }
         .a { width: 70px; }",
        2,
        800.0,
        600.0,
    );
    approx(
        geom(&rt, children[1]).0,
        70.0,
        "next track starts at the intrinsic floor",
    );

    let (rt, children) = drive(
        ".grid { display: grid; width: 200px; grid-template-columns: fit-content(100px) auto; }
         .a { width: 30px; }",
        2,
        800.0,
        600.0,
    );
    approx(
        geom(&rt, children[1]).0,
        30.0,
        "only the auto track stretches",
    );
}

#[test]
fn fit_content_math_uses_the_grid_width_and_resolves_relative_lengths() {
    let words = ViewNode::text("word word word word word word word word word word");
    let item = tag(el(100), "a").with_children(ViewFragment::from_nodes(vec![words]));
    let marker = tag(el(101), "b");
    let grid = tag(el(20), "grid").with_children(ViewFragment::from_nodes(vec![item, marker]));
    let view = View::from_fragment(ViewFragment::from_nodes(vec![grid]));
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(800.0, 600.0));
    let sheet = compile(
        ".grid { display: grid; width: 200px; font-size: 20px;
                 grid-template-columns: fit-content(clamp(30px, 50% - 1em, 90px)) 1fr; }",
    );
    let _ = rt.render_now(&view, &[sheet]);
    let children = rt
        .tree()
        .get(container(&rt))
        .unwrap()
        .child_nodes()
        .to_vec();
    approx(
        geom(&rt, children[1]).0,
        80.0,
        "math cap is 100px minus 20px",
    );
}

#[test]
fn grid_css_aligns_first_text_baselines_through_nested_boxes() {
    let text = |id, class| {
        tag(el(id), class).with_children(ViewFragment::from_nodes(vec![ViewNode::text("Text")]))
    };
    let small =
        tag(el(100), "small").with_children(ViewFragment::from_nodes(vec![text(102, "label")]));
    let large = text(101, "large");
    let grid = tag(el(20), "grid").with_children(ViewFragment::from_nodes(vec![small, large]));
    let view = View::from_fragment(ViewFragment::from_nodes(vec![grid]));
    for alignment in [
        ".grid { align-items: baseline; }",
        ".small, .large { align-self: first baseline; }",
        ".grid { place-items: first baseline stretch; }",
        ".small, .large { place-self: baseline first stretch; }",
    ] {
        let css = format!(
            ".grid {{ display: grid; grid-template-columns: 100px 100px; }}
             .small {{ font-size: 10px; }}
             .label {{ padding-top: 2px; }}
             .large {{ font-size: 20px; }}
             {alignment}"
        );
        let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(800.0, 600.0));
        let _ = rt.render_now(&view, &[compile(&css)]);
        let children = rt
            .tree()
            .get(container(&rt))
            .unwrap()
            .child_nodes()
            .to_vec();
        approx(geom(&rt, children[0]).1, 6.0, "16px baseline minus 10px");
        approx(
            geom(&rt, children[1]).1,
            0.0,
            "leading baseline stays in place",
        );
    }
}

#[test]
fn grid_css_aligns_last_baselines_of_single_and_multiline_text() {
    let small =
        tag(el(100), "small").with_children(ViewFragment::from_nodes(vec![ViewNode::text("One")]));
    let large = tag(el(101), "large").with_children(ViewFragment::from_nodes(vec![tag(
        el(102).with_children(ViewFragment::from_nodes(vec![ViewNode::text(
            "First\nLast",
        )])),
        "nested",
    )]));
    let grid = tag(el(20), "grid").with_children(ViewFragment::from_nodes(vec![small, large]));
    let view = View::from_fragment(ViewFragment::from_nodes(vec![grid]));
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(800.0, 600.0));
    let sheet = compile(
        ".grid { display: grid; grid-template-columns: 100px 100px; grid-template-rows: 100px;
                 align-items: last baseline; }
         .small { font-size: 10px; }
         .large { font-size: 20px; white-space: pre; align-self: last baseline; }",
    );
    let _ = rt.render_now(&view, &[sheet]);
    let children = rt
        .tree()
        .get(container(&rt))
        .unwrap()
        .child_nodes()
        .to_vec();
    approx(
        geom(&rt, children[0]).1,
        88.0,
        "small text baseline at 96px",
    );
    approx(geom(&rt, children[1]).1, 60.0, "last line baseline at 96px");
}
