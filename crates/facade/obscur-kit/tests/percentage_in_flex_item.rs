//! A flex item's own percentage size resolves against the **flex container's
//! content box**, in the cross-size measure as well as in the main-size solve.
//!
//! `specs/w3c/css-flexbox-1.txt:640`, vendored here: "Flex containers form a
//! containing block for their contents exactly like block containers do." The
//! sizing corollary is at `:2389` — "percentage values of flex-basis are resolved
//! against the flex item's containing block (i.e. its flex container)".
//!
//! The defect this pins is the audit's grid finding 2-4 (a) wearing flex's
//! clothes (`docs/external-consumer-audit.md`): a caller that knows the
//! containing block and does not say it. Flex resolves an item's `width: 50%`
//! against the container correctly, then measures that item's cross size with the
//! main axis pinned to the used size it just computed — and handed no basis, so
//! the item read *that* number back as its containing block and laid its own
//! content out at half again. Anything whose block size depends on its inline
//! size then answered for a box that does not exist: the item below measured two
//! wrapped lines (120) where its arranged self has one (60), so the item's height
//! and its own content disagreed on screen.
//!
//! The geometry is chosen to be font-free: `flex-wrap` on two rigid 60px boxes
//! couples block size to inline size without a single glyph, so the numbers here
//! are decided by the flex algorithm alone.
#![forbid(unsafe_code)]

use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::geometry::{Rect, Size};
use obscur_kit::identity::{ComponentId, NodeId, RuntimeId};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::view::view::{CLASS, View, ViewFragment, ViewNode, ViewProperty, ViewValue};

/// Roomy enough that nothing here is clipped by it.
const VIEWPORT: Size = Size {
    width: 480.0,
    height: 240.0,
};

/// A 400-wide row holding one `width: 50%` item — so the item is 200 — which is
/// itself a wrapping row of two 60px boxes. Two 60s fit on one line at 200 and
/// wrap at 100, so the item's height says which number it believed its
/// containing block was. `align-items: flex-start` is what routes the item down
/// the *measured* cross-size path rather than the stretch.
const SHEET: &str = ".app { width: 400px; height: 200px; display: flex; \
                     align-items: flex-start } \
                     .item { width: 50%; display: flex; flex-wrap: wrap } \
                     .kid { width: 60px; height: 60px }";

fn compile(css: &str) -> Stylesheet {
    let src = CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&src).parse();
    StylesheetCompiler::new().compile(&parsed).matchable
}

fn el(class: &str, kids: Vec<ViewNode>) -> ViewNode {
    ViewNode::element(ComponentId::from_raw(82))
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text(class.into())))
        .with_children(ViewFragment::from_nodes(kids))
}

/// `.app > .item > (.kid, .kid)`, laid out once; returns the item's box and its
/// two children's boxes.
fn boxes() -> (Rect, Vec<Rect>) {
    let sheet = compile(SHEET);
    let view = View::from_fragment(ViewFragment::from_nodes(vec![el(
        "app",
        vec![el("item", vec![el("kid", vec![]), el("kid", vec![])])],
    )]));
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), VIEWPORT);
    let _ = rt.render_now(&view, std::slice::from_ref(&sheet));

    let root: NodeId = rt.tree().root().expect("a root after a frame");
    let app = rt.tree().get(root).expect("live").child_nodes()[0];
    let item = rt.tree().get(app).expect("live").child_nodes()[0];
    let kids: Vec<Rect> = rt
        .tree()
        .get(item)
        .expect("live")
        .child_nodes()
        .iter()
        .map(|kid| rt.layout_boxes().get(kid).expect("an arranged box").bounds)
        .collect();
    (
        rt.layout_boxes()
            .get(&item)
            .expect("an arranged box")
            .bounds,
        kids,
    )
}

/// The item is 200 (half of 400) and 60 tall — one line.
///
/// Before the fix this read 200 x **120**: the item's cross measure was handed
/// its own used main size as the only number available, resolved `width: 50%`
/// against it, laid its content out at 100, and reported the two wrapped lines
/// that width forces.
#[test]
fn a_percentage_item_measures_its_content_at_the_width_it_is_laid_out_at() {
    let (item, _) = boxes();
    assert!((item.size.width - 200.0).abs() < 0.01, "item {item:?}");
    assert!((item.size.height - 60.0).abs() < 0.01, "item {item:?}");
}

/// The half that made it visible rather than merely wrong by a number: the
/// arranged content really is on one line, so the 120 the item used to report was
/// height it never filled.
#[test]
fn the_items_arranged_content_agrees_with_the_height_it_reported() {
    let (item, kids) = boxes();
    assert_eq!(kids.len(), 2, "two children");
    assert!(
        (kids[0].origin.y - kids[1].origin.y).abs() < 0.01,
        "both boxes sit on one line: {kids:?}"
    );
    assert!(
        (kids[1].origin.x - kids[0].origin.x - 60.0).abs() < 0.01,
        "the second box follows the first along the line: {kids:?}"
    );
    assert!(
        (item.size.height - kids[0].size.height).abs() < 0.01,
        "the item is exactly as tall as the one line it holds: {item:?}"
    );
}
