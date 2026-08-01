#![doc = include_str!("okui_screen.doc/_module.md")]

use std::collections::HashSet;

use crate::okui_compiler::bridge::UiIrViewBridge;
use crate::okui_compiler::compile::Compiler;
use crate::okui_compiler::symbol::Symbol;
use crate::okui_syntax::ast::Document;
use crate::okui_syntax::parser::Parser;
use crate::ui_ir::document::{UiIrDocument, UiIrExpression, UiIrNode, UiIrNodeKind, UiIrValue};
use crate::ui_ir::name_binding;
use crate::view::view::{View, ViewFragment, ViewNode, ViewProperty, ViewValue};

use super::{ScreenData, ScreenError};

#[doc = include_str!("okui_screen.doc/SCREEN_NAME.md")]
const SCREEN_NAME: &str = "screen.okui";

#[doc = include_str!("okui_screen.doc/VIRTUAL_FOR.md")]
const VIRTUAL_FOR: &str = "VirtualFor";

#[doc = include_str!("okui_screen.doc/OkuiScreen.md")]
pub struct OkuiScreen {
    #[doc = include_str!("okui_screen.doc/document.md")]
    document: UiIrDocument,
    #[doc = include_str!("okui_screen.doc/data.md")]
    data: ScreenData,
    #[doc = include_str!("okui_screen.doc/dynamic.md")]
    dynamic: HashSet<u64>,
}

impl OkuiScreen {
    #[doc = include_str!("okui_screen.doc/compile.md")]
    pub fn compile(source: &str) -> Result<Self, ScreenError> {
        Self::compile_with(source, ScreenData::new())
    }

    #[doc = include_str!("okui_screen.doc/compile_with.md")]
    pub fn compile_with(source: &str, data: ScreenData) -> Result<Self, ScreenError> {
        let parsed = Parser::new().parse(source);
        if !parsed.errors().is_empty() {
            return Err(ScreenError::Parse(
                parsed.errors().iter().map(ToString::to_string).collect(),
            ));
        }
        let document = Document::cast(parsed.tree().root().clone()).ok_or_else(|| {
            ScreenError::Parse(vec!["the `.okui` root is not a document".to_owned()])
        })?;
        let mut compiler = Compiler::new(SCREEN_NAME);
        for name in data.names() {
            compiler = compiler.defining(Symbol::state(name));
        }
        let result = compiler.compile(&document);
        if result.has_errors() {
            return Err(ScreenError::Compile(
                result
                    .diagnostics()
                    .iter()
                    .map(ToString::to_string)
                    .collect(),
            ));
        }
        let document = result
            .into_document()
            .ok_or_else(|| ScreenError::Compile(vec!["no document was produced".to_owned()]))?;
        let mut dynamic = HashSet::new();
        mark_dynamic(document.root(), &mut dynamic);
        Ok(Self {
            document,
            data,
            dynamic,
        })
    }

    #[doc = include_str!("okui_screen.doc/data_2.md")]
    #[must_use]
    pub const fn data(&self) -> &ScreenData {
        &self.data
    }

    #[doc = include_str!("okui_screen.doc/document_2.md")]
    #[must_use]
    pub const fn document(&self) -> &UiIrDocument {
        &self.document
    }

    #[doc = include_str!("okui_screen.doc/dynamic_node_count.md")]
    #[must_use]
    pub fn dynamic_node_count(&self) -> usize {
        self.dynamic.len()
    }

    #[doc = include_str!("okui_screen.doc/view.md")]
    #[must_use]
    pub fn view(&self) -> View {
        View::from_fragment(ViewFragment::from_nodes(self.lower(self.document.root())))
    }

    #[doc = include_str!("okui_screen.doc/lower.md")]
    fn lower(&self, node: &UiIrNode) -> Vec<ViewNode> {
        // A body `{expr}`: the IR carries the expression, and the bridge — which
        // has nothing to evaluate it with — lowers it to an anonymous element with
        // the expression under a hashed property, i.e. an empty box. This is the
        // one node kind whose lowering is *replaced* rather than repaired.
        if let Some(expression) = node.interpolated_value() {
            return vec![ViewNode::text(self.render(expression))];
        }
        // Nothing in this subtree reads the data, so the bridge's lowering of it is
        // already this frame's answer. This is what keeps a mostly static screen at
        // one bridge call.
        if !self.dynamic.contains(&node.id()) {
            return UiIrViewBridge::node_to_view_nodes(node);
        }
        match node.kind() {
            // A real element: keep the bridge's node, substitute its own bound
            // properties, and re-lower its children through this path so a nested
            // binding is substituted too.
            UiIrNodeKind::Element | UiIrNodeKind::Component => {
                let mut lowered = UiIrViewBridge::node_to_view_nodes(node);
                for view in &mut lowered {
                    self.substitute(node, view);
                }
                lowered
            }
            // Transparent kinds: the bridge splices their children into the
            // surrounding fragment, so the re-lowered children *are* the result.
            // Read from the node's own `kind`, not re-derived from its tag — the
            // front ends already decided which it is.
            UiIrNodeKind::Fragment | UiIrNodeKind::Conditional | UiIrNodeKind::Loop => {
                self.lower_children(node)
            }
            // Literal content: nothing to substitute (and never marked dynamic —
            // this arm exists so the match stays total rather than by luck).
            UiIrNodeKind::Text => UiIrViewBridge::node_to_view_nodes(node),
        }
    }

