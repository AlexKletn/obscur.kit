#![doc = include_str!("intrinsic_sizing.doc/_module.md")]
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

#[doc = include_str!("intrinsic_sizing.doc/FONT.md")]
const FONT: f32 = 16.0;
const GLYPH: f32 = FONT * MockTextShaper::ADVANCE_RATIO;

#[doc = include_str!("intrinsic_sizing.doc/TEXT.md")]
const TEXT: &str = "hello wonderful world";
#[doc = include_str!("intrinsic_sizing.doc/ONE_LINE.md")]
const ONE_LINE: f32 = 21.0 * GLYPH;
#[doc = include_str!("intrinsic_sizing.doc/LONGEST_WORD.md")]
const LONGEST_WORD: f32 = 9.0 * GLYPH;

fn compile(css: &str) -> Stylesheet {
    let src = CssSource::anonymous(css);
    StylesheetParser::new(&src)
        .parse()
        .pipe(|parsed| StylesheetCompiler::new().compile(&parsed).matchable)
}

#[doc = include_str!("intrinsic_sizing.doc/Pipe.md")]
trait Pipe: Sized {
    fn pipe<R>(self, f: impl FnOnce(Self) -> R) -> R {
        f(self)
    }
}
impl<T> Pipe for T {}

#[doc = include_str!("intrinsic_sizing.doc/item_size.md")]
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
    let page_id = runtime.tree().get(root).expect("live").child_nodes()[0];
    let item_id: NodeId = runtime.tree().get(page_id).expect("live").child_nodes()[0];
    let layout = runtime
        .layout_boxes()
        .get(&item_id)
        .expect("the item is laid out");
    (layout.size().width, layout.size().height)
}

#[doc = include_str!("intrinsic_sizing.doc/page_with.md")]
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
fn min_width_min_content_floors_a_narrower_authored_width() {
    // The keywords are values of the BOUNDS too, not just of `width` (CSS Sizing 3
    // §4.1). A 40px box whose content cannot break below `wonderful` (86.4) is
    // floored at 86.4 and overflows — the bound was silently ignored before.
    let (width, height) = item_size(&format!(
        ".page {{ width: 400px; height: 400px; font-size: {FONT}px }} \
         .item {{ width: 40px; min-width: min-content }}"
    ));
    assert!(
        (width - LONGEST_WORD).abs() < 1e-3,
        "`min-width: min-content` floors at {LONGEST_WORD}, got {width}"
    );
    assert!(width > 40.0, "it really did overflow the authored width");
    // The independent witness: the children were laid out at the FLOORED width,
    // so the text wraps into three lines rather than the many a 40px box needs.
    assert!(
        (height - FONT * 3.0).abs() < 1e-3,
        "three words, three lines at the floored width, got {height}"
    );
}

#[test]
fn max_width_max_content_caps_a_stretching_box() {
    // `width: auto` fills its 400px container; `max-width: max-content` caps it at
    // the width the content actually wants.
    let (width, _) = item_size(&format!(
        ".page {{ width: 400px; height: 400px; font-size: {FONT}px }} \
         .item {{ max-width: max-content }}"
    ));
    assert!(
        (width - ONE_LINE).abs() < 1e-3,
        "`max-width: max-content` caps at {ONE_LINE}, got {width}"
    );
    assert!(width < 400.0, "the cap did nothing and the box stretched");
}

#[test]
fn min_height_min_content_floors_a_shorter_authored_height() {
    // The block axis: `min-content` and `max-content` coincide with the content
    // height there, so this floors the 5px box at the one line of text it holds.
    let (_, height) = item_size(&format!(
        ".page {{ width: 400px; height: 400px; font-size: {FONT}px }} \
         .item {{ height: 5px; min-height: min-content }}"
    ));
    assert!(
        (height - FONT).abs() < 1e-3,
        "`min-height: min-content` floors at the one line of text ({FONT}), got {height}"
    );
}

#[test]
fn an_auto_bound_still_imposes_no_limit() {
    // The regression guard for the bounds path: `min-width`/`max-width` unset must
    // leave an `auto` box filling its container, exactly as before.
    let (width, _) = item_size(&page_with("auto"));
    assert!((width - 400.0).abs() < 1e-3, "got {width}");
}

