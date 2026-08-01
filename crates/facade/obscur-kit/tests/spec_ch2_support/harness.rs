#![doc = include_str!("harness.doc/_module.md")]
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

#[doc = include_str!("harness.doc/el.md")]
#[must_use]
pub fn el(id: u64, class: &str, kids: Vec<ViewNode>) -> ViewNode {
    ViewNode::element(ComponentId::from_raw(id))
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text(class.into())))
        .with_children(ViewFragment::from_nodes(kids))
}

#[doc = include_str!("harness.doc/keyed.md")]
#[must_use]
pub fn keyed(id: u64, class: &str, key: &str, kids: Vec<ViewNode>) -> ViewNode {
    el(id, class, kids).with_key(key)
}

#[doc = include_str!("harness.doc/txt.md")]
#[must_use]
pub fn txt(content: &str) -> ViewNode {
    ViewNode::text(content)
}

#[doc = include_str!("harness.doc/view.md")]
#[must_use]
pub fn view(nodes: Vec<ViewNode>) -> View {
    View::from_fragment(ViewFragment::from_nodes(nodes))
}

#[doc = include_str!("harness.doc/runtime.md")]
#[must_use]
pub fn runtime(w: f32, h: f32) -> UiRuntime {
    UiRuntime::new(RuntimeId::from_raw(1), Size::new(w, h))
}

#[doc = include_str!("harness.doc/small_runtime.md")]
#[must_use]
pub fn small_runtime() -> UiRuntime {
    runtime(400.0, 400.0)
}

#[doc = include_str!("harness.doc/commit.md")]
pub fn commit(rt: &mut UiRuntime, v: &View) {
    let sheets: [Stylesheet; 0] = [];
    let _ = rt.render_now(v, &sheets);
}

// -------------------------------------------------------------- tree reading

#[doc = include_str!("harness.doc/children.md")]
#[must_use]
pub fn children(rt: &UiRuntime, node: NodeId) -> Vec<NodeId> {
    rt.tree()
        .get(node)
        .map(|n| n.child_nodes().to_vec())
        .unwrap_or_default()
}

#[doc = include_str!("harness.doc/root_children.md")]
#[must_use]
pub fn root_children(rt: &UiRuntime) -> Vec<NodeId> {
    rt.tree()
        .root()
        .map(|r| children(rt, r))
        .unwrap_or_default()
}

#[doc = include_str!("harness.doc/by_class.md")]
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

#[doc = include_str!("harness.doc/one_by_class.md")]
#[must_use]
pub fn one_by_class(rt: &UiRuntime, class: &str) -> Option<NodeId> {
    let found = by_class(rt, class);
    if found.len() == 1 {
        found.first().copied()
    } else {
        None
    }
}

#[doc = include_str!("harness.doc/by_key.md")]
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

#[doc = include_str!("harness.doc/all_text.md")]
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

#[doc = include_str!("harness.doc/has_text.md")]
#[must_use]
pub fn has_text(rt: &UiRuntime, needle: &str) -> bool {
    all_text(rt).iter().any(|t| t == needle)
}

#[doc = include_str!("harness.doc/count_class.md")]
#[must_use]
pub fn count_class(rt: &UiRuntime, class: &str) -> usize {
    by_class(rt, class).len()
}

#[doc = include_str!("harness.doc/retained_prop.md")]
#[must_use]
pub fn retained_prop(rt: &UiRuntime, node: NodeId, name: InternedSymbol) -> Option<ViewValue> {
    rt.tree()
        .get(node)?
        .props()
        .iter()
        .find(|p| p.name() == name)
        .map(|p| p.value().clone())
}

#[doc = include_str!("harness.doc/color_of.md")]
#[must_use]
pub fn color_of(rt: &UiRuntime, node: NodeId) -> Option<obscur_kit::style::property::StyleColor> {
    rt.computed_style(node)?
        .get(obscur_kit::style::property::PropertyId::Color)
        .and_then(obscur_kit::style::computed::ComputedValue::as_color)
}

#[doc = include_str!("harness.doc/compile_css.md")]
#[must_use]
pub fn compile_css(css: &str) -> Stylesheet {
    use obscur_kit::css::compiler::StylesheetCompiler;
    use obscur_kit::css::parser::StylesheetParser;
    use obscur_kit::css::source::CssSource;
    let src = CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&src).parse();
    StylesheetCompiler::new().compile(&parsed).matchable
}

#[doc = include_str!("harness.doc/commit_styled.md")]
pub fn commit_styled(rt: &mut UiRuntime, css: &str, v: &View) {
    let sheet = compile_css(css);
    let _ = rt.render_now(v, std::slice::from_ref(&sheet));
}

// ------------------------------------------------------------- view reading

#[doc = include_str!("harness.doc/roots.md")]
#[must_use]
pub fn roots(v: &View) -> &[ViewNode] {
    v.root().nodes()
}

#[doc = include_str!("harness.doc/desired_children.md")]
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

#[doc = include_str!("harness.doc/desired_prop.md")]
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

#[doc = include_str!("harness.doc/desired_text.md")]
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

#[doc = include_str!("harness.doc/view_text.md")]
#[must_use]
pub fn view_text(v: &View) -> Vec<String> {
    let mut out = Vec::new();
    for node in roots(v) {
        collect_text(node, &mut out);
    }
    out
}

// ------------------------------------------------------- `.okui` front end

#[doc = include_str!("harness.doc/parse_okui.md")]
#[must_use]
pub fn parse_okui(source: &str) -> Document {
    Document::cast(Parser::new().parse(source).tree().root().clone())
        .expect("the source parses into a `.okui` document")
}

#[doc = include_str!("harness.doc/okui_diagnostics_with.md")]
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

#[doc = include_str!("harness.doc/okui_diagnostics.md")]
#[must_use]
pub fn okui_diagnostics(source: &str) -> Vec<String> {
    okui_diagnostics_with(source, ContractRegistry::builtins(), &[])
}

#[doc = include_str!("harness.doc/okui_reports.md")]
#[must_use]
pub fn okui_reports(source: &str, needle: &str) -> bool {
    okui_diagnostics(source).iter().any(|d| d.contains(needle))
}

#[doc = include_str!("harness.doc/okui_produces_artifact.md")]
#[must_use]
pub fn okui_produces_artifact(source: &str) -> bool {
    let document = parse_okui(source);
    let result = Compiler::new("spec_ch2.okui")
        .with_registry(ContractRegistry::builtins())
        .compile(&document);
    !result.has_errors() && result.document().is_some()
}
