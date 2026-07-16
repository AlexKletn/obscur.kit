//! End-to-end: **relative units survive the layout↔paint seam** (spec §4.5,
//! §4.14).
//!
//! CSS specifies that the computed value of a `<length>` is already absolute:
//! `em` / `rem` / `vw` / `vh` are folded once, at the computed-value step, so
//! every consumer downstream of it reads plain pixels. These tests hold the
//! *whole pipeline* to that, because it is the one place the property could be
//! resolved twice and get two answers: layout used to fold relative units itself
//! while paint read the same computed property through `Length::as_px` (absolute
//! `px` only), so the two phases disagreed about the same box — a `.1em` border
//! reserved space and painted nothing.
//!
//! Each test authors real CSS, runs one real frame through [`UiRuntime`], and
//! asserts against the *published* [`DisplayList`] — never a hand-built scene —
//! so a regression anywhere from the CSS compiler through the resolver to the
//! scene coordinator fails here.

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

/// The viewport every test renders under. 400x200 is deliberately non-square so a
/// `vw` assertion cannot pass by accidentally reading `vh`.
const VIEWPORT: Size = Size {
    width: 400.0,
    height: 200.0,
};

fn compile(css: &str) -> Stylesheet {
    let src = CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&src).parse();
    StylesheetCompiler::new().compile(&parsed).matchable
}

/// A single `.box` element.
fn one_box_view() -> View {
    let node = ViewNode::element(ComponentId::from_raw(70))
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("box".into())));
    View::from_fragment(ViewFragment::from_nodes(vec![node]))
}

/// A `.page` **root** wrapping a `.box` that wraps a text node.
///
/// The nesting is load-bearing for the `font-size` tests: `.box` must have a real
/// parent, or `em` (which means the parent's size) and `rem` (which means the
/// root's) would collapse onto the same node and agree by accident.
fn text_view(text: &str) -> View {
    let component = ComponentId::from_raw(70);
    let node = ViewNode::element(component)
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("box".into())))
        .with_children(ViewFragment::from_nodes(vec![ViewNode::text(text)]));
    let page = ViewNode::element(component)
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("page".into())))
        .with_children(ViewFragment::from_nodes(vec![node]));
    View::from_fragment(ViewFragment::from_nodes(vec![page]))
}

/// The font size of the one painted text run in `rt`'s last frame.
fn painted_font_size(commands: &[PaintCommand]) -> f32 {
    commands
        .iter()
        .find_map(|c| match c {
            PaintCommand::DrawText { run, .. } => Some(run.font_size()),
            _ => None,
        })
        .expect("the text node paints a run")
}

fn box_node(rt: &UiRuntime) -> NodeId {
    let root = rt.tree().root().expect("root after a frame");
    rt.tree().get(root).expect("live").children()[0]
}

/// The `.box` under the `.page` root of [`text_view`].
fn nested_box_node(rt: &UiRuntime) -> NodeId {
    let page = box_node(rt);
    rt.tree().get(page).expect("live").children()[0]
}

/// The `.box`'s arranged border box after one frame under `css`.
fn layout_bounds(rt: &UiRuntime) -> Rect {
    rt.layout_boxes()
        .get(&box_node(rt))
        .copied()
        .expect("the box is laid out")
        .bounds
}

// ---- font-size ------------------------------------------------------------

// The audit's headline case: `font-size: 2rem` used to RENDER AT 16px, because
// paint read the computed `font-size` through `Length::as_px` and a `Rem` is not
// a `Px`. The run the pipeline actually published must carry 32.
#[test]
fn font_size_in_rem_paints_at_the_resolved_size() {
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), VIEWPORT);
    // The root font-size is the initial 16px, so 2rem = 32px.
    let sheet = compile(".box{font-size:2rem}");
    let result = rt.render_now(&text_view("Ag"), std::slice::from_ref(&sheet));

    let run_size = painted_font_size(result.snapshot().display_list().commands());
    assert!(
        (run_size - 32.0).abs() < 1e-3,
        "2rem must paint at 32px, got {run_size}"
    );
}

// `rem` follows the ROOT's font-size, not the element's own and not the initial.
// With the root at 10px, `2rem` on a descendant is 20 — an implementation that
// resolved `rem` against a hard-coded 16 (or against the element) fails here.
#[test]
fn rem_follows_the_root_font_size() {
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), VIEWPORT);
    let sheet = compile(":root{font-size:10px}.box{font-size:2rem}");
    let result = rt.render_now(&text_view("Ag"), std::slice::from_ref(&sheet));

    let run_size = painted_font_size(result.snapshot().display_list().commands());
    assert!(
        (run_size - 20.0).abs() < 1e-3,
        "2rem against a 10px root is 20px, got {run_size}"
    );
}

