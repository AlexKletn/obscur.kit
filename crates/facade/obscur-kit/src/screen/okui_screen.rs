//! [`OkuiScreen`] — a compiled `.okui` document, re-lowered to a [`View`] per frame.

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

/// The logical name compile diagnostics are reported against. A screen handed over
/// as a string has no path, and inventing one would put a file that does not exist
/// in an error message.
const SCREEN_NAME: &str = "screen.okui";

/// The control tag whose body is a **row template**, not content: a
/// `<VirtualFor>`. See [`OkuiScreen::view`] for why a binding inside one is left
/// alone.
const VIRTUAL_FOR: &str = "VirtualFor";

/// A `.okui` source compiled **once** to UI IR and lowered to a [`View`] on every
/// frame, resolving its `{bindings}` against a [`ScreenData`] table (spec §2.9).
///
/// # What this replaces
///
/// `App::screen` used to compile the source in `run` and
/// hand the runtime a builder that returned that one pre-built `View` for the
/// application's whole life:
///
/// ```ignore
/// let view = compile_screen(&src)?;
/// Box::new(move || view.clone())      // "static v1 materialization"
/// ```
///
/// so no state change could reach the screen — §2.9 defines `.okui` as a front end
/// "compiled into the same UI IR as the Rust DSL", a *peer* of the Rust DSL rather
/// than a static subset of it. Holding the IR instead of the `View` costs one
/// lowering per frame and no parsing at all, and gives the lowering somewhere to
/// read state from.
///
/// # What the two `.okui` routes now do
///
/// - **Build time** (`build.rs` +
///   [`ViewCodegen`](crate::okui_compiler::codegen::ViewCodegen) with
///   [`CodegenRoot::facade`](crate::okui_compiler::codegen::CodegenRoot::facade)):
///   the document becomes ordinary Rust, `{count}` becomes a read of the
///   application's own `count`, and §2.13's contract validation happens while the
///   crate compiles. This is the route §2.9's "compile-time contract validation"
///   describes, and it is the one to use for anything with real data behind it.
/// - **Run time** (this type): the source is text until `run`, so its contract
///   errors are a [`ScreenError`] at startup rather than a compile error, and its
///   bindings resolve against [`ScreenData`] instead of Rust locals. Structure is
///   fixed at compile time — a `<If>`/`<For>` is statically materialised by the
///   bridge — so this route reaches *values*, not shape.
///
/// # Cost
///
/// Lowering is O(nodes) and allocates the frame's `View`; nothing re-parses and
/// nothing re-compiles. Subtrees that contain neither a bound property nor a body
/// `{…}` are handed to [`UiIrViewBridge`] whole and never walked twice, so a mostly
/// static screen costs exactly one bridge lowering — see
/// [`dynamic_node_count`](Self::dynamic_node_count).
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
    dynamic: HashSet<u64>,
}

impl OkuiScreen {
    /// Parses and compiles `source` with no state declared — for a screen whose only
    /// bindings are in element bodies.
    ///
    /// An **attribute** binding (`class={theme}`) references a name, and §2.13 makes
    /// an unknown name a compile error, so a screen that binds an attribute must
    /// declare it: use [`compile_with`](Self::compile_with) and bind the name in the
    /// table first.
    ///
    /// # Errors
    ///
    /// See [`compile_with`](Self::compile_with).
    pub fn compile(source: &str) -> Result<Self, ScreenError> {
        Self::compile_with(source, ScreenData::new())
    }

