//! End-to-end proof that the spec's §2.8 **primary** authoring surface — the
//! `okui!` proc-macro — exists, expands, and compiles to ordinary code.
//!
//! The macro is the point at which "`.okui` compiles to ordinary Rust, not a
//! runtime" has to hold for the *inline* path: `okui!` parses the TSX-like DSL,
//! lowers it to UI IR **while the macro runs**, and emits the `View`-building
//! Rust that `ViewCodegen` produces. Nothing here interprets an IR document at
//! runtime — the assertions below inspect a plain `View`.
//!
//! These tests are the call site the audit found missing: previously
//! `DslMacro::expand` was solid logic that no macro invoked.
#![forbid(unsafe_code)]

use obscur_kit::identity::{ComponentId, InternedSymbol};
use obscur_kit::screen::OkuiScreen;
use obscur_kit::ui_macros::okui;
use obscur_kit::view::view::{AUTOMATION_ID, TEST_ID, View, ViewNode, ViewValue};
use obscur_kit::widgets::button::Button;

/// The `View`'s single root node.
fn root(view: &View) -> &ViewNode {
    let nodes = view.root().nodes();
    assert_eq!(nodes.len(), 1, "expected exactly one root node");
    &nodes[0]
}

/// The named property's value on `node`.
fn prop<'a>(node: &'a ViewNode, name: &str) -> Option<&'a ViewValue> {
    by_symbol(node, obscur_kit::ui_ir::name_binding::attr_symbol(name))
}

/// The value `node` carries under `symbol`, asked for by handle rather than by
/// spelling — the only way to tell a *reserved* binding from a hashed one, since
/// both are just `InternedSymbol`s once a `View` exists.
fn by_symbol(node: &ViewNode, symbol: InternedSymbol) -> Option<&ViewValue> {
    let ViewNode::Element { props, .. } = node else {
        return None;
    };
    props
        .iter()
        .find(|p| p.name() == symbol)
        .map(obscur_kit::view::view::ViewProperty::value)
}

/// Every property on `node` as (raw symbol, value), in authored order — the whole
/// of what a front end decided about an element's attributes, comparable across
/// the two front ends.
fn props_of(node: &ViewNode) -> Vec<(u32, ViewValue)> {
    let ViewNode::Element { props, .. } = node else {
        panic!("expected an element node");
    };
    props
        .iter()
        .map(|p| (p.name().raw(), p.value().clone()))
        .collect()
}

#[test]
fn the_macro_expands_to_a_real_view() {
    // A lone element becomes the root directly.
    let view = okui! { <container class="row"/> };
    let node = root(&view);
    assert!(matches!(node, ViewNode::Element { .. }));
    assert_eq!(
        prop(node, "class"),
        Some(&ViewValue::Text("row".into())),
        "the authored class reached the View as a real property"
    );
}

#[test]
fn an_uppercase_tag_binds_to_the_real_component_id() {
    // THE point of the shared `name_binding` registry: `<Button>` is not a
    // string that some runtime resolves later — it is lowered to the widget's
    // actual COMPONENT_ID at expansion time.
    let view = okui! { <Button label="OK"/> };
    let ViewNode::Element { component, .. } = root(&view) else {
        panic!("a component tag lowers to an element node");
    };
    assert_eq!(*component, Button::COMPONENT_ID);
}

#[test]
fn a_lowercase_tag_is_a_plain_container_not_a_component() {
    let view = okui! { <container/> };
    let ViewNode::Element { component, .. } = root(&view) else {
        panic!("an element tag lowers to an element node");
    };
    assert_eq!(*component, ComponentId::from_raw(0x00C0_0001));
}

#[test]
fn children_text_and_nesting_survive_expansion() {
    let view = okui! {
        <container class="card">
            <container class="row"/>
            "Ready"
        </container>
    };
    let ViewNode::Element { children, .. } = root(&view) else {
        panic!("root is an element");
    };
    let kids = children.nodes();
    assert_eq!(kids.len(), 2);
    assert_eq!(
        prop(&kids[0], "class"),
        Some(&ViewValue::Text("row".into()))
    );
    let ViewNode::Text { content, .. } = &kids[1] else {
        panic!("the bare string literal lowers to a text node");
    };
    assert_eq!(&**content, "Ready");
}

