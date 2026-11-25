//! [`OkuiScreen`] — a compiled `.okui` document, re-lowered to a [`View`] per frame.

use std::{collections::HashSet, sync::Arc};

use crate::okui_compiler::compile::{Compiler, ContractRegistry};
use crate::okui_compiler::symbol::Symbol;
use crate::okui_syntax::ast::Document;
use crate::okui_syntax::parser::Parser;
use crate::ui_ir::document::UiIrDocument;
use crate::view::view::{View, ViewFragment};

use super::{
    ScreenData, ScreenError,
    screen_lowering::{ScreenLowering, mark_dynamic},
};

/// The logical name compile diagnostics are reported against. A screen handed over
/// as a string has no path, and inventing one would put a file that does not exist
/// in an error message.
const SCREEN_NAME: &str = "screen.okui";

#[doc = include_str!("../../docs/api/screen/okui_screen.doc/OkuiScreen.md")]
pub struct OkuiScreen {
    /// The compiled IR — the artefact §2.9 calls the shared representation. Kept
    /// instead of the lowered `View` so a frame can be rebuilt from it.
    document: UiIrDocument,
    /// The values this screen's bindings read.
    data: ScreenData,
    /// The ids of the IR nodes whose lowering the data can change: a node with a
    /// bound property or a body interpolation, and every ancestor of one. A node
    /// outside this set lowers identically on every frame, so the bridge's own
    /// result for it is reused verbatim.
    dynamic: Arc<HashSet<u64>>,
    list_scope: u32,
}

impl OkuiScreen {
    #[doc = include_str!("../../docs/api/screen/okui_screen.doc/compile.md")]
    pub fn compile(source: &str) -> Result<Self, ScreenError> {
        Self::compile_with(source, ScreenData::new())
    }

    #[doc = include_str!("../../docs/api/screen/okui_screen.doc/compile_with.md")]
    pub fn compile_with(source: &str, data: ScreenData) -> Result<Self, ScreenError> {
        Self::compile_with_registry(source, data, ContractRegistry::builtins())
    }

    /// Compiles a screen against the supplied component contract registry and application data bindings.
    pub fn compile_with_registry(
        source: &str,
        data: ScreenData,
        registry: ContractRegistry,
    ) -> Result<Self, ScreenError> {
        let parsed = Parser::new().parse(source);
        if !parsed.errors().is_empty() {
            return Err(ScreenError::Parse(
                parsed.errors().iter().map(ToString::to_string).collect(),
            ));
        }
        let document = Document::cast(parsed.tree().root().clone()).ok_or_else(|| {
            ScreenError::Parse(vec!["the `.okui` root is not a document".to_owned()])
        })?;
        let mut compiler = Compiler::new(SCREEN_NAME).with_registry(registry);
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
        static NEXT_SCOPE: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
        let list_scope = NEXT_SCOPE.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Ok(Self {
            document,
            data,
            dynamic: Arc::new(dynamic),
            list_scope,
        })
    }

    /// The table this screen's bindings resolve against.
    #[must_use]
    pub const fn data(&self) -> &ScreenData {
        &self.data
    }

    /// The compiled IR, for a host that wants to inspect or re-lower it itself.
    #[must_use]
    pub const fn document(&self) -> &UiIrDocument {
        &self.document
    }

    /// How many IR nodes the data can change — the size of the per-frame walk that
    /// is *not* a plain [`UiIrViewBridge`] call. `0` means this screen is entirely
    /// static and its lowering is the bridge's, node for node.
    #[must_use]
    pub fn dynamic_node_count(&self) -> usize {
        self.dynamic.len()
    }

    #[doc = include_str!("../../docs/api/screen/okui_screen.doc/view.md")]
    #[must_use]
    pub fn view(&self) -> View {
        let lowering = ScreenLowering::new(
            self.data.snapshot(),
            Arc::clone(&self.dynamic),
            self.list_scope,
        );
        View::from_fragment(ViewFragment::from_nodes(
            lowering.lower_nodes(std::iter::once(self.document.root())),
        ))
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
    fn a_custom_screen_contract_validates_its_required_props() {
        use crate::okui_compiler::compile::ContractRegistry;
        use crate::view::contract::ComponentSurface;

        let surface = ComponentSurface::new("CustomBadge").requiring_prop("label");
        let registry = ContractRegistry::builtins().with_surface(&surface);
        let invalid = OkuiScreen::compile_with_registry(
            "<CustomBadge/>",
            ScreenData::new(),
            registry.clone(),
        );
        assert!(matches!(invalid, Err(super::ScreenError::Compile(messages))
            if messages.iter().any(|message| message.contains("label"))));
        let valid = OkuiScreen::compile_with_registry(
            r#"<CustomBadge label="ok"/>"#,
            ScreenData::new(),
            registry,
        );
        assert!(valid.is_ok());
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
