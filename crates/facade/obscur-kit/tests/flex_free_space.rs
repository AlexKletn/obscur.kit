//! Free space on a flex line goes to the items that asked to grow (CSS Flexbox
//! §9.7 *Resolving Flexible Lengths*), and an item that did not ask stays at its
//! content size (§9.2.3 — an `auto` basis with no definite main size resolves to
//! the item's max-content size).
//!
//! The defect these pin: a plain item with no flex properties absorbed the whole
//! line and starved the sibling that declared `flex: 1`. On an 800px row a text
//! item took 700px while a `flex: 1` sibling got **zero** — inverted, silently,
//! with no error anywhere.
//!
//! It was invisible in the single-item case: `flex: 1` beside a fixed-width item
//! is correct, which is why the control below matters as much as the case.
#![forbid(unsafe_code)]

use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::geometry::Size;
use obscur_kit::identity::{ComponentId, NodeId, RuntimeId};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::view::view::{CLASS, View, ViewFragment, ViewNode, ViewProperty, ViewValue};

/// A wide, short viewport so the row has plenty of free space to misplace.
const VIEWPORT: Size = Size {
    width: 800.0,
    height: 100.0,
};

const CSS: &str = ".row { width: 100%; height: 100%; display: flex; flex-direction: row } \
                   .grow { flex: 1 } \
                   .fixed { width: 100px } \
                   .auto-basis { flex: 0 0 auto } \
                   .nested { flex: 0 0 auto; display: flex; justify-content: center }";

fn compile(css: &str) -> Stylesheet {
    let src = CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&src).parse();
    StylesheetCompiler::new().compile(&parsed).matchable
}

fn el(raw: u64, class: &str, kids: Vec<ViewNode>) -> ViewNode {
    ViewNode::element(ComponentId::from_raw(raw))
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text(class.into())))
        .with_children(ViewFragment::from_nodes(kids))
}

/// Lays out `items` as a flex row and returns each one's used width.
fn widths(items: Vec<ViewNode>) -> Vec<f32> {
    let sheet = compile(CSS);
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), VIEWPORT);
    let view = View::single(el(1, "row", items));
    let _ = rt.render_now(&view, std::slice::from_ref(&sheet));

    let root = rt.tree().root().expect("a root after a frame");
    let row = rt.tree().get(root).expect("live").child_nodes()[0];
    let children: Vec<NodeId> = rt.tree().get(row).expect("live").child_nodes().to_vec();
    children
        .iter()
        .map(|node| {
            rt.layout_boxes()
                .get(node)
                .expect("an arranged item")
                .bounds
                .size
                .width
        })
        .collect()
}

/// The control, and the reason the defect hid for so long: with nothing else
/// competing, `flex: 1` correctly takes everything the fixed item leaves.
#[test]
fn a_growing_item_takes_the_space_a_fixed_sibling_leaves() {
    let widths = widths(vec![
        el(2, "grow", vec![]),
        el(3, "fixed", vec![ViewNode::text("Fixed")]),
    ]);
    assert_eq!(widths.len(), 2);
    assert!(
        (widths[0] - 700.0).abs() < 1.0,
        "the growing item takes the remaining 700px, got {}",
        widths[0]
    );
    assert!(
        (widths[1] - 100.0).abs() < 1.0,
        "the fixed item keeps its 100px, got {}",
        widths[1]
    );
}

/// The case. A plain item — no `flex` at all, so `flex-grow: 0` — must stay at
/// its content size and leave the free space to the sibling that asked for it.
#[test]
fn a_plain_item_does_not_absorb_the_space_a_sibling_asked_to_grow_into() {
    let widths = widths(vec![
        el(2, "plain", vec![ViewNode::text("Plain")]),
        el(3, "grow", vec![]),
        el(4, "fixed", vec![ViewNode::text("Fixed")]),
    ]);
    assert_eq!(widths.len(), 3);

    // The exact content width depends on the installed shaper, so this asserts
    // the property rather than a number: the item is near its text, not near the
    // line. Before the fix it was 700.
    assert!(
        widths[0] < 200.0,
        "a plain item stays at its content size, got {}",
        widths[0]
    );
    assert!(
        widths[1] > 400.0,
        "the growing item gets the free space, got {}",
        widths[1]
    );
    assert!(
        (widths[2] - 100.0).abs() < 1.0,
        "the fixed item is untouched, got {}",
        widths[2]
    );
    let total: f32 = widths.iter().sum();
    assert!(
        (total - 800.0).abs() < 1.0,
        "the three items fill the line exactly once, got {total}"
    );
}

/// `flex: 0 0 auto` is the explicit spelling of the same thing, and must behave
/// identically — it is what an author writes when they mean "size me to my
/// content and leave me alone".
#[test]
fn an_explicit_auto_basis_sizes_to_content_too() {
    let widths = widths(vec![
        el(2, "auto-basis", vec![ViewNode::text("Plain")]),
        el(3, "grow", vec![]),
    ]);
    assert_eq!(widths.len(), 2);
    assert!(
        widths[0] < 200.0,
        "`flex: 0 0 auto` sizes to content, got {}",
        widths[0]
    );
    assert!(
        widths[1] > 600.0,
        "and the rest of the line is the growing item's, got {}",
        widths[1]
    );
}

/// The case a real panel hit: the item is itself a **flex container**, which is
/// what every styled button in this kit is.
///
/// It needed a second fix. A flex container with `width: auto` reported the space
/// it was *offered* along its main axis — the block answer, "as much as I was
/// given" — so under the max-content probe it answered infinity, and the caller
/// fell back to the whole line. The answer has to be summed from its items, and
/// summed rather than read off their placed positions: `justify-content` has
/// already spread them across the infinite line, so the rightmost edge is
/// infinite too.
#[test]
fn a_nested_flex_container_sizes_to_its_own_content() {
    let widths = widths(vec![
        el(2, "nested", vec![ViewNode::text("Tap me")]),
        el(3, "grow", vec![]),
    ]);
    assert_eq!(widths.len(), 2);
    assert!(
        widths[0] < 200.0,
        "a nested flex container sizes to its content, got {}",
        widths[0]
    );
    assert!(
        widths[1] > 600.0,
        "and leaves the line to the growing sibling, got {}",
        widths[1]
    );
}