// `em` on `font-size` means the PARENT's font-size — the one property where `em`
// cannot mean "my own size", because that is what it is defining. `.page` is the
// root at 10px, so `.box`'s 2.5em is 25.
#[test]
fn font_size_in_em_resolves_against_the_parent() {
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), VIEWPORT);
    let sheet = compile(".page{font-size:10px}.box{font-size:2.5em}");
    let result = rt.render_now(&text_view("Ag"), std::slice::from_ref(&sheet));

    let run_size = painted_font_size(result.snapshot().display_list().commands());
    assert!(
        (run_size - 25.0).abs() < 1e-3,
        "2.5em against a 10px parent is 25px, got {run_size}"
    );
}

// A `2rem` font-size used to poison every `em` on that node: `em` resolves
// against the node's own font-size, and paint read the un-absolutized 16px
// fallback. Here `.box` is 2rem = 32px, so its own `.5em` border is 16px — a
// number reachable only if the font-size folded FIRST and the border folded
// against the result.
#[test]
fn an_em_on_a_node_resolves_against_that_nodes_resolved_rem_font_size() {
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), VIEWPORT);
    let sheet = compile(
        ".page{font-size:16px}.box{font-size:2rem;width:100px;height:60px;\
         border-style:solid;border-width:.5em;border-color:#ff0000}",
    );
    let result = rt.render_now(&text_view("Ag"), std::slice::from_ref(&sheet));

    assert!((painted_font_size(result.snapshot().display_list().commands()) - 32.0).abs() < 1e-3);
    let bounds = rt
        .layout_boxes()
        .get(&nested_box_node(&rt))
        .copied()
        .expect("the box is laid out")
        .bounds;
    let painted = painted_edge_extent(result.snapshot().display_list().commands(), bounds);
    assert!(
        (painted - 16.0).abs() < 1e-3,
        ".5em of a 2rem (32px) font-size is 16px, got {painted} \
         (8 would mean the font-size never left its 16px default)"
    );
}

// ---- border-width ---------------------------------------------------------

/// The painted width of the box's **top** border edge, in logical pixels, or `0.0`
/// when no edge was painted at all.
///
/// Reads the published fills rather than the computed style, so it measures what a
/// renderer would actually put on screen — the half of the seam the audit found
/// disagreeing with layout.
fn painted_edge_extent(commands: &[PaintCommand], bounds: Rect) -> f32 {
    // Each border side is a mitered quad. The top side's is the one whose bounds
    // start at the box's top edge and span its full width; its height is the
    // painted border width. `0.0` (the fold's identity) means nothing matched —
    // which is exactly the old bug's signature, so it must not be confusable with
    // a real measurement.
    commands
        .iter()
        .filter_map(|c| match c {
            PaintCommand::DrawPath { path, .. } => path.bounds(),
            _ => None,
        })
        .filter(|r| {
            (r.origin.y - bounds.origin.y).abs() < 1e-3 && r.size.width >= bounds.size.width - 1e-3
        })
        .map(|r| r.size.height)
        .fold(0.0f32, f32::max)
}

// The audit's sharpest case: `border-width: .1em` **reserved layout space and
// painted nothing** — the two phases literally disagreed about the same box.
// This asserts they now agree, by measuring BOTH sides of the seam in one frame.
#[test]
fn a_border_width_in_em_paints_exactly_what_layout_reserved() {
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), VIEWPORT);
    // font-size 40px, so .1em = 4px of border on every side.
    let sheet = compile(
        ".box{width:100px;height:60px;font-size:40px;border-style:solid;\
         border-width:.1em;border-color:#ff0000}",
    );
    let result = rt.render_now(&one_box_view(), std::slice::from_ref(&sheet));
    assert!(result.is_balanced(), "the display list is well-formed");

    let bounds = layout_bounds(&rt);
    let painted = painted_edge_extent(result.snapshot().display_list().commands(), bounds);
    assert!(
        (painted - 4.0).abs() < 1e-3,
        ".1em at font-size 40 must paint a 4px edge, got {painted}"
    );

    // And the layout phase reserved the same 4px: with `box-sizing: content-box`
    // the border box is the 100x60 content plus 4px of border on each side.
    assert!(
        (bounds.size.width - 108.0).abs() < 1e-3 && (bounds.size.height - 68.0).abs() < 1e-3,
        "layout reserved a 4px ring (108x68 border box), got {:?}",
        bounds.size
    );
}

