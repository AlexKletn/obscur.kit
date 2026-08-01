#![doc = include_str!("spec_ch4b_generated_parts.doc/_module.md")]

mod spec_ch4b_support;

use obscur_kit::identity::NodeId;
use obscur_kit::runtime::UiRuntime;
use obscur_kit::scene::command::PaintCommand;
use obscur_kit::widgets::input::Input;
use spec_ch4b_support::harness::{at, compile, drive, el, hit, runtime, txt, view};

#[doc = include_str!("spec_ch4b_generated_parts.doc/painted_texts.md")]
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

#[doc = include_str!("spec_ch4b_generated_parts.doc/child_count.md")]
fn child_count(rt: &UiRuntime, node: NodeId) -> usize {
    rt.tree().get(node).expect("a live node").children().len()
}

#[doc = include_str!("spec_ch4b_generated_parts.doc/ch4_16_a_generated_content_paints_without_becoming_a_node.md")]
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

#[doc = include_str!("spec_ch4b_generated_parts.doc/ch4_16_b_scrollbar_parts_are_hidden_from_the_component_tree_but_in_devtools.md")]
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

#[doc = include_str!("spec_ch4b_generated_parts.doc/ch4_16_c_the_placeholder_is_an_exported_part_not_a_child.md")]
#[test]
fn ch4_16_c_the_placeholder_is_an_exported_part_not_a_child() {
    let css = "
        Input::part(placeholder) { color: rgb(128,128,128); }
        input { position: absolute; left: 0px; top: 0px;
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