    /// Parses and compiles `source` against `data`, keeping the IR and the table.
    ///
    /// Every name **currently bound** in `data` is declared to the compiler as the
    /// screen's state, which is what lets an attribute binding reference it — and what
    /// makes a reference to anything else a §2.13 compile error ("unknown attributes,
    /// events and string method names are compile errors") instead of a value that
    /// silently never arrives. So bind a name to its initial value *before* compiling;
    /// for `App` that means writing
    /// `screen_data` before `run`.
    ///
    /// Body interpolations are not name-checked on this route — an unbound `{count}`
    /// renders its own path, visibly — which is a gap in the front end's resolution
    /// pass rather than a decision made here.
    ///
    /// # Errors
    ///
    /// [`ScreenError::Parse`] when the source does not parse, and
    /// [`ScreenError::Compile`] when it parses but fails §2.13/§2.12 validation —
    /// each carrying the diagnostics, because a screen that failed to compile must
    /// not render as a silently empty document.
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

    /// Lowers the compiled document to this frame's [`View`].
    ///
    /// Call once per frame — this is the body of the builder
    /// `App::screen` hands
    /// [`UiRuntime::set_root_view`](crate::runtime::UiRuntime::set_root_view).
    ///
    /// # What the lowering adds to the bridge
    ///
    /// [`UiIrViewBridge`] owns every naming decision (tags → `ComponentId`s,
    /// reserved attributes, events, keys, `<VirtualFor>` markers) and this does not
    /// re-implement any of it: each node's lowering *is* the bridge's. Two things
    /// the bridge cannot know are repaired afterwards, because it has no values to
    /// resolve a binding with:
    ///
    /// - a body `{expr}` — which the bridge lowers to an anonymous box that draws
    ///   nothing — becomes a text node carrying the bound value;
    /// - a bound property (`class={theme}`), which the bridge lowers to its *path*
    ///   as placeholder text, takes the bound value verbatim. Verbatim means a
    ///   `class` wants a [`Text`](ViewValue::Text): the value is not stringified,
    ///   because a property is typed data and an `Int` bound to a numeric attribute
    ///   should stay an `Int`.
    ///
    /// An unbound path keeps the bridge's placeholder — the path itself — so a
    /// misspelled binding is visible on screen instead of blanking silently.
    ///
    /// # `<VirtualFor>` is deliberately left alone
    ///
    /// A `<VirtualFor>` lowers to one row *template* that the runtime clones per
    /// realized row, so a value substituted into it would render identically in
    /// every row — the one thing a list must not do. Per-row data comes from
    /// `App::virtual_list`'s Rust row builder,
    /// which is called with the row's model index; a `{binding}` inside a
    /// `<VirtualFor>` therefore keeps its placeholder.
    #[must_use]
    pub fn view(&self) -> View {
        View::from_fragment(ViewFragment::from_nodes(self.lower(self.document.root())))
    }

    /// Lowers one IR node, substituting what the data supplies. Returns a `Vec`
    /// for the same reason the bridge does: a transparent node splices.
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

    /// Every child of `node`, lowered through [`lower`](Self::lower) and spliced.
    fn lower_children(&self, node: &UiIrNode) -> Vec<ViewNode> {
        node.children()
            .iter()
            .flat_map(|child| self.lower(child))
            .collect()
    }

    /// Replaces `view`'s bound property values and its children with the data-aware
    /// versions. A no-op for a text node, which has neither.
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

    /// What a body `{expr}` renders this frame.
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

/// Records every IR node whose lowering the data can change — one with a bound
/// property or a body interpolation, and every ancestor of one — returning whether
/// `node`'s own subtree is such a node.
///
/// An interpolation counts whatever its expression is (a literal `{7}` included),
/// so a body `{…}` renders its value consistently rather than depending on whether
/// a sibling happened to be bound.
///
/// A bound `key={…}` deliberately does **not** count: a key is reconciliation
/// identity, and this route materialises a loop statically, so there is nothing
/// per-item to key and substituting one would change node identity between frames
/// for no gain.
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

/// The [`ViewValue`] a **non-text** IR literal is worth — the same mapping the
/// bridge makes for a property, needed here because rendering goes through
/// [`ScreenData::render`].
///
/// Text is handled by the caller: cloning a `Box<str>` is not possible in a `const
/// fn`, and a literal string needs no mapping to read as itself.
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