#[test]
fn a_hyphenated_attribute_reaches_the_reserved_symbol_it_names() {
    // Audit 2-3. `test-id="a"` was `expected ident` on this front end, so §7.5's
    // stable identifiers — the names `App::on_command_at` and
    // `App::add_event_listener` target a node by — could be written in a `.okui`
    // file and nowhere else. What has to hold is not merely that it parses: the
    // joined name must land on the *reserved* handle `SelectorResolver` reads,
    // because a hashed one is a property nothing ever looks at.
    let view = okui! { <div test-id="row" automation-id="auto" data-kind="cell"/> };
    let node = root(&view);
    assert_eq!(
        by_symbol(node, TEST_ID),
        Some(&ViewValue::Text("row".into()))
    );
    assert_eq!(
        by_symbol(node, AUTOMATION_ID),
        Some(&ViewValue::Text("auto".into()))
    );
    // And an unreserved hyphenated name is not lost either: it hashes as the
    // whole string, `data-kind`, not as `data`.
    assert_eq!(
        prop(node, "data-kind"),
        Some(&ViewValue::Text("cell".into()))
    );
    assert_eq!(prop(node, "data"), None);
}

#[test]
fn the_two_front_ends_lower_a_hyphenated_name_identically() {
    // §2.8's two front ends are one language, and audit 2-3 was them disagreeing
    // about what an attribute may be *called*: the file lexer's identifier rule
    // admits `-`, and Rust's does not. The claim here is the strong one — not
    // "both accept it" but "both produce the same properties, under the same
    // handles, in the same order", which is what makes a `.okui` file and an
    // `okui!` block interchangeable.
    const MARKUP: &str = r#"<div class="row" test-id="a" automation-id="auto" data-kind="cell"/>"#;
    let file = OkuiScreen::compile(MARKUP).expect("the file front end compiles it");
    let file_view = file.view();
    let macro_view = okui! {
        <div class="row" test-id="a" automation-id="auto" data-kind="cell"/>
    };
    assert_eq!(props_of(root(&file_view)), props_of(root(&macro_view)));
}

#[test]
fn the_names_only_the_file_front_end_admits_are_recorded_not_imagined() {
    // The exact width of the disagreement that survives audit 2-3, measured
    // rather than asserted from reading a lexer — and it is the claim
    // `docs/api/09-authoring-formats.md` now makes in a table, so it has to be
    // true. The file lexer's identifier rule is shared with *value* position
    // (`value=item.name` must lex as one token), so a dot and a digit-initial
    // segment are admitted in name position incidentally. None of them spells a
    // reserved attribute; none can be handed to a proc-macro as an identifier.
    for markup in [
        r#"<div test-="x"/>"#,    // a trailing hyphen
        r#"<div data-1="x"/>"#,   // a digit-initial segment
        r#"<div a.b="x"/>"#,      // a dot inside the name
        r#"<my-tag class="x"/>"#, // and the same rule applied to a TAG
    ] {
        assert!(
            OkuiScreen::compile(markup).is_ok(),
            "the file front end accepts `{markup}`, and the macro cannot"
        );
    }
}

#[test]
fn multiple_roots_splice_through_the_transparent_fragment() {
    // Several top-level nodes are wrapped in an IR fragment, and a fragment is
    // TRANSPARENT: codegen splices its children into the root list rather than
    // materialising a node of its own. So two authored siblings stay two roots.
    let view = okui! { <container class="a"/><container class="b"/> };
    let roots = view.root().nodes();
    assert_eq!(roots.len(), 2, "the fragment itself must not become a node");
    assert_eq!(prop(&roots[0], "class"), Some(&ViewValue::Text("a".into())));
    assert_eq!(prop(&roots[1], "class"), Some(&ViewValue::Text("b".into())));
}
