//! Shared harness for the `spec_ch2_*` **specification-derived** test files.
//!
//! These tests are NOT implementation-derived. Every expected value in the
//! `spec_ch2_*.rs` files was written by an agent reading obscur.kit §2
//! (*Components, tree & DSL*) **without opening any source in this repository**.
//! The cases are therefore a statement of what the implementation owes the
//! spec, not a record of what it currently does.
//!
//! This module exists only to give those cases a compiling home; it contains no
//! expectations of its own. Nothing here may be "fixed" to make a case pass. A
//! failing `spec_ch2_*` test is a claim about the implementation, not about the
//! test.
#![allow(dead_code, unreachable_pub)]

use obscur_kit::geometry::Size;
use obscur_kit::identity::{ComponentId, InternedSymbol, NodeId, RuntimeId};
use obscur_kit::okui_compiler::compile::{CompileDiagnostic, Compiler, ContractRegistry};
use obscur_kit::okui_syntax::ast::Document;
use obscur_kit::okui_syntax::parser::Parser;
use obscur_kit::runtime::UiRuntime;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::tree::node::NodeKind;
use obscur_kit::view::view::{CLASS, View, ViewFragment, ViewNode, ViewProperty, ViewValue};

// ---------------------------------------------------------------- view build

/// An element node carrying `class`, with `kids` as its children.
#[must_use]
pub fn el(id: u64, class: &str, kids: Vec<ViewNode>) -> ViewNode {
    ViewNode::element(ComponentId::from_raw(id))
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text(class.into())))
        .with_children(ViewFragment::from_nodes(kids))
}

/// An element node carrying `class` and an explicit §2.11 `key`.
#[must_use]
pub fn keyed(id: u64, class: &str, key: &str, kids: Vec<ViewNode>) -> ViewNode {
    el(id, class, kids).with_key(key)
}

/// A text node.
#[must_use]
pub fn txt(content: &str) -> ViewNode {
    ViewNode::text(content)
}

/// A view whose root fragment is `nodes`.
#[must_use]
pub fn view(nodes: Vec<ViewNode>) -> View {
    View::from_fragment(ViewFragment::from_nodes(nodes))
}

/// A headless runtime with a `w x h` viewport.
#[must_use]
pub fn runtime(w: f32, h: f32) -> UiRuntime {
    UiRuntime::new(RuntimeId::from_raw(1), Size::new(w, h))
}

/// A headless runtime with a 400x400 viewport.
#[must_use]
pub fn small_runtime() -> UiRuntime {
    runtime(400.0, 400.0)
}

/// Renders one real frame of `v` with no stylesheets.
pub fn commit(rt: &mut UiRuntime, v: &View) {
    let sheets: [Stylesheet; 0] = [];
    let _ = rt.render_now(v, &sheets);
}

// -------------------------------------------------------------- tree reading

/// The retained children of `node`.
#[must_use]
pub fn children(rt: &UiRuntime, node: NodeId) -> Vec<NodeId> {
    rt.tree()
        .get(node)
        .map(|n| n.child_nodes().to_vec())
        .unwrap_or_default()
}

/// The retained root's children, or an empty vector when there is no root.
#[must_use]
pub fn root_children(rt: &UiRuntime) -> Vec<NodeId> {
    rt.tree()
        .root()
        .map(|r| children(rt, r))
        .unwrap_or_default()
}

/// Every retained node carrying `class`, in `NodeId` order.
#[must_use]
pub fn by_class(rt: &UiRuntime, class: &str) -> Vec<NodeId> {
    let mut found: Vec<NodeId> = rt
        .tree()
        .iter()
        .filter(|(_, n)| n.attributes().has_class(class))
        .map(|(id, _)| id)
        .collect();
    found.sort_unstable();
    found
}

/// The single retained node carrying `class`, or `None`.
#[must_use]
pub fn one_by_class(rt: &UiRuntime, class: &str) -> Option<NodeId> {
    let found = by_class(rt, class);
    if found.len() == 1 {
        found.first().copied()
    } else {
        None
    }
}

/// The retained node whose explicit key is `key`, if exactly one exists.
#[must_use]
pub fn by_key(rt: &UiRuntime, key: &str) -> Option<NodeId> {
    let wanted = obscur_kit::identity::StableKey::from(key);
    let found: Vec<NodeId> = rt
        .tree()
        .iter()
        .filter(|(_, n)| n.key() == Some(&wanted))
        .map(|(id, _)| id)
        .collect();
    if found.len() == 1 {
        found.first().copied()
    } else {
        None
    }
}

/// Every text node's content in the retained tree, in `NodeId` order.
#[must_use]
pub fn all_text(rt: &UiRuntime) -> Vec<String> {
    let mut pairs: Vec<(NodeId, String)> = rt
        .tree()
        .iter()
        .filter_map(|(id, n)| match n.kind() {
            NodeKind::Text { content } => Some((id, content.to_string())),
            _ => None,
        })
        .collect();
    pairs.sort_by_key(|(id, _)| *id);
    pairs.into_iter().map(|(_, t)| t).collect()
}

/// Whether the retained tree contains a text node whose content is `needle`.
#[must_use]
pub fn has_text(rt: &UiRuntime, needle: &str) -> bool {
    all_text(rt).iter().any(|t| t == needle)
}

