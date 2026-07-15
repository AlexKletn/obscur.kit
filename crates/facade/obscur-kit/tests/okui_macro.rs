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

use obscur_kit::identity::ComponentId;
use obscur_kit::ui_macros::okui;
use obscur_kit::view::view::{View, ViewNode, ViewValue};
use obscur_kit::widgets::button::Button;

/// The `View`'s single root node.
fn root(view: &View) -> &ViewNode {
    let nodes = view.root().nodes();
    assert_eq!(nodes.len(), 1, "expected exactly one root node");
    &nodes[0]
}

/// The named property's value on `node`.
fn prop<'a>(node: &'a ViewNode, name: &str) -> Option<&'a ViewValue> {
    let ViewNode::Element { props, .. } = node else {
        return None;
    };
    props
        .iter()
        .find(|p| p.name() == obscur_kit::ui_ir::name_binding::attr_symbol(name))
        .map(obscur_kit::view::view::ViewProperty::value)
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
    assert_eq!(prop(&kids[0], "class"), Some(&ViewValue::Text("row".into())));
    let ViewNode::Text { content, .. } = &kids[1] else {
        panic!("the bare string literal lowers to a text node");
    };
    assert_eq!(&**content, "Ready");
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
