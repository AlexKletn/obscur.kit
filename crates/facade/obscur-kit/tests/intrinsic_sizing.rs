//! End-to-end: the **intrinsic sizing keywords** (`min-content` / `max-content` /
//! `fit-content`) are answered by measuring content through the live pipeline
//! (spec §4.2, CSS Sizing 3 §4.1).
//!
//! These three cannot be resolved by arithmetic — the answer is a property of the
//! content, so the box is sized by measuring its children a second time under a
//! different offer. That makes them easy to fake and easy to get subtly wrong, so
//! every test here asserts a real arranged box against a number derived from the
//! text, not against itself:
//!
//! * `max-content` — everything on one line;
//! * `min-content` — the widest word, with the rest wrapped under it;
//! * `fit-content` — the content when it fits, the container when it does not,
//!   and never below the widest word.
//!
//! The default shaper is monospace at `0.6 × font-size` per glyph, so a run of
//! `n` characters at 16px is exactly `n * 9.6` wide. Every expectation below is
//! written from the actual string, which is what makes a wrong answer visible
//! rather than plausible.

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

/// Font size every test authors, and the per-glyph advance that follows from it.
const FONT: f32 = 16.0;
const GLYPH: f32 = FONT * MockTextShaper::ADVANCE_RATIO;

/// The text under test: three words, the longest of which is `wonderful` (9).
const TEXT: &str = "hello wonderful world";
/// 21 characters on one line — the `max-content` width.
const ONE_LINE: f32 = 21.0 * GLYPH;
/// `wonderful`, the widest run that cannot be broken — the `min-content` width.
const LONGEST_WORD: f32 = 9.0 * GLYPH;

fn compile(css: &str) -> Stylesheet {
    let src = CssSource::anonymous(css);
    StylesheetParser::new(&src)
        .parse()
        .pipe(|parsed| StylesheetCompiler::new().compile(&parsed).matchable)
}

/// Tiny helper so `compile` reads top-down.
trait Pipe: Sized {
    fn pipe<R>(self, f: impl FnOnce(Self) -> R) -> R {
        f(self)
    }
}
impl<T> Pipe for T {}

/// The `.item`'s arranged size under `css`, with `TEXT` inside it.
fn item_size(css: &str) -> (f32, f32) {
    let sheet = compile(css);
    let component = ComponentId::from_raw(0x00C0_0001);
    let item = ViewNode::element(component)
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("item".into())))
        .with_children(ViewFragment::from_nodes(vec![ViewNode::text(TEXT)]));
    let page = ViewNode::element(component)
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("page".into())))
        .with_children(ViewFragment::from_nodes(vec![item]));
    let view = View::from_fragment(ViewFragment::from_nodes(vec![page]));

    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(400.0, 400.0));
    runtime.render_now(&view, std::slice::from_ref(&sheet));
    let root = runtime.tree().root().expect("a root");
    let page_id = runtime.tree().get(root).expect("live").children()[0];
    let item_id: NodeId = runtime.tree().get(page_id).expect("live").children()[0];
    let layout = runtime
        .layout_boxes()
        .get(&item_id)
        .expect("the item is laid out");
    (layout.size().width, layout.size().height)
}

/// A 400x400 page. `width` is what the `.item` authors.
fn page_with(width: &str) -> String {
    format!(
        ".page {{ width: 400px; height: 400px; font-size: {FONT}px }} .item {{ width: {width} }}"
    )
}

#[test]
fn max_content_puts_everything_on_one_line() {
    let (width, height) = item_size(&page_with("max-content"));
    assert!(
        (width - ONE_LINE).abs() < 1e-3,
        "21 characters on one line = {ONE_LINE}, got {width}"
    );
    assert!(
        (height - FONT).abs() < 1e-3,
        "one line tall, got {height} (it wrapped, so it was not offered unbounded space)"
    );
    // The assertion that gives it teeth: it did NOT just take the container.
    assert!(
        width < 400.0,
        "`max-content` must shrink to the content, not stretch"
    );
}

#[test]
fn min_content_is_the_widest_word_and_the_rest_wraps_under_it() {
    let (width, height) = item_size(&page_with("min-content"));
    assert!(
        (width - LONGEST_WORD).abs() < 1e-3,
        "`wonderful` is the widest unbreakable run = {LONGEST_WORD}, got {width}"
    );
    // Height is the independent witness that the probe really offered no space:
    // three words that each need their own line.
    assert!(
        (height - FONT * 3.0).abs() < 1e-3,
        "three words, three lines, got {height}"
    );
    // And it is not zero — the trap this went through once. A text leaf offered
    // no width must report the width it NEEDS, not the width it was given.
    assert!(width > 0.0, "`min-content` collapsed to nothing");
}

