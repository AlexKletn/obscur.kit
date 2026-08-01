#![doc = include_str!("percentage_in_grid_item.doc/_module.md")]
#![forbid(unsafe_code)]

use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::geometry::Size;
use obscur_kit::identity::{ComponentId, NodeId, RuntimeId};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::view::view::{CLASS, View, ViewFragment, ViewNode, ViewProperty, ViewValue};

#[doc = include_str!("percentage_in_grid_item.doc/VIEWPORT.md")]
const VIEWPORT: Size = Size {
    width: 480.0,
    height: 240.0,
};

#[doc = include_str!("percentage_in_grid_item.doc/SHELL.md")]
const SHELL: &str = ".app { width: 100%; height: 100%; box-sizing: border-box; \
                     padding-left: 24px; padding-right: 24px; \
                     padding-top: 24px; padding-bottom: 24px; \
                     display: grid; grid-template-columns: 1fr 1fr; gap: 10px } \
                     .kid { width: 50% }";

#[doc = include_str!("percentage_in_grid_item.doc/ROW_SHELL.md")]
const ROW_SHELL: &str = ".app { width: 100%; height: 100%; box-sizing: border-box; \
                         padding-left: 24px; padding-right: 24px; \
                         padding-top: 24px; padding-bottom: 24px; \
                         display: grid; grid-template-columns: 1fr 1fr; \
                         grid-template-rows: 100px; gap: 10px } \
                         .kid { height: 50% }";

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

#[doc = include_str!("percentage_in_grid_item.doc/view.md")]
fn view() -> View {
    View::from_fragment(ViewFragment::from_nodes(vec![el(
        "app",
        vec![el("item", vec![el("kid", vec![])])],
    )]))
}

#[doc = include_str!("percentage_in_grid_item.doc/sizes.md")]
fn sizes(item_rule: &str) -> Vec<Size> {
    sizes_in(SHELL, item_rule)
}

#[doc = include_str!("percentage_in_grid_item.doc/sizes_in.md")]
fn sizes_in(shell: &str, item_rule: &str) -> Vec<Size> {
    let sheet = compile(&format!("{shell} .item {{ {item_rule} }}"));
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

#[doc = include_str!("percentage_in_grid_item.doc/widths.md")]
fn widths(item_rule: &str) -> Vec<f32> {
    sizes(item_rule).into_iter().map(|s| s.width).collect()
}

#[doc = include_str!("percentage_in_grid_item.doc/a_percentage_item_hands_its_own_track_down_to_its_children.md")]
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

#[doc = include_str!("percentage_in_grid_item.doc/an_auto_item_was_never_affected.md")]
#[test]
fn an_auto_item_was_never_affected() {
    let widths = widths("width: auto");
    assert!((widths[1] - 211.0).abs() < 0.01, "item {}", widths[1]);
    assert!((widths[2] - 105.5).abs() < 0.01, "kid {}", widths[2]);
}

#[doc = include_str!("percentage_in_grid_item.doc/a_fixed_width_item_keeps_its_declared_width.md")]
#[test]
fn a_fixed_width_item_keeps_its_declared_width() {
    let widths = widths("width: 50px");
    assert!((widths[1] - 50.0).abs() < 0.01, "item {}", widths[1]);
    assert!((widths[2] - 25.0).abs() < 0.01, "kid {}", widths[2]);
}

#[doc = include_str!("percentage_in_grid_item.doc/an_authored_stretch_does_not_override_a_declared_size.md")]
#[test]
fn an_authored_stretch_does_not_override_a_declared_size() {
    let widths = widths("width: 50px; justify-self: stretch; align-self: stretch");
    assert!((widths[1] - 50.0).abs() < 0.01, "item {}", widths[1]);
    assert!((widths[2] - 25.0).abs() < 0.01, "kid {}", widths[2]);
}

#[doc = include_str!("percentage_in_grid_item.doc/a_declared_height_survives_on_the_block_axis_alone.md")]
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

// ---------------------------------------------------------------------------
// The block axis — the same rule, the other half.
//
// The inline fix above stated only the item's grid area *width*, leaving the
// block axis to fall back to the grid container's content height. That left the
// identical defect standing in the other axis, and this is where it is measured.
// `specs/w3c/css-grid-2.txt:3527`, vendored here: "Lay out the grid items into
// their respective containing blocks. Each grid area's width and height are
// considered definite for this purpose."
//
// Every case below uses [`ROW_SHELL`]: a 100px row inside a 192 content height,
// so a child that resolves against the container instead of its row reads 96
// where it should read 50, and the two cannot be confused.
// ---------------------------------------------------------------------------

#[doc = include_str!("percentage_in_grid_item.doc/a_percentage_item_hands_its_own_row_down_to_its_children.md")]
#[test]
fn a_percentage_item_hands_its_own_row_down_to_its_children() {
    let sizes = sizes_in(ROW_SHELL, "height: 100%");
    assert!(
        (sizes[1].height - 100.0).abs() < 0.01,
        "item height {}",
        sizes[1].height
    );
    assert!(
        (sizes[2].height - 50.0).abs() < 0.01,
        "kid height {}",
        sizes[2].height
    );
}

#[doc = include_str!("percentage_in_grid_item.doc/an_auto_item_in_a_definite_row_hands_the_row_down.md")]
#[test]
fn an_auto_item_in_a_definite_row_hands_the_row_down() {
    let sizes = sizes_in(ROW_SHELL, "height: auto");
    assert!(
        (sizes[1].height - 100.0).abs() < 0.01,
        "item height {}",
        sizes[1].height
    );
    assert!(
        (sizes[2].height - 50.0).abs() < 0.01,
        "kid height {}",
        sizes[2].height
    );
}

#[doc = include_str!("percentage_in_grid_item.doc/a_fixed_height_item_hands_its_declared_height_down.md")]
#[test]
fn a_fixed_height_item_hands_its_declared_height_down() {
    let sizes = sizes_in(ROW_SHELL, "height: 60px");
    assert!(
        (sizes[1].height - 60.0).abs() < 0.01,
        "item height {}",
        sizes[1].height
    );
    assert!(
        (sizes[2].height - 30.0).abs() < 0.01,
        "kid height {}",
        sizes[2].height
    );
}

#[doc = include_str!("percentage_in_grid_item.doc/an_explicit_start_still_reads_the_declared_width.md")]
#[test]
fn an_explicit_start_still_reads_the_declared_width() {
    let widths = widths("width: 50px; justify-self: start");
    assert!((widths[1] - 50.0).abs() < 0.01, "item {}", widths[1]);
}
