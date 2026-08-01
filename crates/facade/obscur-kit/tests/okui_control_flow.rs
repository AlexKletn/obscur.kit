#![doc = include_str!("okui_control_flow.doc/_module.md")]
#![forbid(unsafe_code)]

use obscur_kit::identity::InternedSymbol;
use obscur_kit::ui_macros::okui;
use obscur_kit::view::view::{View, ViewNode, ViewValue};

#[doc = include_str!("okui_control_flow.doc/roots.md")]
fn roots(view: &View) -> &[ViewNode] {
    view.root().nodes()
}

#[doc = include_str!("okui_control_flow.doc/prop.md")]
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
    assert_eq!(
        children.nodes().len(),
        1,
        "a true condition renders its body"
    );
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
fn a_for_renders_one_node_per_item() {
    // THE assertion. This body used to render exactly once, whatever `items` held.
    let items = [String::from("a"), String::from("b"), String::from("c")];
    let view = okui! {
        <container>
            <For each={items} as={item} key={item}>
                <span class={item}/>
            </For>
        </container>
    };
    let ViewNode::Element { children, .. } = &roots(&view)[0] else {
        panic!("expected an element root");
    };
    let classes: Vec<_> = children.nodes().iter().map(|n| prop(n, "class")).collect();
    assert_eq!(
        classes,
        vec![
            Some(ViewValue::Text("a".into())),
            Some(ViewValue::Text("b".into())),
            Some(ViewValue::Text("c".into())),
        ],
        "one node per item, each reading its own item"
    );
}

#[test]
fn an_empty_collection_renders_the_empty_slot() {
    fn list(items: &[String]) -> View {
        okui! {
            <container>
                <For each={items} as={item} key={item}>
                    <span class={item}/>
                    <Empty><span class="none"/></Empty>
                </For>
            </container>
        }
    }

    let view = list(&[]);
    let ViewNode::Element { children, .. } = &roots(&view)[0] else {
        panic!("expected an element root");
    };
    assert_eq!(children.nodes().len(), 1, "the empty slot, and only it");
    assert_eq!(
        prop(&children.nodes()[0], "class"),
        Some(ViewValue::Text("none".into()))
    );

    // …and it must NOT render when there is something to show.
    let view = list(&[String::from("x")]);
    let ViewNode::Element { children, .. } = &roots(&view)[0] else {
        panic!("expected an element root");
    };
    assert_eq!(children.nodes().len(), 1);
    assert_eq!(
        prop(&children.nodes()[0], "class"),
        Some(ViewValue::Text("x".into())),
        "the item, not the empty slot"
    );
}

#[test]
fn a_separator_goes_between_items_and_never_at_the_ends() {
    let items = [String::from("a"), String::from("b"), String::from("c")];
    let view = okui! {
        <container>
            <For each={items} as={item} key={item}>
                <span class={item}/>
                <Separator><span class="sep"/></Separator>
            </For>
        </container>
    };
    let ViewNode::Element { children, .. } = &roots(&view)[0] else {
        panic!("expected an element root");
    };
    let classes: Vec<_> = children
        .nodes()
        .iter()
        .map(|n| match prop(n, "class") {
            Some(ViewValue::Text(text)) => text.to_string(),
            other => panic!("expected a class, got {other:?}"),
        })
        .collect();
    assert_eq!(
        classes,
        vec!["a", "sep", "b", "sep", "c"],
        "n items means n-1 separators, and none before the first or after the last"
    );
}

#[test]
fn one_item_gets_no_separator() {
    let items = [String::from("only")];
    let view = okui! {
        <container>
            <For each={items} as={item} key={item}>
                <span class={item}/>
                <Separator><span class="sep"/></Separator>
            </For>
        </container>
    };
    let ViewNode::Element { children, .. } = &roots(&view)[0] else {
        panic!("expected an element root");
    };
    assert_eq!(children.nodes().len(), 1, "nothing to separate");
}

#[test]
fn each_item_carries_its_own_key() {
    // §2.11's mandatory stable key. Every item used to share the key `"item.id"`
    // — the path's spelling — which is one identity for the whole collection.
    let items = [String::from("a"), String::from("b")];
    let view = okui! {
        <container>
            <For each={items} as={item} key={item}>
                <span class={item}/>
            </For>
        </container>
    };
    let ViewNode::Element { children, .. } = &roots(&view)[0] else {
        panic!("expected an element root");
    };
    let keys: Vec<_> = children
        .nodes()
        .iter()
        .map(|node| {
            let ViewNode::Element { key, .. } = node else {
                panic!("expected elements, got {node:?}")
            };
            key.clone()
        })
        .collect();
    assert!(keys[0].is_some() && keys[1].is_some(), "both are keyed");
    assert_ne!(keys[0], keys[1], "and the two items are told apart");
}

#[test]
fn a_conditional_inside_a_loop_still_keys_its_item() {
    // Why the key is threaded down rather than applied at the loop: the item's
    // root can sit under a branch.
    let items = [String::from("a"), String::from("b")];
    let show = true;
    let view = okui! {
        <container>
            <For each={items} as={item} key={item}>
                <If cond={show}>
                    <span class={item}/>
                </If>
            </For>
        </container>
    };
    let ViewNode::Element { children, .. } = &roots(&view)[0] else {
        panic!("expected an element root");
    };
    assert_eq!(children.nodes().len(), 2);
    for node in children.nodes() {
        let ViewNode::Element { key, .. } = node else {
            panic!("expected elements");
        };
        assert!(
            key.is_some(),
            "a branch's node still carries the item's key"
        );
    }
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
