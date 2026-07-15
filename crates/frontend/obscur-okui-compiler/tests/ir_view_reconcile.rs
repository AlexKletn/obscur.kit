//! End-to-end: a compiled `.okui` document (and a hand-built IR document) is
//! bridged into a desired [`View`] and reconciled into a retained tree — proving
//! the frontend IR and the reconciler connect (audit Area B, spec §2.11/§2.15).

use obscur_identity::StableKey;
use obscur_okui_compiler::bridge::UiIrViewBridge;
use obscur_okui_compiler::compile::Compiler;
use obscur_okui_compiler::symbol::Symbol;
use obscur_okui_syntax::ast::Document;
use obscur_okui_syntax::parser::Parser;
use obscur_tree::reconciliation::Reconciler;
use obscur_tree::tree::RetainedTree;
use obscur_ui_ir::document::{UiIrDocument, UiIrExpression, UiIrNode};

fn parse(src: &str) -> Document {
    Document::cast(Parser::new().parse(src).tree().root().clone()).expect("Document")
}

fn child_ids(tree: &RetainedTree, parent: obscur_identity::NodeId) -> Vec<obscur_identity::NodeId> {
    tree.get(parent)
        .map(|node| node.children().to_vec())
        .unwrap_or_default()
}

#[test]
fn compiled_okui_with_a_for_loop_reconciles_into_a_retained_tree() {
    // A `<For key>` inside a `<ul>`: the loop statically materializes its `<li>`
    // template body under the list.
    let doc = parse("<ul><For each={items} key={id}><li/></For></ul>");
    let result = Compiler::new("app.okui")
        .defining(Symbol::prop("items"))
        .defining(Symbol::prop("id"))
        .compile(&doc);
    let ir = result.document().expect("a lowered document");

    let view = UiIrViewBridge::document_to_view(ir);

    let mut tree = RetainedTree::new();
    let plan = Reconciler::new().reconcile_root(&mut tree, &view);
    assert!(!plan.is_empty());

    // root -> ul -> li
    let root = tree.root().expect("root created");
    let top = child_ids(&tree, root);
    assert_eq!(top.len(), 1, "one top-level <ul>");
    let ul = top[0];
    assert!(tree.get(ul).expect("live").kind().is_element());
    let ul_children = child_ids(&tree, ul);
    assert_eq!(
        ul_children.len(),
        1,
        "the <For> materialized its <li> template"
    );
    assert!(tree.get(ul_children[0]).expect("live").kind().is_element());
}

#[test]
fn hand_built_ir_document_reconciles_with_keys_and_text() {
    // <section><A key="alpha"/>hello</section>
    let root = UiIrNode::element(1, "section")
        .with_child(UiIrNode::component(2, "A").with_key(UiIrExpression::text("alpha")))
        .with_child(UiIrNode::text(3, "hello"));
    let document = UiIrDocument::new(root);

    let view = UiIrViewBridge::document_to_view(&document);
    let mut tree = RetainedTree::new();
    let _ = Reconciler::new().reconcile_root(&mut tree, &view);

    let root_id = tree.root().expect("root");
    let section = child_ids(&tree, root_id);
    assert_eq!(section.len(), 1);
    let children = child_ids(&tree, section[0]);
    assert_eq!(children.len(), 2, "the keyed element and the text node");

    // The keyed element preserved its stable key across reconciliation.
    let keyed = tree.get(children[0]).expect("live");
    assert!(keyed.kind().is_element());
    assert_eq!(keyed.key(), Some(&StableKey::from("alpha")));

    // The text child is a real retained text node.
    let text = tree.get(children[1]).expect("live");
    assert_eq!(text.kind().text(), Some("hello"));
}