    #[doc = include_str!("okui_screen.doc/lower_children.md")]
    fn lower_children(&self, node: &UiIrNode) -> Vec<ViewNode> {
        node.children()
            .iter()
            .flat_map(|child| self.lower(child))
            .collect()
    }

    #[doc = include_str!("okui_screen.doc/substitute.md")]
    fn substitute(&self, node: &UiIrNode, view: &mut ViewNode) {
        let ViewNode::Element {
            props, children, ..
        } = view
        else {
            return;
        };
        for property in node.properties() {
            let UiIrExpression::Binding(path) = property.value() else {
                continue;
            };
            let Some(value) = self.data.get(path) else {
                // Unbound: the bridge left the path there, and that is what stays.
                continue;
            };
            // The symbol the bridge bound this attribute to — through the same
            // tag-aware function it used, so the two cannot disagree about which
            // property to overwrite.
            let symbol = name_binding::attr_symbol_on(node.tag(), property.name());
            for existing in props.iter_mut().filter(|prop| prop.name() == symbol) {
                *existing = ViewProperty::new(symbol, value.clone());
            }
        }
        // A `<VirtualFor>`'s children are its row template, not content; see the
        // note on `view`. `dynamic` never marks one, so this only guards a template
        // reached through an ancestor that is dynamic for its own reasons.
        if node.kind() == UiIrNodeKind::Loop && node.tag() == VIRTUAL_FOR {
            return;
        }
        *children = ViewFragment::from_nodes(self.lower_children(node));
    }

    #[doc = include_str!("okui_screen.doc/render.md")]
    fn render(&self, expression: &UiIrExpression) -> Box<str> {
        match expression {
            UiIrExpression::Binding(path) => match self.data.get(path) {
                Some(value) => ScreenData::render(&value),
                // Keep the path visible rather than rendering nothing: an unbound
                // binding is a mistake, and a blank line hides it.
                None => path.clone(),
            },
            // Text is its own rendering; the other literals go through the value
            // mapping so one function decides how a number reads.
            UiIrExpression::Literal(UiIrValue::Text(text)) => text.clone(),
            UiIrExpression::Literal(value) => ScreenData::render(&view_value(value)),
            // `{format(x)}` has no evaluator on this route — there are no Rust
            // locals to call anything with. The build-time route emits the call as
            // Rust and is the one that can run it.
            UiIrExpression::Call { .. } => Box::from(""),
        }
    }
}

impl std::fmt::Debug for OkuiScreen {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OkuiScreen")
            .field("nodes", &self.document.node_count())
            .field("dynamic", &self.dynamic.len())
            .field("data", &self.data)
            .finish()
    }
}

#[doc = include_str!("okui_screen.doc/mark_dynamic.md")]
fn mark_dynamic(node: &UiIrNode, out: &mut HashSet<u64>) -> bool {
    // A `<VirtualFor>` row template is shared by every realized row, so a value
    // substituted into it would appear in all of them; stop the walk here.
    if node.kind() == UiIrNodeKind::Loop && node.tag() == VIRTUAL_FOR {
        return false;
    }
    let mut dynamic = node.interpolated_value().is_some()
        || node
            .properties()
            .iter()
            .any(|property| matches!(property.value(), UiIrExpression::Binding(_)));
    for child in node.children() {
        // Not `||`: every child must be visited, so short-circuiting would leave
        // later subtrees unmarked.
        dynamic |= mark_dynamic(child, out);
    }
    if dynamic {
        out.insert(node.id());
    }
    dynamic
}

#[doc = include_str!("okui_screen.doc/view_value.md")]
const fn view_value(value: &UiIrValue) -> ViewValue {
    match value {
        // Text is unreachable here — the caller matches it first (see the note
        // above) — and joins `Null` rather than duplicating its arm.
        UiIrValue::Null | UiIrValue::Text(_) => ViewValue::Null,
        UiIrValue::Bool(flag) => ViewValue::Bool(*flag),
        UiIrValue::Int(number) => ViewValue::Int(*number),
        UiIrValue::Float(number) => ViewValue::Float(*number),
    }
}

