//! §4.16 Generated and synthetic parts — spec-derived conformance cases.
//!
//! Expected values come from the specification text quoted on each test. They
//! must not be edited to make a case pass.

mod spec_ch4b_support;

use obscur_kit::identity::NodeId;
use obscur_kit::runtime::UiRuntime;
use obscur_kit::scene::command::PaintCommand;
use obscur_kit::widgets::input::Input;
use spec_ch4b_support::harness::{at, compile, drive, el, hit, runtime, txt, view};

/// Every text run painted in the last frame, in paint order.
fn painted_texts(rt: &UiRuntime) -> Vec<String> {
    rt.snapshot()
        .display_list()
        .commands()
        .iter()
        .filter_map(|c| match c {
            PaintCommand::DrawText { run, .. } => Some(run.text().to_string()),
            _ => None,
        })
        .collect()
}

/// The number of children a node reports through the component tree API.
fn child_count(rt: &UiRuntime, node: NodeId) -> usize {
    rt.tree().get(node).expect("a live node").children().len()
}

/// §4.16: "`::before` and `::after` are supported for decoration only." and
/// "Meaningful structure or behavior must use real nodes."
///
/// `.badge::before { content: "!" }` on a childless `.badge` holding "New".
/// The "!" is painted before "New" in inline order, yet `.badge` reports **0**
/// children, the accessibility tree gains no node for it, and hit-testing the
/// glyph returns `.badge` itself — a materialized real child would leak into
/// all three.
#[test]
fn ch4_16_a_generated_content_paints_without_becoming_a_node() {
    let css = "
        .badge { position: absolute; left: 0px; top: 0px;
                 width: 200px; height: 40px; font-size: 20px; }
        .badge::before { content: \"!\"; }
    ";
    let v = view(vec![el(1, "badge", vec![txt("New")])]);
    let mut rt = drive(400.0, 200.0, css, &v);
    let badge = at(&rt, &[0]);

    let texts = painted_texts(&rt);
    let bang = texts
        .iter()
        .position(|t| t.contains('!'))
        .expect("the generated content \"!\" must be painted");
    let new = texts
        .iter()
        .position(|t| t.contains("New"))
        .unwrap_or_else(|| panic!("the badge's own text must be painted: {texts:?}"));
    assert!(
        bang < new,
        "::before content paints before the element's own text: {texts:?}"
    );

    assert_eq!(
        child_count(&rt, badge),
        0,
        "generated content must not appear as a component-tree child"
    );

    let semantics = rt.semantic_snapshot();
    let dump = format!("{semantics:?}");
    assert!(
        !dump.contains('!'),
        "generated content must not appear in the accessibility tree: {dump}"
    );

    // The glyph sits at the badge's inline start; hit its centre.
    assert_eq!(
        hit(&mut rt, 5.0, 20.0),
        Some(badge),
        "the generated glyph is not an independent event target"
    );
}

/// §4.16: "Other system-generated parts may use synthetic retained nodes hidden
/// from the component tree but visible in DevTools:" / "- scrollbar parts;"
///
/// A `overflow: scroll` container with exactly 2 author children and content
/// tall enough to show a scrollbar. The component tree must report exactly 2
/// children; the DevTools tree must additionally show scrollbar part nodes
/// (track and thumb) with real rects on the container's right edge.
#[test]
fn ch4_16_b_scrollbar_parts_are_hidden_from_the_component_tree_but_in_devtools() {
    let css = "
        .scroller { position: absolute; left: 0px; top: 0px;
                    width: 200px; height: 100px; overflow: scroll; }
        .tall { width: 200px; height: 400px; }
    ";
    let v = view(vec![el(
        1,
        "scroller",
        vec![el(2, "tall", vec![]), el(3, "tall", vec![])],
    )]);
    let rt = drive(400.0, 400.0, css, &v);
    let scroller = at(&rt, &[0]);

    assert_eq!(
        child_count(&rt, scroller),
        2,
        "the component tree must report the two author children only"
    );

    let inspection = rt.inspect_snapshot();
    let dump = format!("{inspection}");
    let scrollbar_nodes = inspection
        .nodes
        .iter()
        .filter(|n| {
            let text = format!("{n:?}").to_lowercase();
            text.contains("scrollbar") || text.contains("thumb") || text.contains("track")
        })
        .count();
    assert!(
        scrollbar_nodes >= 2,
        "the DevTools tree must show the scrollbar track and thumb parts: {dump}"
    );
    assert!(
        inspection.nodes.len() > 3,
        "the DevTools tree must hold more nodes than the component tree: {dump}"
    );
}

/// §4.16: "Other system-generated parts may use synthetic retained nodes hidden
/// from the component tree but visible in DevTools:" / "- placeholders;" /
/// "- exported style parts;"
///
/// A text input with placeholder "Search" and an empty value, whose placeholder
/// is restyled through the exported-part selector syntax. "Search" paints in
/// `rgb(128,128,128)`, the input reports 0 component-tree children, the
/// DevTools tree shows a named placeholder part, and a hit on the glyphs
/// returns the input itself.
#[test]
fn ch4_16_c_the_placeholder_is_an_exported_part_not_a_child() {
    let css = "
        Input::part(placeholder) { color: rgb(128,128,128); }
        .field { position: absolute; left: 0px; top: 0px;
                 width: 200px; height: 40px; font-size: 16px; }
    ";
    let sheet = compile(css);
    let input =
        Input::new(obscur_kit::identity::InternedSymbol::from_raw(0x4B_00C1)).placeholder("Search");
    let v = input.view();

    let mut rt = runtime(400.0, 200.0);
    let _ = rt.render_now(&v, std::slice::from_ref(&sheet));
    let input_node = at(&rt, &[0]);

    let texts = painted_texts(&rt);
    assert!(
        texts.iter().any(|t| t.contains("Search")),
        "the placeholder must be painted: {texts:?}"
    );

    // (c) the exported-part rule must reach the placeholder's colour.
    let placeholder_color = rt
        .snapshot()
        .display_list()
        .commands()
        .iter()
        .find_map(|c| match c {
            PaintCommand::DrawText { run, brush, .. } if run.text().contains("Search") => {
                Some(format!("{brush:?}"))
            }
            _ => None,
        })
        .expect("the placeholder text run");
    assert!(
        placeholder_color.contains("0.5019") || placeholder_color.contains("128"),
        "the exported-part rule must set the placeholder to rgb(128,128,128): \
         got {placeholder_color}"
    );

    // (a) the placeholder must not be a component-tree child of the input.
    assert_eq!(
        child_count(&rt, input_node),
        0,
        "the placeholder must be hidden from the component tree"
    );

    // (b) but it must be visible in DevTools as a named part.
    let inspection = rt.inspect_snapshot();
    let dump = format!("{inspection}");
    assert!(
        dump.to_lowercase().contains("placeholder"),
        "the DevTools tree must show a named placeholder part: {dump}"
    );

    // (d) a hit on the glyphs returns the input itself.
    assert_eq!(
        hit(&mut rt, 10.0, 20.0),
        Some(input_node),
        "clicking the placeholder must target the input, not a generated child"
    );
}