// The same border, authored in `px`, must paint identically — the proof that the
// `em` path above lands on the real answer rather than on some other constant.
#[test]
fn an_em_border_and_the_equivalent_px_border_paint_the_same_edge() {
    let render = |css: &str| {
        let mut rt = UiRuntime::new(RuntimeId::from_raw(1), VIEWPORT);
        let sheet = compile(css);
        let result = rt.render_now(&one_box_view(), std::slice::from_ref(&sheet));
        let bounds = layout_bounds(&rt);
        (
            painted_edge_extent(result.snapshot().display_list().commands(), bounds),
            bounds,
        )
    };
    let base =
        ".box{width:100px;height:60px;font-size:40px;border-style:solid;border-color:#ff0000;";
    let (em_edge, em_bounds) = render(&format!("{base}border-width:.1em}}"));
    let (px_edge, px_bounds) = render(&format!("{base}border-width:4px}}"));

    assert!((em_edge - px_edge).abs() < 1e-3, "{em_edge} vs {px_edge}");
    assert!(px_edge > 0.0, "the px control paints an edge at all");
    assert!(
        (em_bounds.size.width - px_bounds.size.width).abs() < 1e-3,
        "{:?} vs {:?}",
        em_bounds.size,
        px_bounds.size
    );
}

// ---- ch -------------------------------------------------------------------

// The audit's measured `ch` case, and the last unit to die at this seam: at HEAD
// `.box{width:100px;font-size:40px;border-style:solid;border-width:1ch}` arranged
// a 148px border box — 24px of `ch` per side — and painted a 0px edge. `ch` is a
// length like any other; the computed value must already be absolute.
//
// The default `MockTextShaper` advances 0.6em per glyph, so `1ch` at font-size 40
// is 24. That number is chosen because it disagrees with BOTH foils: the `0.5em`
// substitute the spec allows only when the `0` glyph cannot be measured would
// paint 20, and the old bug painted 0.
#[test]
fn a_border_width_in_ch_paints_exactly_what_layout_reserved() {
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), VIEWPORT);
    let sheet = compile(
        ".box{width:100px;height:60px;font-size:40px;border-style:solid;\
         border-width:1ch;border-color:#ff0000}",
    );
    let result = rt.render_now(&one_box_view(), std::slice::from_ref(&sheet));
    assert!(result.is_balanced(), "the display list is well-formed");

    let bounds = layout_bounds(&rt);
    // Layout's side of the seam, unchanged: a 24px ring around a 100x60 box.
    assert!(
        (bounds.size.width - 148.0).abs() < 1e-3 && (bounds.size.height - 108.0).abs() < 1e-3,
        "layout reserved a 24px ch ring (148x108 border box), got {:?}",
        bounds.size
    );

    let painted = painted_edge_extent(result.snapshot().display_list().commands(), bounds);
    assert!(
        (painted - 24.0).abs() < 1e-3,
        "1ch at font-size 40 must paint the 24px edge layout reserved, got {painted} \
         (0 is the old seam; 20 would mean `ch` baked in the 0.5em substitute)"
    );
}

// `font-size: 2ch` is the case round five called unreachable, on the grounds that
// the `0` glyph "must be shaped in this node's own face". CSS Values 4 says the
// opposite, and says it explicitly (§6.1.2, the paragraph after the unit table):
//
// > When used in the value of any font-* property on the element they refer to,
// > the font-relative lengths resolve against the computed metrics of the PARENT
// > element.
//
// So there is no circularity to break: `2ch` on `font-size` means two of the
// PARENT's `0` advances. The parent is at 20px and the mock advances 0.6em, so the
// parent's `ch` is 12 and `.box` paints at 24. The old answer — silently falling
// back to the parent's font-size — was 20, so this discriminates.
#[test]
fn font_size_in_ch_resolves_against_the_parents_metrics() {
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), VIEWPORT);
    let sheet = compile(".page{font-size:20px}.box{font-size:2ch}");
    let result = rt.render_now(&text_view("Ag"), std::slice::from_ref(&sheet));

    let run_size = painted_font_size(result.snapshot().display_list().commands());
    assert!(
        (run_size - 24.0).abs() < 1e-3,
        "2ch against a 20px parent whose `0` advances 12px is 24px, got {run_size} \
         (20 would mean `ch` fell back to the parent's font-size)"
    );
}

// ---- box-shadow -----------------------------------------------------------

