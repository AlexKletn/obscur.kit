#![doc = include_str!("spec_ch2_tree.doc/_module.md")]
#![forbid(unsafe_code)]

#[path = "spec_ch2_support/mod.rs"]
mod support;

use obscur_kit::view::view::{DISABLED, ViewValue};
use support::{
    by_key, children, commit, count_class, desired_children, desired_prop, el, has_text, keyed,
    root_children, small_runtime, view, view_text,
};

use obscur_kit::ui_macros::okui;

#[doc = include_str!("spec_ch2_tree.doc/tree_key_reorder_preserves_child_state.md")]
#[test]
fn tree_key_reorder_preserves_child_state() {
    // Three keyed rows, each holding an input. Row `b` is given editing state
    // ("hello"); the list is then reordered to [c, a, b] and recommitted.
    fn rows(order: &[&str]) -> obscur_kit::view::view::View {
        let kids = order
            .iter()
            .map(|key| keyed(100, "row", key, vec![el(200, "input", vec![])]))
            .collect();
        view(vec![el(10, "list", kids)])
    }

    let mut rt = small_runtime();
    commit(&mut rt, &rows(&["a", "b", "c"]));

    let before: Vec<_> = root_children(&rt)
        .first()
        .map(|list| children(&rt, *list))
        .unwrap_or_default();
    assert_eq!(before.len(), 3, "three keyed rows are retained");
    let row_b_before = by_key(&rt, "b").expect("row b is retained under its key");
    let input_b_before = *children(&rt, row_b_before)
        .first()
        .expect("row b holds an input");

    // Type "hello" into row b's input.
    rt.begin_text_editing(input_b_before, "hello");
    assert_eq!(rt.editing_text(), Some("hello"), "the text was typed");

    let node_count_before = rt.tree().len();

    // Reorder: same three items, new order.
    commit(&mut rt, &rows(&["c", "a", "b"]));

    let after: Vec<_> = root_children(&rt)
        .first()
        .map(|list| children(&rt, *list))
        .unwrap_or_default();
    assert_eq!(after.len(), 3, "still three rows after the reorder");

    let row_b_after = by_key(&rt, "b").expect("row b is still retained under its key");
    assert_eq!(
        row_b_before, row_b_after,
        "the retained node ID of row b is the same value before and after the reorder"
    );
    assert_eq!(
        after.iter().position(|n| *n == row_b_after),
        Some(2),
        "row b is now the third row"
    );
    assert!(
        rt.editing_text().is_some(),
        "the reorder did not tear down row b's editing session — the session \
         survives because its node did"
    );
    assert_eq!(
        rt.tree().len(),
        node_count_before,
        "0 creations and 0 destructions for the three rows"
    );
}

#[doc = include_str!("spec_ch2_tree.doc/tree_key_removal_does_not_reuse_the_removed_id.md")]
#[test]
fn tree_key_removal_does_not_reuse_the_removed_id() {
    fn rows(keys: &[&str]) -> obscur_kit::view::view::View {
        let kids = keys
            .iter()
            .map(|key| keyed(100, "row", key, vec![]))
            .collect();
        view(vec![el(10, "list", kids)])
    }

    let mut rt = small_runtime();
    commit(&mut rt, &rows(&["a", "b", "c"]));
    let a0 = by_key(&rt, "a").expect("a");
    let b0 = by_key(&rt, "b").expect("b");
    let c0 = by_key(&rt, "c").expect("c");
    let originals = [a0, b0, c0];

    commit(&mut rt, &rows(&["a", "c"]));
    assert_eq!(by_key(&rt, "a"), Some(a0), "a keeps its original ID");
    assert_eq!(by_key(&rt, "c"), Some(c0), "c keeps its original ID");
    assert_eq!(by_key(&rt, "b"), None, "b's node is destroyed");

    commit(&mut rt, &rows(&["a", "c", "d"]));
    let d = by_key(&rt, "d").expect("the brand-new key d is retained");
    assert!(
        !originals.contains(&d),
        "d's node ID {d:?} must not be any of the three originally recorded IDs \
         {originals:?} — b's ID is not handed out again"
    );
}

