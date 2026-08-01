#![doc = include_str!("spec_ch7_selectors.doc/_module.md")]
#![forbid(unsafe_code)]

#[path = "spec_ch7_support/mod.rs"]
mod support;

use obscur_kit::headless::input::{SelectorResolver, SelectorTarget};
use obscur_kit::identity::{ComponentId, NodeId, StableKey};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::style::computed::ComputedValue;
use obscur_kit::style::property::{PropertyId, StyleColor};
use obscur_kit::view::view::{
    AUTOMATION_ID, CLASS, TEST_ID, View, ViewFragment, ViewKey, ViewNode, ViewProperty, ViewValue,
};
use support::{all_geom, all_nodes, at, drive, view};

const VW: f32 = 1000.0;

fn color_of(rt: &UiRuntime, node: NodeId) -> Option<StyleColor> {
    rt.computed_style(node)?
        .get(PropertyId::Color)
        .and_then(ComputedValue::as_color)
}

#[doc = include_str!("spec_ch7_selectors.doc/tagged.md")]
fn tagged(
    id: u64,
    class: &str,
    test_id: Option<&str>,
    automation_id: Option<&str>,
    kids: Vec<ViewNode>,
) -> ViewNode {
    let mut node = ViewNode::element(ComponentId::from_raw(id))
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text(class.into())))
        .with_children(ViewFragment::from_nodes(kids));
    if let Some(t) = test_id {
        node = node.with_prop(ViewProperty::new(TEST_ID, ViewValue::Text(t.into())));
    }
    if let Some(a) = automation_id {
        node = node.with_prop(ViewProperty::new(AUTOMATION_ID, ViewValue::Text(a.into())));
    }
    node
}

#[doc = include_str!("spec_ch7_selectors.doc/a_test_id_is_not_a_css_attribute.md")]
#[test]
fn a_test_id_is_not_a_css_attribute() {
    let css = ".p { color: green } \
               [test-id] { color: red } \
               [automation-id=\"save\"] { color: blue } \
               .p, .a, .b { width: 100px; height: 10px }";

    let v = view(vec![tagged(
        1,
        "p",
        None,
        None,
        vec![
            tagged(2, "a", Some("a"), Some("save"), vec![]),
            tagged(3, "b", None, None, vec![]),
        ],
    )]);

    let rt = drive(VW, VW, css, &v);
    let green = color_of(&rt, at(&rt, &[0])).expect("the parent resolved a color");
    let tagged_color = color_of(&rt, at(&rt, &[0, 0])).expect("the tagged node resolved a color");
    let plain_color = color_of(&rt, at(&rt, &[0, 1])).expect("the plain node resolved a color");

    assert_eq!(
        tagged_color, green,
        "a node carrying test-id/automation-id still inherits green: neither attribute rule matches"
    );
    assert_eq!(plain_color, green, "the untagged sibling inherits green");
    assert_eq!(
        tagged_color, plain_color,
        "adding a test id must not change computed style at all"
    );
}

#[doc = include_str!("spec_ch7_selectors.doc/test_ids_change_neither_geometry_nor_node_identity.md")]
#[test]
fn test_ids_change_neither_geometry_nor_node_identity() {
    let css = ".row { display: flex; flex-wrap: wrap; width: 300px; height: 100px } \
               .item { width: 25%; height: 20px }";

    let plain = view(vec![tagged(
        1,
        "row",
        None,
        None,
        (0..5)
            .map(|i| tagged(10 + i, "item", None, None, vec![]))
            .collect(),
    )]);
    let identified = view(vec![tagged(
        1,
        "row",
        Some("row"),
        Some("row"),
        (0..5)
            .map(|i| {
                let name = format!("item-{i}");
                tagged(10 + i, "item", Some(&name), Some(&name), vec![])
            })
            .collect(),
    )]);

    let before = drive(VW, VW, css, &plain);
    let after = drive(VW, VW, css, &identified);

    assert_eq!(
        all_geom(&before),
        all_geom(&after),
        "adding test-id/automation-id to all six nodes changes no geometry"
    );
    assert_eq!(
        all_nodes(&before),
        all_nodes(&after),
        "adding test-id/automation-id destroys no node identity"
    );
}

#[doc = include_str!("spec_ch7_selectors.doc/keyed.md")]
fn keyed(key: &str) -> ViewNode {
    ViewNode::element(ComponentId::from_raw(7))
        .with_key(ViewKey::new(StableKey::from(key)))
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("item".into())))
        .with_prop(ViewProperty::new(TEST_ID, ViewValue::Text(key.into())))
}

#[doc = include_str!("spec_ch7_selectors.doc/a_node_id_survives_a_keyed_reorder.md")]
#[test]
fn a_node_id_survives_a_keyed_reorder() {
    let css = ".list { width: 300px; height: 200px } .item { width: 300px; height: 20px }";
    let sheet = obscur_kit::css::compiler::StylesheetCompiler::new()
        .compile(
            &obscur_kit::css::parser::StylesheetParser::new(
                &obscur_kit::css::source::CssSource::anonymous(css),
            )
            .parse(),
        )
        .matchable;

    let list = |keys: &[&str]| -> View {
        view(vec![
            ViewNode::element(ComponentId::from_raw(1))
                .with_prop(ViewProperty::new(CLASS, ViewValue::Text("list".into())))
                .with_children(ViewFragment::from_nodes(
                    keys.iter().map(|k| keyed(k)).collect(),
                )),
        ])
    };

    let mut rt = UiRuntime::new(
        obscur_kit::identity::RuntimeId::from_raw(1),
        obscur_kit::geometry::Size::new(VW, VW),
    );
    let _ = rt.render_now(&list(&["a", "b", "c"]), std::slice::from_ref(&sheet));

    let recorded = SelectorResolver::new()
        .resolve(&rt, &SelectorTarget::test_id("b"))
        .expect("key b resolves before the reorder");

    let _ = rt.render_now(&list(&["c", "b", "a"]), std::slice::from_ref(&sheet));
    let _ = rt.render_now(&list(&["d", "c", "b", "a"]), std::slice::from_ref(&sheet));

    let resolved = SelectorResolver::new()
        .resolve(&rt, &SelectorTarget::test_id("b"))
        .expect("key b still resolves after the reorder");

    assert_eq!(
        resolved, recorded,
        "the recorded node ID still addresses the item bearing key b"
    );
    assert_eq!(
        at(&rt, &[0, 2]),
        recorded,
        "after [d, c, b, a] the recorded ID is the item at index 2"
    );

    let matches = all_nodes(&rt)
        .into_iter()
        .filter(|&n| n == recorded)
        .count();
    assert_eq!(matches, 1, "selecting by that ID returns exactly one node");
}