// `box-shadow` offsets/blur/spread in relative units used to become 0 silently.
// Every one of the four lengths must fold, in the units they are each authored in.
#[test]
fn box_shadow_lengths_in_relative_units_all_reach_paint() {
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), VIEWPORT);
    // font-size 20px -> 1em = 20; root font-size is the initial 16 -> .5rem = 8;
    // viewport width 400 -> 1vw = 4.
    let sheet = compile(
        ".box{width:80px;height:40px;font-size:20px;background-color:#00ff00;\
         box-shadow:1em 0.5rem 1vw 2px #000000}",
    );
    let result = rt.render_now(&one_box_view(), std::slice::from_ref(&sheet));

    let commands = result.snapshot().display_list().commands();
    let shadow = commands
        .iter()
        .find(|c| matches!(c, PaintCommand::DrawShadow { .. }))
        .expect("an authored box-shadow emits a DrawShadow");
    let PaintCommand::DrawShadow {
        blur,
        spread,
        offset,
        ..
    } = shadow
    else {
        unreachable!("matched DrawShadow above");
    };
    assert_eq!(*offset, (20.0, 8.0), "1em -> 20, 0.5rem -> 8");
    assert_eq!(*blur, 4.0, "1vw of a 400px viewport -> 4");
    assert_eq!(*spread, 2.0, "the absolute term is untouched");
}

// ---- viewport units -------------------------------------------------------

// `vw` and `vh` are distinct bases. Under a 400x200 viewport a `10vw` border is
// 40px and a `10vh` one is 20px; swapping the two axes fails this.
#[test]
fn viewport_units_paint_against_the_axis_they_name() {
    let render = |css: &str| {
        let mut rt = UiRuntime::new(RuntimeId::from_raw(1), VIEWPORT);
        let sheet = compile(css);
        let result = rt.render_now(&one_box_view(), std::slice::from_ref(&sheet));
        let bounds = layout_bounds(&rt);
        painted_edge_extent(result.snapshot().display_list().commands(), bounds)
    };
    let base = ".box{width:100px;height:60px;border-style:solid;border-color:#ff0000;";
    assert!((render(&format!("{base}border-width:10vw}}")) - 40.0).abs() < 1e-3);
    assert!((render(&format!("{base}border-width:10vh}}")) - 20.0).abs() < 1e-3);
}

// ---- what is deliberately NOT absolutized ---------------------------------

// A percentage is NOT a length that can be absolutized: it resolves against a
// containing block the computed-value step has not produced. This pins the
// boundary of the pass — `width: 50%` must still reach layout as a percentage and
// be resolved there, not folded to a wrong number (or to zero) at computed time.
#[test]
fn a_percentage_width_still_resolves_against_the_containing_block() {
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), VIEWPORT);
    let sheet = compile(".box{width:50%;height:20px}");
    rt.render_now(&one_box_view(), std::slice::from_ref(&sheet));
    let width = layout_bounds(&rt).size.width;
    assert!(
        (width - 200.0).abs() < 1e-3,
        "50% of the 400px viewport is 200, got {width}"
    );
}

// Absolutizing `vw`/`vh` at computed-value time creates one regression risk: a
// baked viewport unit goes stale unless a resize re-resolves the computed style.
// This pins that a resized frame paints the new value.
//
// What it does NOT cover: `render_now` renders with `FrameReason::UiCommit`,
// which always restyles, so this exercises the resolve path rather than the
// invalidation decision. The incremental path (`set_viewport` +
// `request_frame(WindowChange)`) is what a real host drives, and it restyles
// because `WindowChange` routes to `InvalidationKind::Environment`, whose
// `needs_restyle()` is `true` — asserted in that type's own tests, not here.
#[test]
fn a_viewport_resize_re_resolves_viewport_units() {
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), VIEWPORT);
    let sheet = compile(
        ".box{width:100px;height:60px;border-style:solid;border-width:10vw;border-color:#ff0000}",
    );
    let result = rt.render_now(&one_box_view(), std::slice::from_ref(&sheet));
    let painted = painted_edge_extent(
        result.snapshot().display_list().commands(),
        layout_bounds(&rt),
    );
    assert!(
        (painted - 40.0).abs() < 1e-3,
        "10vw of 400px is 40, got {painted}"
    );

    // Halve the viewport width: the same declaration must now paint 20.
    rt.set_viewport(Size::new(200.0, 200.0));
    let result = rt.render_now(&one_box_view(), std::slice::from_ref(&sheet));
    let painted = painted_edge_extent(
        result.snapshot().display_list().commands(),
        layout_bounds(&rt),
    );
    assert!(
        (painted - 20.0).abs() < 1e-3,
        "10vw of a resized 200px viewport is 20, got {painted} \
         (40 would mean the computed style went stale across the resize)"
    );
}