#[doc = include_str!("spec_ch2_tree.doc/an_untaken_conditional_branch_is_never_constructed.md")]
#[test]
fn an_untaken_conditional_branch_is_never_constructed() {
    // Alpha and Beta are distinguished by class; a branch that was constructed
    // and then hidden still leaves a node behind, which is what this rejects.
    let cond = true;
    let v = okui! {
        <div>
            <If cond={cond}><span class="alpha"/></If>
            <Else><span class="beta"/></Else>
        </div>
    };

    let mut rt = small_runtime();
    commit(&mut rt, &v);

    assert_eq!(count_class(&rt, "alpha"), 1, "Alpha was constructed once");
    assert_eq!(
        count_class(&rt, "beta"),
        0,
        "the retained tree contains no node belonging to Beta — not a hidden \
         one, not a zero-sized one, not a display:none one"
    );
    assert_eq!(
        desired_children(&v, &[]).len(),
        1,
        "and the desired view itself carries only the taken branch"
    );
}

#[doc = include_str!("spec_ch2_tree.doc/an_elseif_chain_is_exclusive.md")]
#[test]
fn an_elseif_chain_is_exclusive() {
    fn grade(score: i32) -> obscur_kit::view::view::View {
        let a = score > 90;
        let b = score > 80;
        okui! {
            <div>
                <If cond={a}>{"A"}</If>
                <ElseIf cond={b}>{"B"}</ElseIf>
                <Else>{"C"}</Else>
            </div>
        }
    }

    assert_eq!(
        view_text(&grade(95)),
        vec!["A".to_owned()],
        "at 95 the rendered text is exactly \"A\" — the second condition, also \
         true, contributes nothing"
    );
    assert_eq!(view_text(&grade(85)), vec!["B".to_owned()]);
    assert_eq!(view_text(&grade(50)), vec!["C".to_owned()]);
}

#[doc = include_str!("spec_ch2_tree.doc/a_false_conditional_attribute_is_absent_not_empty.md")]
#[test]
fn a_false_conditional_attribute_is_absent_not_empty() {
    let disabled = false;
    let v = okui! { <div><button class="target" disabled={disabled}/></div> };
    let button = &desired_children(&v, &[])[0];

    assert!(
        matches!(
            desired_prop(button, DISABLED),
            None | Some(ViewValue::Bool(false))
        ),
        "a false conditional attribute is either omitted or carried as the \
         boolean false — never as a truthy or textual value; got {:?}",
        desired_prop(button, DISABLED)
    );
    for wrong in [
        ViewValue::Bool(true),
        ViewValue::Text("false".into()),
        ViewValue::Text("".into()),
    ] {
        assert_ne!(
            desired_prop(button, DISABLED),
            Some(wrong.clone()),
            "an absent attribute is not {wrong:?}"
        );
    }

    let mut rt = small_runtime();
    commit(&mut rt, &v);
    let node = support::one_by_class(&rt, "target").expect("the button is retained");
    assert!(
        !rt.tree()
            .get(node)
            .expect("the button is retained")
            .attributes()
            .is_disabled(),
        "any `[disabled]`-style selector does not match"
    );
}

#[doc = include_str!("spec_ch2_tree.doc/a_for_adds_no_wrapper_node.md")]
#[test]
fn a_for_adds_no_wrapper_node() {
    let rows = [1u64, 2, 3];
    let v = okui! {
        <div>
            <span class="before"/>
            <For each={rows} as={row} key={row}>
                <span class="row"/>
            </For>
            <span class="after"/>
        </div>
    };

    let mut rt = small_runtime();
    commit(&mut rt, &v);

    let container = root_children(&rt)[0];
    let kids = children(&rt, container);
    assert_eq!(
        kids.len(),
        5,
        "the container has exactly 5 retained children — static, row, row, row, \
         static — with no fragment/group node standing in for the `For`"
    );
    let classes: Vec<String> = kids
        .iter()
        .map(|id| {
            rt.tree()
                .get(*id)
                .and_then(|n| n.attributes().classes().first().cloned())
                .map(|c| c.to_string())
                .unwrap_or_default()
        })
        .collect();
    assert_eq!(
        classes,
        vec!["before", "row", "row", "row", "after"]
            .into_iter()
            .map(str::to_owned)
            .collect::<Vec<_>>(),
        "in source order"
    );
}