#[cfg(test)]
mod tests {
    use super::{OkuiScreen, ScreenData, ScreenError};
    use crate::view::view::{View, ViewNode};

    /// `node`'s children, or a panic naming what it was instead.
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

    /// The first text content anywhere under `node`, depth first.
    fn first_text(node: &ViewNode) -> Option<&str> {
        match node {
            ViewNode::Text { content, .. } => Some(content),
            ViewNode::Element { children, .. } => children.nodes().iter().find_map(first_text),
        }
    }

    #[test]
    fn a_bound_body_renders_the_value_and_tracks_a_change() {
        // The case that could not exist: two lowerings of ONE compiled screen,
        // showing two values.
        let screen = OkuiScreen::compile("<div><span>{count}</span></div>").expect("compiles");
        screen.data().set("count", 0_i64);
        assert_eq!(first_text(root(&screen.view())), Some("0"));
        screen.data().set("count", 41_i64);
        assert_eq!(first_text(root(&screen.view())), Some("41"));
    }

    #[test]
    fn an_unbound_body_keeps_its_path_visible() {
        // A blank span would hide the typo; the bridge's own convention is to keep
        // a binding's path visible, and that is what an unbound one falls back to.
        let screen = OkuiScreen::compile("<div>{cuont}</div>").expect("compiles");
        assert_eq!(first_text(root(&screen.view())), Some("cuont"));
    }

    #[test]
    fn a_bound_attribute_takes_the_value_verbatim() {
        use crate::view::view::{CLASS, ViewValue};

        // Declared by being bound before the compile — §2.13 makes an undeclared name
        // a compile error, which the case below checks.
        let data = ScreenData::new();
        data.set("theme", "dark");
        let screen = OkuiScreen::compile_with(r"<div class={theme}/>", data).expect("compiles");
        let view = screen.view();
        let ViewNode::Element { props, .. } = root(&view) else {
            panic!("an element");
        };
        let class = props
            .iter()
            .find(|prop| prop.name() == CLASS)
            .expect("the class property survives substitution");
        assert_eq!(class.value(), &ViewValue::Text("dark".into()));
    }

    #[test]
    fn a_static_screen_is_marked_entirely_static() {
        // The cost claim: with nothing bound anywhere, every node takes the
        // bridge's own lowering and none is walked twice.
        let screen = OkuiScreen::compile("<div><span>hello</span></div>").expect("compiles");
        assert_eq!(screen.dynamic_node_count(), 0);
    }

    #[test]
    fn only_the_path_to_a_binding_is_dynamic() {
        // `div > span > {count}`: three nodes on the path, and the static sibling
        // is not one of them.
        let screen =
            OkuiScreen::compile("<div><span>{count}</span><b>x</b></div>").expect("compiles");
        assert_eq!(screen.dynamic_node_count(), 3);
    }

    #[test]
    fn a_literal_body_renders_too() {
        // Independent of whether anything else is bound: `{7}` is a value, and the
        // bridge alone would have made it an empty box.
        let screen = OkuiScreen::compile("<div>{7}</div>").expect("compiles");
        assert_eq!(first_text(root(&screen.view())), Some("7"));
    }

    #[test]
    fn a_static_sibling_of_a_binding_keeps_its_own_content() {
        let screen =
            OkuiScreen::compile("<div><span>{count}</span><b>literal</b></div>").expect("compiles");
        screen.data().set("count", 1_i64);
        let view = screen.view();
        let kids = children(root(&view));
        assert_eq!(first_text(&kids[0]), Some("1"));
        assert_eq!(first_text(&kids[1]), Some("literal"));
    }

    #[test]
    fn an_attribute_bound_to_an_undeclared_name_is_a_compile_error() {
        // §2.13: "unknown attributes, events and string method names are compile
        // errors". The name is declared by being bound, so a typo here is caught at
        // startup rather than rendering an attribute nobody set.
        let error =
            OkuiScreen::compile(r"<div class={theme}/>").expect_err("nothing declared `theme`");
        assert!(matches!(error, ScreenError::Compile(_)));
    }

    #[test]
    fn an_invalid_screen_reports_its_diagnostics() {
        // `<slot/>` without a `name` parses and then fails §2.12 validation.
        let error = OkuiScreen::compile("<div><slot/></div>")
            .expect_err("a malformed screen is an error, not an empty view");
        assert!(matches!(error, ScreenError::Compile(_)));
        assert!(!error.messages().is_empty(), "the diagnostics are carried");
    }
}
