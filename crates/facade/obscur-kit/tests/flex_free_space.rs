#![doc = include_str!("flex_free_space.doc/_module.md")]
#![forbid(unsafe_code)]

use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::geometry::Size;
use obscur_kit::identity::{ComponentId, NodeId, RuntimeId};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::view::view::{CLASS, View, ViewFragment, ViewNode, ViewProperty, ViewValue};

#[doc = include_str!("flex_free_space.doc/VIEWPORT.md")]
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

#[doc = include_str!("flex_free_space.doc/widths.md")]
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

#[doc = include_str!("flex_free_space.doc/a_growing_item_takes_the_space_a_fixed_sibling_leaves.md")]
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

#[doc = include_str!("flex_free_space.doc/a_plain_item_does_not_absorb_the_space_a_sibling_asked_to_grow_into.md")]
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

#[doc = include_str!("flex_free_space.doc/an_explicit_auto_basis_sizes_to_content_too.md")]
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

#[doc = include_str!("flex_free_space.doc/a_nested_flex_container_sizes_to_its_own_content.md")]
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
