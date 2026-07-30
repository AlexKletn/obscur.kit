//! A grid item's children resolve their percentages against the **item's** box —
//! because a grid item's containing block is its grid area, not the grid
//! container's content box.
//!
//! `specs/w3c/css-grid-2.txt:712`, vendored here: "A grid item's grid area forms
//! the containing block into which it is laid out." §6.2 (`:1042`) says it again
//! from the sizing side: "A grid item is sized within the containing block defined
//! by its grid area."
//!
//! The defect this pins: the arrange walk handed every child of a node the same
//! percentage basis — that node's content box. That *is* the containing block for
//! a block child and a flex item, so block flow and flex were both right and only
//! grid was wrong. A `width: 100%` grid item came out at its track (correct) while
//! everything below it resolved against the whole grid container's content box, a
//! containing block the item never had: the item's used size and the size its
//! children saw disagreed, and the child's number tracked the shell's *padding*
//! rather than its parent's width. The grid algorithm now states each item's grid
//! area alongside the item's box, and the runtime prefers it
//! (`LayoutOutput::child_percent_basis`).
//!
//! The geometry below is the external-consumer audit's, measured through this
//! facade (`docs/external-consumer-audit.md`, 2-4).
#![forbid(unsafe_code)]

use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::geometry::Size;
use obscur_kit::identity::{ComponentId, NodeId, RuntimeId};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::view::view::{CLASS, View, ViewFragment, ViewNode, ViewProperty, ViewValue};

/// The audit's viewport.
const VIEWPORT: Size = Size {
    width: 480.0,
    height: 240.0,
};

/// The audit's shell: a `border-box` grid filling the viewport with 24px of
/// padding — a 432 content box — and two `1fr` columns split by a 10px gap, so
/// each track is (432 − 10) / 2 = 211.
const SHELL: &str = ".app { width: 100%; height: 100%; box-sizing: border-box; \
                     padding-left: 24px; padding-right: 24px; \
                     padding-top: 24px; padding-bottom: 24px; \
                     display: grid; grid-template-columns: 1fr 1fr; gap: 10px } \
                     .kid { width: 50% }";

fn compile(css: &str) -> Stylesheet {
    let src = CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&src).parse();
    StylesheetCompiler::new().compile(&parsed).matchable
}

fn el(class: &str, kids: Vec<ViewNode>) -> ViewNode {
    ViewNode::element(ComponentId::from_raw(81))
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text(class.into())))
        .with_children(ViewFragment::from_nodes(kids))
}

/// `.app > .item > .kid` — one grid item holding one percentage-sized child, the
/// shape the audit's `<span>` first showed the defect in.
fn view() -> View {
    View::from_fragment(ViewFragment::from_nodes(vec![el(
        "app",
        vec![el("item", vec![el("kid", vec![])])],
    )]))
}

/// Lays the shell out with `.item { <item_rule> }` and returns the used **sizes**
/// down the single spine: `[.app, .item, .kid]`.
fn sizes(item_rule: &str) -> Vec<Size> {
    let sheet = compile(&format!("{SHELL} .item {{ {item_rule} }}"));
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), VIEWPORT);
    let _ = rt.render_now(&view(), std::slice::from_ref(&sheet));

    let mut out = Vec::new();
    let mut node: NodeId = rt.tree().root().expect("a root after a frame");
    loop {
        let kids = rt.tree().get(node).expect("live").child_nodes().to_vec();
        let Some(&child) = kids.first() else { break };
        out.push(
            rt.layout_boxes()
                .get(&child)
                .expect("an arranged box")
                .size(),
        );
        node = child;
    }
    out
}

/// [`sizes`], projected onto the inline axis — the column the audit's table
/// measures.
fn widths(item_rule: &str) -> Vec<f32> {
    sizes(item_rule).into_iter().map(|s| s.width).collect()
}