#[test]
fn min_content_and_max_content_disagree_on_wrappable_text() {
    // If these ever agree for a multi-word string, the two probes are not probing.
    let (min, _) = item_size(&page_with("min-content"));
    let (max, _) = item_size(&page_with("max-content"));
    assert!(
        min < max,
        "min-content {min} must be narrower than max-content {max}"
    );
}

#[test]
fn auto_still_stretches_to_the_container() {
    // The regression guard: `auto` is the overwhelmingly common case and must be
    // untouched by any of this.
    let (width, height) = item_size(&page_with("auto"));
    assert!(
        (width - 400.0).abs() < 1e-3,
        "`auto` fills its container, got {width}"
    );
    assert!(
        (height - FONT).abs() < 1e-3,
        "and the text fits on one line, got {height}"
    );
}

#[test]
fn fit_content_shrinks_to_the_content_when_it_fits() {
    // max-content (201.6) fits the 400 page → take the content's width.
    let (width, _) = item_size(&page_with("fit-content"));
    assert!(
        (width - ONE_LINE).abs() < 1e-3,
        "`fit-content` takes the content when it fits, want {ONE_LINE}, got {width}"
    );
}

#[test]
fn fit_content_falls_back_to_the_container_when_the_content_is_wider() {
    // A 120px page: narrower than max-content (201.6) but wider than min-content
    // (86.4), so the box takes the space available and the text wraps into it.
    let (width, _) = item_size(&format!(
        ".page {{ width: 120px; height: 400px; font-size: {FONT}px }} .item {{ width: fit-content }}"
    ));
    assert!(
        (width - 120.0).abs() < 1e-3,
        "`fit-content` takes the space it is offered when content overflows, got {width}"
    );
}

#[test]
fn fit_content_never_squeezes_below_the_widest_word() {
    // A 40px page is narrower than `wonderful` (86.4). CSS 2.1 §10.3.5: the box
    // does not go below min-content — it overflows instead.
    let (width, _) = item_size(&format!(
        ".page {{ width: 40px; height: 400px; font-size: {FONT}px }} .item {{ width: fit-content }}"
    ));
    assert!(
        (width - LONGEST_WORD).abs() < 1e-3,
        "`fit-content` floors at min-content ({LONGEST_WORD}) and overflows, got {width}"
    );
    assert!(width > 40.0, "it really did overflow its container");
}

#[test]
fn an_intrinsic_width_is_still_clamped_by_max_width() {
    // The keywords resolve the *width*; the authored bounds still apply on top,
    // exactly as they do for an explicit length.
    let (width, _) = item_size(&format!(
        ".page {{ width: 400px; height: 400px; font-size: {FONT}px }} \
         .item {{ width: max-content; max-width: 100px }}"
    ));
    assert!(
        (width - 100.0).abs() < 1e-3,
        "max-content (201.6) clamped by max-width, got {width}"
    );
}

#[test]
fn an_intrinsic_width_is_still_floored_by_min_width() {
    let (width, _) = item_size(&format!(
        ".page {{ width: 400px; height: 400px; font-size: {FONT}px }} \
         .item {{ width: min-content; min-width: 300px }}"
    ));
    assert!(
        (width - 300.0).abs() < 1e-3,
        "min-content (86.4) floored by min-width, got {width}"
    );
}

#[test]
fn a_percentage_child_does_not_poison_a_max_content_box() {
    // The infinity trap. A `max-content` probe offers unbounded inline space; a
    // `50%` child resolving against that would come out infinitely wide and take
    // the whole box with it. CSS Sizing 3 §5.1 says an indefinite basis makes a
    // percentage behave as `auto`, and this is that rule doing its job.
    let sheet = compile(
        ".page { width: 400px; height: 400px; font-size: 16px } \
         .item { width: max-content } .inner { width: 50%; height: 10px }",
    );
    let component = ComponentId::from_raw(0x00C0_0001);
    let inner = ViewNode::element(component)
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("inner".into())));
    let item = ViewNode::element(component)
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("item".into())))
        .with_children(ViewFragment::from_nodes(vec![inner]));
    let page = ViewNode::element(component)
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("page".into())))
        .with_children(ViewFragment::from_nodes(vec![item]));
    let view = View::from_fragment(ViewFragment::from_nodes(vec![page]));

    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(400.0, 400.0));
    runtime.render_now(&view, std::slice::from_ref(&sheet));
    let root = runtime.tree().root().expect("a root");
    let page_id = runtime.tree().get(root).expect("live").children()[0];
    let item_id: NodeId = runtime.tree().get(page_id).expect("live").children()[0];
    let width = runtime
        .layout_boxes()
        .get(&item_id)
        .expect("laid out")
        .size()
        .width;

    assert!(
        width.is_finite(),
        "a `%` child resolved against unbounded space and made the box infinite"
    );
    assert!(width <= 400.0, "the box escaped its container: {width}");
}
