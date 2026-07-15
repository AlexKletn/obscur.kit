//! End-to-end: `.okui` **conditions decide** and **bindings carry values**
//! (spec §2.10, §2.11).
//!
//! Both were inert, and inert in the worst way — silently wrong rather than
//! unimplemented. `<If cond={open}>` rendered its body whatever `open` was,
//! because the conditional was a transparent passthrough that spliced its
//! children. And a binding emitted its own *spelling*: `value={label}` produced
//! the literal text `"label"`, so a dynamic document rendered its own source code.
//!
//! The fix needs no evaluator. A binding is a **Rust path**, emitted verbatim into
//! the scope the macro expands in, and a condition is a real `if`. Everything
//! below is checked against a `View` built by the actual `okui!` macro, with real
//! locals in scope — the same thing an application writes.

#![forbid(unsafe_code)]

use obscur_kit::identity::InternedSymbol;
use obscur_kit::ui_macros::okui;
use obscur_kit::view::view::{View, ViewNode, ViewValue};

/// The root fragment's nodes.
fn roots(view: &View) -> &[ViewNode] {
    view.root().nodes()
}

/// The value of `node`'s property named by `symbol`, if any.
fn prop(node: &ViewNode, name: &str) -> Option<ViewValue> {
    let ViewNode::Element { props, .. } = node else {
        return None;
    };
    let symbol = InternedSymbol::from_raw(obscur_kit::ui_ir::name_binding::attr_symbol(name).raw());
    props
        .iter()
        .find(|p| p.name() == symbol)
        .map(|p| p.value().clone())
}

#[test]
fn a_false_condition_renders_nothing() {
    // THE assertion. This body used to render regardless.
    let open = false;
    let view = okui! {
        <container>
            <If cond={open}>
                <span class="detail"/>
            </If>
        </container>
    };
    let ViewNode::Element { children, .. } = &roots(&view)[0] else {
        panic!("expected an element root");
    };
    assert!(
        children.nodes().is_empty(),
        "a false condition must render nothing, got {:?}",
        children.nodes()
    );
}

#[test]
fn a_true_condition_renders_its_body() {
    let open = true;
    let view = okui! {
        <container>
            <If cond={open}>
                <span class="detail"/>
            </If>
        </container>
    };
    let ViewNode::Element { children, .. } = &roots(&view)[0] else {
        panic!("expected an element root");
    };
    assert_eq!(children.nodes().len(), 1, "a true condition renders its body");
}

#[test]
fn exactly_one_branch_of_a_chain_renders() {
    // `<If>`/`<ElseIf>`/`<Else>` are siblings in the source but one construct.
    // Emitted separately they would be three independent statements and the
    // `<Else>` would always run — so this is the assertion that the chain is a
    // chain.
    fn pick(first: bool, second: bool) -> View {
        okui! {
            <container>
                <If cond={first}><span class="a"/></If>
                <ElseIf cond={second}><span class="b"/></ElseIf>
                <Else><span class="c"/></Else>
            </container>
        }
    }

    for (first, second, expected) in [
        (true, true, "a"),
        (true, false, "a"),
        (false, true, "b"),
        (false, false, "c"),
    ] {
        let view = pick(first, second);
        let ViewNode::Element { children, .. } = &roots(&view)[0] else {
            panic!("expected an element root");
        };
        assert_eq!(
            children.nodes().len(),
            1,
            "exactly one branch renders for ({first}, {second})"
        );
        assert_eq!(
            prop(&children.nodes()[0], "class"),
            Some(ViewValue::Text(expected.into())),
            "the wrong branch rendered for ({first}, {second})"
        );
    }
}

#[test]
fn a_binding_carries_the_value_not_its_spelling() {
    // The other half: `class={label}` used to produce the text "label".
    let label = String::from("live-value");
    let view = okui! { <span class={label}/> };
    assert_eq!(
        prop(&roots(&view)[0], "class"),
        Some(ViewValue::Text("live-value".into())),
        "the binding must carry what the local holds"
    );
}

#[test]
fn a_binding_reads_a_field_path() {
    struct User {
        name: String,
    }
    let user = User {
        name: String::from("ada"),
    };
    let view = okui! { <span class={user.name}/> };
    assert_eq!(
        prop(&roots(&view)[0], "class"),
        Some(ViewValue::Text("ada".into()))
    );
}

#[test]
fn a_binding_converts_a_non_string_field() {
    // `ViewValue::from` is the seam, so any type it accepts works — and any type
    // it does not is a compile error in the caller's build, which is the whole
    // type system of the expression language.
    let count: i64 = 42;
    let view = okui! { <span class={count}/> };
    assert_eq!(prop(&roots(&view)[0], "class"), Some(ViewValue::Int(42)));
}

#[test]
fn a_condition_and_a_binding_compose() {
    let show = true;
    let label = String::from("shown");
    let view = okui! {
        <container>
            <If cond={show}>
                <span class={label}/>
            </If>
        </container>
    };
    let ViewNode::Element { children, .. } = &roots(&view)[0] else {
        panic!("expected an element root");
    };
    assert_eq!(children.nodes().len(), 1);
    assert_eq!(
        prop(&children.nodes()[0], "class"),
        Some(ViewValue::Text("shown".into()))
    );
}

#[test]
fn a_static_document_is_unchanged() {
    // The regression guard: markup with no bindings and no control flow is the
    // common case and must be untouched by any of this.
    let view = okui! {
        <container class="row">
            <span class="a"/>
            <span class="b"/>
        </container>
    };
    let ViewNode::Element { children, .. } = &roots(&view)[0] else {
        panic!("expected an element root");
    };
    assert_eq!(children.nodes().len(), 2);
    assert_eq!(
        prop(&roots(&view)[0], "class"),
        Some(ViewValue::Text("row".into()))
    );
}