/// The row that was wrong: a `width: 100%` item is its track (211), and its child
/// is half of **that** — not half of the grid container's 432.
#[test]
fn a_percentage_item_hands_its_own_track_down_to_its_children() {
    let widths = widths("width: 100%");
    assert!((widths[0] - 480.0).abs() < 0.01, "app {}", widths[0]);
    assert!((widths[1] - 211.0).abs() < 0.01, "item {}", widths[1]);
    // 105.5 = 50% of the 211 track. Before the fix: 216 = 50% of 432, the grid
    // container's content box — and it followed the shell's padding (0 → 240,
    // 24 → 216, 50 → 190) instead of its parent's width.
    assert!((widths[2] - 105.5).abs() < 0.01, "kid {}", widths[2]);
}

/// The control: an `auto` item never went through the item's own percentage and
/// was always right. It reads exactly as it did before the fix, which is how the
/// fix is known to have moved only what was broken.
#[test]
fn an_auto_item_was_never_affected() {
    let widths = widths("width: auto");
    assert!((widths[1] - 211.0).abs() < 0.01, "item {}", widths[1]);
    assert!((widths[2] - 105.5).abs() < 0.01, "kid {}", widths[2]);
}

/// The audit's third row: a `width: 50px` item keeps its declared 50.
///
/// This was the audit's other defect (2-4 (b)), and it used to read 211 — the
/// whole track. `specs/w3c/css-align-3.txt:1377` scopes `stretch` to a box "When
/// the box's computed width/height (as appropriate to the axis) **is auto**",
/// falling back to `flex-start` otherwise, so an item with a width of its own is
/// never stretched: it keeps that width and sits at the start of its grid area.
/// The child's 25 is half of the declared 50 and never moved — this row's child
/// was never the problem, which is exactly why the two defects had to be pinned
/// apart.
#[test]
fn a_fixed_width_item_keeps_its_declared_width() {
    let widths = widths("width: 50px");
    assert!((widths[1] - 50.0).abs() < 0.01, "item {}", widths[1]);
    assert!((widths[2] - 25.0).abs() < 0.01, "kid {}", widths[2]);
}

/// Writing `justify-self: stretch` / `align-self: stretch` by hand changes
/// nothing. The spec's condition is about the box's **computed size**, not about
/// how the alignment value was arrived at, so the author who asks for the stretch
/// out loud gets the same flex-start fallback as the one who inherits it from the
/// container's default.
#[test]
fn an_authored_stretch_does_not_override_a_declared_size() {
    let widths = widths("width: 50px; justify-self: stretch; align-self: stretch");
    assert!((widths[1] - 50.0).abs() < 0.01, "item {}", widths[1]);
    assert!((widths[2] - 25.0).abs() < 0.01, "kid {}", widths[2]);
}

/// The rule is per axis ("as appropriate to the axis"), so a declared `height` is
/// kept the same way a declared `width` is, and the *other* axis keeps stretching.
///
/// The row here is content-sized, so the block extent an unconstrained item would
/// take is its own content height; `height: 40px` must therefore read 40 while the
/// inline axis still fills the 211 track.
#[test]
fn a_declared_height_survives_on_the_block_axis_alone() {
    let item = sizes("height: 40px")[1];
    assert!(
        (item.width - 211.0).abs() < 0.01,
        "item width {}",
        item.width
    );
    assert!(
        (item.height - 40.0).abs() < 0.01,
        "item height {}",
        item.height
    );
}

/// `justify-self: start` was the audit's own proof that the declared value is
/// understood and the *alignment* was what discarded it: it already gave 50 while
/// the default gave 211. It still gives 50 — the fallback replaced `stretch`, and
/// left every other alignment keyword exactly where it was.
#[test]
fn an_explicit_start_still_reads_the_declared_width() {
    let widths = widths("width: 50px; justify-self: start");
    assert!((widths[1] - 50.0).abs() < 0.01, "item {}", widths[1]);
}
