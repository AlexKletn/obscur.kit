//! End-to-end: the **`ch` unit** resolves from the `0` glyph actually shaped by
//! the runtime's font, not from a stand-in (spec §4.5).
//!
//! `ch` is the advance of `0` (ZERO, U+0030) in the element's own font — the unit
//! that sizes a box by character count (`width: 60ch`). CSS Values 4 §6.1.2 allows
//! `0.5em` as a substitute *only* "in the cases where it is impossible or
//! impractical to determine the measure of the ZERO glyph". That escape hatch is
//! the thing these tests exist to rule out: resolving every `ch` to `0.5em` would
//! look like support while mis-sizing the one case anyone writes `ch` for.
//!
//! The default [`MockTextShaper`] is monospace at `0.6 × font-size` per glyph.
//! That ratio is what makes these assertions meaningful: a real measurement lands
//! on `0.6em` per `ch`, the substitute would land on `0.5em`, and every test below
//! is written so those two answers disagree. A `ch` that quietly fell back would
//! fail here rather than pass with plausible numbers.

#![forbid(unsafe_code)]

use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::geometry::Size;
use obscur_kit::identity::{ComponentId, NodeId, RuntimeId};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::text::shaping::MockTextShaper;
use obscur_kit::view::view::{CLASS, View, ViewFragment, ViewNode, ViewProperty, ViewValue};

/// The advance the default shaper gives every glyph, as a fraction of the font
/// size. `ch` must equal this — not the `0.5` substitute.
const RATIO: f32 = MockTextShaper::ADVANCE_RATIO;

/// The substitute [`LengthContext::ch_base`] falls back to when the `0` glyph
/// cannot be measured. Named here only so the tests can assert they did **not**
/// get it.
const SUBSTITUTE: f32 = 0.5;

fn compile(css: &str) -> Stylesheet {
    let src = CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&src).parse();
    StylesheetCompiler::new().compile(&parsed).matchable
}

/// A `.page` box containing one `.item`.
fn view() -> View {
    let container = ComponentId::from_raw(0x00C0_0001);
    let item = ViewNode::element(container)
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("item".into())));
    let page = ViewNode::element(container)
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("page".into())))
        .with_children(ViewFragment::from_nodes(vec![item]));
    View::from_fragment(ViewFragment::from_nodes(vec![page]))
}

/// The `.item`'s arranged size after rendering `css`.
fn item_size(css: &str) -> (f32, f32) {
    let sheet = compile(css);
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(600.0, 600.0));
    runtime.render_now(&view(), std::slice::from_ref(&sheet));

    let root = runtime.tree().root().expect("a root");
    let page = runtime.tree().get(root).expect("live").children()[0];
    let item: NodeId = runtime.tree().get(page).expect("live").children()[0];
    let layout = runtime.layout_boxes().get(&item).expect("the item is laid out");
    (layout.size().width, layout.size().height)
}

const PAGE: &str = ".page { width: 600px; height: 600px }";

#[test]
fn a_ch_width_is_the_measured_zero_advance_times_the_count() {
    // 20ch at font-size 16 in a face advancing 0.6em per glyph = 20*0.6*16 = 192.
    let (width, _) = item_size(&format!(
        "{PAGE}\n.item {{ font-size: 16px; width: 20ch; height: 10px }}"
    ));
    let expected = 20.0 * RATIO * 16.0;
    assert!(
        (width - expected).abs() < 1e-3,
        "20ch must be twenty `0`-advances wide, want {expected}, got {width}"
    );
    // The assertion that gives the one above its teeth: the specified substitute
    // would have produced 160, and it did not.
    let substituted = 20.0 * SUBSTITUTE * 16.0;
    assert!(
        (width - substituted).abs() > 1.0,
        "`ch` fell back to the 0.5em substitute ({substituted}) instead of measuring"
    );
}

#[test]
fn ch_follows_the_font_size_it_was_measured_at() {
    // Doubling `font-size` doubles the `0` advance, so the same `10ch` doubles.
    let (small, _) = item_size(&format!(
        "{PAGE}\n.item {{ font-size: 10px; width: 10ch; height: 10px }}"
    ));
    let (large, _) = item_size(&format!(
        "{PAGE}\n.item {{ font-size: 20px; width: 10ch; height: 10px }}"
    ));
    assert!((small - 10.0 * RATIO * 10.0).abs() < 1e-3, "got {small}");
    assert!((large - 10.0 * RATIO * 20.0).abs() < 1e-3, "got {large}");
    assert!(
        (large - small * 2.0).abs() < 1e-3,
        "the measurement tracks the size it was taken at: {small} then {large}"
    );
}