/// The number of retained nodes carrying `class`.
#[must_use]
pub fn count_class(rt: &UiRuntime, class: &str) -> usize {
    by_class(rt, class).len()
}

/// The value of the property named `name` on the retained node `node`, if it
/// carries one at all.
#[must_use]
pub fn retained_prop(rt: &UiRuntime, node: NodeId, name: InternedSymbol) -> Option<ViewValue> {
    rt.tree()
        .get(node)?
        .props()
        .iter()
        .find(|p| p.name() == name)
        .map(|p| p.value().clone())
}

/// A node's computed `color`.
#[must_use]
pub fn color_of(rt: &UiRuntime, node: NodeId) -> Option<obscur_kit::style::property::StyleColor> {
    rt.computed_style(node)?
        .get(obscur_kit::style::property::PropertyId::Color)
        .and_then(obscur_kit::style::computed::ComputedValue::as_color)
}

/// Compiles authored CSS text into a matchable stylesheet through the real CSS
/// frontend (`source -> parse -> compile`).
#[must_use]
pub fn compile_css(css: &str) -> Stylesheet {
    use obscur_kit::css::compiler::StylesheetCompiler;
    use obscur_kit::css::parser::StylesheetParser;
    use obscur_kit::css::source::CssSource;
    let src = CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&src).parse();
    StylesheetCompiler::new().compile(&parsed).matchable
}

/// Renders one real frame of `v` under a single authored stylesheet.
pub fn commit_styled(rt: &mut UiRuntime, css: &str, v: &View) {
    let sheet = compile_css(css);
    let _ = rt.render_now(v, std::slice::from_ref(&sheet));
}

// ------------------------------------------------------------- view reading

/// The root fragment's nodes.
#[must_use]
pub fn roots(v: &View) -> &[ViewNode] {
    v.root().nodes()
}

/// The children of the desired-view element at `path` (child indices from the
/// first root node).
#[must_use]
pub fn desired_children<'a>(v: &'a View, path: &[usize]) -> &'a [ViewNode] {
    let mut node = &roots(v)[0];
    for &index in path {
        let ViewNode::Element { children, .. } = node else {
            panic!("expected an element on the path");
        };
        node = &children.nodes()[index];
    }
    let ViewNode::Element { children, .. } = node else {
        panic!("expected an element at the end of the path");
    };
    children.nodes()
}

/// The value of the desired node's property named `name`, if any.
#[must_use]
pub fn desired_prop(node: &ViewNode, name: InternedSymbol) -> Option<ViewValue> {
    let ViewNode::Element { props, .. } = node else {
        return None;
    };
    props
        .iter()
        .find(|p| p.name() == name)
        .map(|p| p.value().clone())
}

/// Every text node's content beneath (and including) `node`, in source order.
#[must_use]
pub fn desired_text(node: &ViewNode) -> Vec<String> {
    let mut out = Vec::new();
    collect_text(node, &mut out);
    out
}

fn collect_text(node: &ViewNode, out: &mut Vec<String>) {
    match node {
        ViewNode::Text { content, .. } => out.push(content.to_string()),
        ViewNode::Element { children, .. } => {
            for child in children.nodes() {
                collect_text(child, out);
            }
        }
    }
}

/// Every text node's content in `v`, in source order.
#[must_use]
pub fn view_text(v: &View) -> Vec<String> {
    let mut out = Vec::new();
    for node in roots(v) {
        collect_text(node, &mut out);
    }
    out
}

// ------------------------------------------------------- `.okui` front end

/// Parses `source` as a `.okui` document.
///
/// # Panics
///
/// Panics when the source does not parse into a document at all — a parse
/// failure is a different verdict from the compile diagnostics under test, and
/// silently swallowing it would make every rejection case vacuous.
#[must_use]
pub fn parse_okui(source: &str) -> Document {
    Document::cast(Parser::new().parse(source).tree().root().clone())
        .expect("the source parses into a `.okui` document")
}

/// Compiles `source` through the real `.okui` pipeline with `registry` and the
/// symbols `defining` in scope, returning every diagnostic's rendered message.
#[must_use]
pub fn okui_diagnostics_with(
    source: &str,
    registry: ContractRegistry,
    defining: &[obscur_kit::okui_compiler::symbol::Symbol],
) -> Vec<String> {
    let document = parse_okui(source);
    let mut compiler = Compiler::new("spec_ch2.okui").with_registry(registry);
    for symbol in defining {
        compiler = compiler.defining(symbol.clone());
    }
    compiler
        .compile(&document)
        .diagnostics()
        .iter()
        .map(CompileDiagnostic::to_string)
        .collect()
}

/// Compiles `source` with the built-in contracts and no extra symbols.
#[must_use]
pub fn okui_diagnostics(source: &str) -> Vec<String> {
    okui_diagnostics_with(source, ContractRegistry::builtins(), &[])
}

/// Whether compiling `source` reports a diagnostic containing `needle`.
#[must_use]
pub fn okui_reports(source: &str, needle: &str) -> bool {
    okui_diagnostics(source).iter().any(|d| d.contains(needle))
}

/// Whether compiling `source` produces an output artifact (a lowered document)
/// at all.
#[must_use]
pub fn okui_produces_artifact(source: &str) -> bool {
    let document = parse_okui(source);
    let result = Compiler::new("spec_ch2.okui")
        .with_registry(ContractRegistry::builtins())
        .compile(&document);
    !result.has_errors() && result.document().is_some()
}
