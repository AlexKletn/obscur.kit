//! End-to-end: a body `{expr}` **renders its value**, on both front ends
//! (spec §2.8, §2.10).
//!
//! `<div>{user.name}</div>` rendered the literal text `user.name` — the nine
//! characters of the binding's own source. Not a missing feature: a document that
//! silently showed its own spelling. The attribute case was fixed and this one,
//! the more common one, was left live, and the two front ends drifted while it
//! was: the macro emitted the path as text, and the `.okui` file emitted an
//! element named `Expression` that renders as an empty box.
//!
//! One representation now serves both — `UiIrNode::interpolation`, which carries
//! the expression rather than its spelling — so `ViewCodegen` emits the same text
//! node from either. The macro tests below check a `View` built by the real
//! `okui!` with real locals in scope; the file test checks the Rust the file front
//! end emits for the same document.

#![forbid(unsafe_code)]

use obscur_kit::okui_compiler::codegen::ViewCodegen;
use obscur_kit::okui_compiler::compile::Compiler;
use obscur_kit::okui_syntax::ast::Document;
use obscur_kit::okui_syntax::parser::Parser;
use obscur_kit::ui_macros::okui;
use obscur_kit::view::view::{View, ViewNode};

/// `node`'s text content.
fn text(node: &ViewNode) -> &str {
    let ViewNode::Text { content, .. } = node else {
        panic!("expected a text node, got {node:?}");
    };
    content
}

/// `node`'s children.
fn children(node: &ViewNode) -> &[ViewNode] {
    let ViewNode::Element { children, .. } = node else {
        panic!("expected an element, got {node:?}");
    };
    children.nodes()
}

/// The view's single root node.
fn root(view: &View) -> &ViewNode {
    &view.root().nodes()[0]
}

/// The Rust source the `.okui` file front end emits for `source`.
fn generated(source: &str) -> String {
    let parsed = Parser::new().parse(source);
    let document = Document::cast(parsed.tree().root().clone()).expect("root casts to Document");
    let result = Compiler::new("test.okui").compile(&document);
    assert!(
        !result.has_errors(),
        "unexpected diagnostics: {:?}",
        result.diagnostics()
    );
    ViewCodegen::document_to_view_fn(result.document().expect("a lowered document"), "view")
}

#[test]
fn a_body_binding_renders_the_value_not_its_source_text() {
    // THE assertion. This used to render the text "user.name".
    struct User {
        name: String,
    }
    let user = User {
        name: String::from("ada"),
    };
    let view = okui! { <div>{user.name}</div> };
    assert_eq!(
        text(&children(root(&view))[0]),
        "ada",
        "the body binding must carry what the local holds"
    );
}

#[test]
fn a_body_binding_displays_a_non_string() {
    // `Display` is the bound for text content, where an attribute's is
    // `Into<ViewValue>`: `{count}` renders `7`, not a type error.
    let count: i64 = 7;
    let view = okui! { {count} };
    assert_eq!(text(root(&view)), "7");
}

#[test]
fn a_body_binding_reads_the_loop_variable() {
    // The two compose only because both are ordinary Rust: `as={item}` opens a
    // real binding, and the body's `{item}` is a path resolved in it.
    let items = [String::from("a"), String::from("b")];
    let view = okui! {
        <container>
            <For each={items} as={item} key={item}>
                <span>{item}</span>
            </For>
        </container>
    };
    let texts: Vec<&str> = children(root(&view))
        .iter()
        .map(|span| text(&children(span)[0]))
        .collect();
    assert_eq!(texts, vec!["a", "b"], "each item's own text");
}

#[test]
fn a_body_binding_that_is_not_a_bare_path_still_means_what_it_says() {
    // A "path" is only mostly a path: both front ends fall back to a binding on
    // the raw text for anything not classified as a literal. Unparenthesised,
    // this emits `&left + right` — a different expression, and one that happens
    // to compile.
    let left: i64 = 2;
    let right: i64 = 3;
    let view = okui! { {left + right} };
    assert_eq!(text(root(&view)), "5");
}

#[test]
fn a_literal_body_interpolation_stays_a_literal() {
    let view = okui! { <div>{"hi"}</div> };
    assert_eq!(text(&children(root(&view))[0]), "hi");
}

#[test]
fn the_file_front_end_emits_the_same_text_node() {
    // The other front end. It used to emit an element named `Expression` — an
    // unknown component, so an empty box where the value belongs.
    let source = generated("<div>{user.name}</div>");
    assert!(
        source.contains("ViewNode::text(::std::string::ToString::to_string(&(user.name)))"),
        "a body binding emits its Rust path, displayed:\n{source}"
    );
    assert!(
        !source.contains(r#""user.name""#),
        "the path's spelling must not survive as a string literal:\n{source}"
    );
}

#[test]
fn the_file_front_end_checks_a_body_expression_against_2_10() {
    // A binding that evaluates must be a binding that was checked: the emitted
    // path becomes real Rust, so §2.10's allow-list has to see it. (This one held
    // before the emission was fixed — the file path's body check landed first.)
    let parsed = Parser::new().parse("<div>{std::fs::read(p)}</div>");
    let document = Document::cast(parsed.tree().root().clone()).expect("root casts to Document");
    let result = Compiler::new("test.okui").compile(&document);
    assert!(
        result.has_errors(),
        "a call in a body is outside §2.10's allow-list: {:?}",
        result.diagnostics()
    );
}