#[doc = include_str!("intrinsic_sizing.doc/grid_items.md")]
fn grid_items(css: &str) -> [(f32, f32, f32); 2] {
    let sheet = compile(css);
    let component = ComponentId::from_raw(0x00C0_0001);
    let item = ViewNode::element(component)
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("item".into())))
        .with_children(ViewFragment::from_nodes(vec![ViewNode::text(TEXT)]));
    let tail = ViewNode::element(component)
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("tail".into())));
    let page = ViewNode::element(component)
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("page".into())))
        .with_children(ViewFragment::from_nodes(vec![item, tail]));
    let view = View::from_fragment(ViewFragment::from_nodes(vec![page]));

    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(600.0, 400.0));
    runtime.render_now(&view, std::slice::from_ref(&sheet));
    let root = runtime.tree().root().expect("a root");
    let page_id = runtime.tree().get(root).expect("live").child_nodes()[0];
    let kids = runtime
        .tree()
        .get(page_id)
        .expect("live")
        .child_nodes()
        .to_vec();
    std::array::from_fn(|i| {
        let b = runtime.layout_boxes().get(&kids[i]).expect("laid out");
        (b.origin().x, b.size().width, b.size().height)
    })
}

#[doc = include_str!("intrinsic_sizing.doc/grid_page.md")]
fn grid_page(track: &str) -> String {
    format!(
        ".page {{ display: grid; width: 400px; font-size: {FONT}px; \
           grid-template-columns: {track} 1fr }} \
         .tail {{ height: 10px }}"
    )
}

#[test]
fn a_min_content_column_hugs_its_item_and_the_fr_takes_the_rest() {
    // The whole §4-D chain in one assertion: `min-content` survives CSS lowering
    // (it used to abort the declaration and take the `1fr` with it), reaches the
    // track-sizing algorithm as a real sizing function (not `auto`), and is
    // answered by probing the item — which is a BLOCK, so the probe has to carry
    // through its `width: auto` to the text inside it.
    let [(item_x, item_w, item_h), (tail_x, tail_w, _)] = grid_items(&grid_page("min-content"));
    assert!(
        (item_w - LONGEST_WORD).abs() < 1e-3,
        "the min-content column hugs `wonderful` = {LONGEST_WORD}, got {item_w}"
    );
    assert!((item_x - 0.0).abs() < 1e-3, "the item leads, got {item_x}");
    // The `1fr` really took the REST — which is only true if the first track was
    // sized at all. If the template had been dropped, both items would sit in one
    // implicit column at x = 0.
    assert!(
        (tail_x - LONGEST_WORD).abs() < 1e-3,
        "the fr column starts after the min-content one, got {tail_x}"
    );
    assert!(
        (tail_w - (400.0 - LONGEST_WORD)).abs() < 1e-3,
        "the fr column takes the remaining {}, got {tail_w}",
        400.0 - LONGEST_WORD
    );
    // The independent witness that it is min-content and not something that
    // merely looks small: the text wrapped into three lines under it.
    assert!(
        (item_h - FONT * 3.0).abs() < 1e-3,
        "three words, three lines, got {item_h}"
    );
}

#[test]
fn a_max_content_column_puts_its_item_on_one_line() {
    // The same chain for the other keyword, and the two must disagree.
    let [(_, item_w, item_h), (tail_x, tail_w, _)] = grid_items(&grid_page("max-content"));
    assert!(
        (item_w - ONE_LINE).abs() < 1e-3,
        "the max-content column fits all 21 characters = {ONE_LINE}, got {item_w}"
    );
    assert!(
        (item_h - FONT).abs() < 1e-3,
        "one line tall, got {item_h} (it wrapped, so the offer was not unbounded)"
    );
    assert!((tail_x - ONE_LINE).abs() < 1e-3, "tail.x, got {tail_x}");
    assert!(
        (tail_w - (400.0 - ONE_LINE)).abs() < 1e-3,
        "the fr column takes the rest, got {tail_w}"
    );
}

#[test]
fn a_content_keyword_column_is_not_an_auto_column() {
    // `auto` is the fallback the keywords used to degrade to (when they were not
    // dropping the whole declaration). If any of these agreed, this file would be
    // testing nothing: an `auto` column beside a `1fr` stretches to hold the
    // item's whole line, while the two keywords each answer with a different
    // property of the content.
    let [(_, min_w, _), _] = grid_items(&grid_page("min-content"));
    let [(_, max_w, _), _] = grid_items(&grid_page("max-content"));
    let [(_, auto_w, _), _] = grid_items(&grid_page("auto"));
    assert!(
        min_w < max_w,
        "min-content {min_w} must be narrower than max-content {max_w}"
    );
    assert!(
        (min_w - auto_w).abs() > 1e-3,
        "min-content {min_w} is indistinguishable from auto {auto_w}"
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
    let page_id = runtime.tree().get(root).expect("live").child_nodes()[0];
    let item_id: NodeId = runtime.tree().get(page_id).expect("live").child_nodes()[0];
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