#[test]
fn a_ch_box_actually_fits_that_many_characters() {
    // What the unit is FOR, checked rather than asserted in prose: a `20ch` box in
    // a monospace face is exactly as wide as twenty characters shaped in it. This
    // is the claim the 0.5em substitute would break, and it is why measuring is
    // not optional.
    const TEXT: &str = "00000000000000000000"; // twenty of them
    let (box_width, _) = item_size(&format!(
        "{PAGE}\n.item {{ font-size: 16px; width: 20ch; height: 10px }}"
    ));

    // Measure the same twenty characters through the same runtime by letting a
    // text node size itself.
    let sheet = compile(&format!("{PAGE}\n.item {{ font-size: 16px }}"));
    let container = ComponentId::from_raw(0x00C0_0001);
    let item = ViewNode::element(container)
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("item".into())))
        .with_children(ViewFragment::from_nodes(vec![ViewNode::text(TEXT)]));
    let page = ViewNode::element(container)
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("page".into())))
        .with_children(ViewFragment::from_nodes(vec![item]));
    let view = View::from_fragment(ViewFragment::from_nodes(vec![page]));

    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(600.0, 600.0));
    runtime.render_now(&view, std::slice::from_ref(&sheet));
    let root = runtime.tree().root().expect("a root");
    let page_id = runtime.tree().get(root).expect("live").children()[0];
    let item_id = runtime.tree().get(page_id).expect("live").children()[0];
    let text_id: NodeId = runtime.tree().get(item_id).expect("live").children()[0];
    let text_width = runtime
        .layout_boxes()
        .get(&text_id)
        .expect("the text is laid out")
        .size()
        .width;

    assert!(
        (box_width - text_width).abs() < 1e-3,
        "a 20ch box ({box_width}) must fit twenty shaped characters ({text_width})"
    );
}

#[test]
fn ch_and_em_are_different_bases() {
    // Both are font-relative, and confusing them is the easy bug: at 0.6em/glyph
    // they must disagree. If `ch` were implemented as an alias of `em` — or as a
    // fixed fraction of it — this test is what fails.
    let (ch_width, _) = item_size(&format!(
        "{PAGE}\n.item {{ font-size: 20px; width: 10ch; height: 10px }}"
    ));
    let (em_width, _) = item_size(&format!(
        "{PAGE}\n.item {{ font-size: 20px; width: 10em; height: 10px }}"
    ));
    assert!((em_width - 200.0).abs() < 1e-3, "10em at 20px = 200, got {em_width}");
    assert!((ch_width - 120.0).abs() < 1e-3, "10ch at 20px = 120, got {ch_width}");
    assert!(ch_width < em_width, "a `0` is narrower than an em in this face");
}

#[test]
fn ch_composes_with_calc_end_to_end() {
    // `ch` is an ordinary term in the expression grammar, not a special case
    // handled only at the top level of a declaration.
    let (width, _) = item_size(&format!(
        "{PAGE}\n.item {{ font-size: 16px; width: calc(10ch + 8px); height: 10px }}"
    ));
    let expected = 10.0 * RATIO * 16.0 + 8.0;
    assert!((width - expected).abs() < 1e-3, "want {expected}, got {width}");
}

#[test]
fn ch_sizes_a_grid_track() {
    // The other length context that exists: a fixed grid track. A `ch` there must
    // resolve through the same measurement, not silently degrade to auto.
    let sheet = compile(
        ".page { display: grid; width: 600px; height: 600px; font-size: 16px; \
         grid-template-columns: 20ch 1fr }\n.item { height: 10px }",
    );
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(600.0, 600.0));
    runtime.render_now(&view(), std::slice::from_ref(&sheet));

    let root = runtime.tree().root().expect("a root");
    let page = runtime.tree().get(root).expect("live").children()[0];
    let item: NodeId = runtime.tree().get(page).expect("live").children()[0];
    let width = runtime
        .layout_boxes()
        .get(&item)
        .expect("the item is laid out")
        .size()
        .width;

    let expected = 20.0 * RATIO * 16.0;
    assert!(
        (width - expected).abs() < 1e-3,
        "the first track is 20ch wide, want {expected}, got {width}"
    );
}

#[test]
fn a_node_that_writes_no_ch_is_unaffected() {
    // The measurement is taken only for a node that needs it; a node that writes
    // none must lay out exactly as it did before `ch` existed.
    let (width, height) = item_size(&format!(
        "{PAGE}\n.item {{ font-size: 16px; width: 123px; height: 45px }}"
    ));
    assert!((width - 123.0).abs() < 1e-3, "got {width}");
    assert!((height - 45.0).abs() < 1e-3, "got {height}");
}