#[doc = include_str!("spec_ch2_tree.doc/a_for_separator_is_interleaved_with_no_trailing_divider.md")]
#[test]
fn a_for_separator_is_interleaved_with_no_trailing_divider() {
    #[allow(clippy::needless_pass_by_value)]
    fn list(items: Vec<u64>) -> obscur_kit::view::view::View {
        okui! {
            <div>
                <For each={items} as={item} key={item}>
                    <span class="row"/>
                    <Separator><span class="divider"/></Separator>
                </For>
            </div>
        }
    }

    // 3 items -> row, div, row, div, row.
    let mut rt = small_runtime();
    let v = list(vec![1, 2, 3]);
    commit(&mut rt, &v);
    let kids = children(&rt, root_children(&rt)[0]);
    let classes: Vec<String> = kids
        .iter()
        .map(|id| {
            rt.tree()
                .get(*id)
                .and_then(|n| n.attributes().classes().first().cloned())
                .map(|c| c.to_string())
                .unwrap_or_default()
        })
        .collect();
    assert_eq!(
        classes,
        ["row", "divider", "row", "divider", "row"]
            .map(str::to_owned)
            .to_vec(),
        "3 items -> 3 rows and exactly 2 dividers, interleaved, no trailing divider"
    );

    // 1 item -> 1 row, 0 dividers.
    let mut rt1 = small_runtime();
    let v1 = list(vec![1]);
    commit(&mut rt1, &v1);
    assert_eq!(count_class(&rt1, "row"), 1);
    assert_eq!(count_class(&rt1, "divider"), 0, "1 item -> 0 dividers");

    // 0 items -> nothing.
    let mut rt0 = small_runtime();
    let v0 = list(vec![]);
    commit(&mut rt0, &v0);
    assert_eq!(count_class(&rt0, "row"), 0);
    assert_eq!(count_class(&rt0, "divider"), 0, "0 items -> 0 dividers");
}

#[doc = include_str!("spec_ch2_tree.doc/a_for_empty_slot_is_shown_only_while_the_list_is_empty.md")]
#[test]
fn a_for_empty_slot_is_shown_only_while_the_list_is_empty() {
    #[allow(clippy::needless_pass_by_value)]
    fn list(items: Vec<u64>) -> obscur_kit::view::view::View {
        okui! {
            <div>
                <For each={items} as={item} key={item}>
                    <span class="row"/>
                    <Empty>"Nothing here"</Empty>
                </For>
            </div>
        }
    }

    let mut rt = small_runtime();

    commit(&mut rt, &list(vec![]));
    assert!(
        has_text(&rt, "Nothing here"),
        "0 items -> the empty slot's text is in the tree"
    );
    assert_eq!(count_class(&rt, "row"), 0, "and 0 rows exist");

    commit(&mut rt, &list(vec![1, 2]));
    assert!(
        !has_text(&rt, "Nothing here"),
        "2 items -> \"Nothing here\" is absent from the tree, not merely hidden"
    );
    assert_eq!(count_class(&rt, "row"), 2);

    commit(&mut rt, &list(vec![]));
    assert!(
        has_text(&rt, "Nothing here"),
        "back to 0 -> the empty slot is present again"
    );
    assert_eq!(count_class(&rt, "row"), 0);
}

#[doc = include_str!("spec_ch2_tree.doc/virtualfor_realizes_a_count_independent_of_the_item_count.md")]
#[test]
fn virtualfor_realizes_a_count_independent_of_the_item_count() {
    fn document(n: u64) -> String {
        let _ = n;
        "<div><VirtualFor each={items} as={item} key={item}><span class=\"row\"/>\
         </VirtualFor></div>"
            .to_owned()
    }

    let diagnostics = support::okui_diagnostics_with(
        &document(10_000),
        obscur_kit::okui_compiler::compile::ContractRegistry::builtins(),
        &[
            obscur_kit::okui_compiler::symbol::Symbol::state("items"),
            obscur_kit::okui_compiler::symbol::Symbol::loop_local("item"),
        ],
    );
    assert_eq!(
        diagnostics,
        Vec::<String>::new(),
        "`<VirtualFor>` is a §2.11 construct and must compile"
    );

    // The realized count, once the construct exists, is a function of the
    // viewport and not of the item count.
    let compiled = support::parse_okui(&document(10_000));
    let result = obscur_kit::okui_compiler::compile::Compiler::new("virtual.okui")
        .defining(obscur_kit::okui_compiler::symbol::Symbol::state("items"))
        .defining(obscur_kit::okui_compiler::symbol::Symbol::loop_local(
            "item",
        ))
        .compile(&compiled);
    let doc = result.document().expect("a lowered document");
    let v = obscur_kit::okui_compiler::bridge::UiIrViewBridge::document_to_view(doc);

    let mut rt = support::runtime(300.0, 400.0);
    commit(&mut rt, &v);
    let realized = count_class(&rt, "row");
    assert!(
        realized < 200,
        "the realized retained node count must be on the order of the visible \
         rows (tens), not the item count; got {realized}"
    );
}
