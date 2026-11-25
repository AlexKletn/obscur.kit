//! `Analyzer`: the language-server core that turns open documents into LSP
//! diagnostics, document symbols and hover results.

use obscur_okui_compiler::compile::{CompileDiagnostic, Compiler, ContractRegistry};
use obscur_okui_compiler::lowering::LoweringError;
use obscur_okui_syntax::ast::{Document, Element};
use obscur_okui_syntax::parser::Parser;
use obscur_okui_syntax::syntax::SyntaxNode;

use crate::analysis::{DocumentSymbol, Hover, SymbolKind};
use crate::diagnostics::{LspDiagnostic, LspSeverity, Position, Range};
use crate::document::{DocumentStore, LineIndex};

/// Converts a `usize` length to `u32`, saturating at [`u32::MAX`].
fn to_u32(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

#[doc = include_str!("../../docs/api/analysis/analyzer.doc/Analyzer.md")]
#[derive(Debug, Clone)]
pub struct Analyzer {
    store: DocumentStore,
    registry: ContractRegistry,
}

impl Default for Analyzer {
    fn default() -> Self {
        Self {
            store: DocumentStore::default(),
            registry: ContractRegistry::builtins(),
        }
    }
}

impl Analyzer {
    /// Creates an analyzer with an empty document store and the built-in
    /// primitive/widget contracts registered — the same set `Compiler::new`
    /// seeds, so the editor's verdict matches the build's.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[doc = include_str!("../../docs/api/analysis/analyzer.doc/with_registry.md")]
    #[must_use]
    pub fn with_registry(mut self, registry: ContractRegistry) -> Self {
        self.registry = registry;
        self
    }

    /// Returns a shared reference to the document store.
    #[must_use]
    pub const fn store(&self) -> &DocumentStore {
        &self.store
    }

    /// Returns a mutable reference to the document store, for driving the
    /// `didOpen`/`didChange`/`didClose` lifecycle.
    pub const fn store_mut(&mut self) -> &mut DocumentStore {
        &mut self.store
    }

    #[doc = include_str!("../../docs/api/analysis/analyzer.doc/analyze.md")]
    #[must_use]
    pub fn analyze(&self, uri: &str) -> Vec<LspDiagnostic> {
        let Some(document) = self.store.get(uri) else {
            return Vec::new();
        };
        let index = LineIndex::new(&document.text);
        let mut diagnostics = Vec::new();

        let parsed = Parser::new().parse(&document.text);
        for error in parsed.errors() {
            diagnostics.push(LspDiagnostic::new(
                point_range(&index, error.offset()),
                LspSeverity::Error,
                error.message(),
                "okui-parse",
            ));
        }

        if parsed.is_ok()
            && let Some(ast) = Document::cast(parsed.tree().root().clone())
        {
            let result = Compiler::new(uri.to_owned())
                .with_registry(self.registry.clone())
                .compile(&ast);
            for diagnostic in result.diagnostics() {
                let (offset, message, severity) = map_compile_diagnostic(diagnostic);
                diagnostics.push(LspDiagnostic::new(
                    point_range(&index, offset),
                    severity,
                    message,
                    "okui-compile",
                ));
            }
        }

        diagnostics
    }

    /// Returns the hierarchical outline of elements/components in the document at
    /// `uri` (empty if no document is open).
    #[must_use]
    pub fn document_symbols(&self, uri: &str) -> Vec<DocumentSymbol> {
        let Some(document) = self.store.get(uri) else {
            return Vec::new();
        };
        let index = LineIndex::new(&document.text);
        let parsed = Parser::new().parse(&document.text);
        let Some(ast) = Document::cast(parsed.tree().root().clone()) else {
            return Vec::new();
        };
        ast.elements()
            .filter_map(|element| symbol_of(&index, &element))
            .collect()
    }

    /// Returns hover information for the innermost element/component containing
    /// `position`, if any.
    #[must_use]
    pub fn hover(&self, uri: &str, position: Position) -> Option<Hover> {
        let symbols = self.document_symbols(uri);
        let symbol = deepest_containing(&symbols, position)?;
        Some(Hover::new(
            format!("{} {}", symbol.kind.label(), symbol.name),
            symbol.range,
        ))
    }
}

#[doc = include_str!("../../docs/api/analysis/analyzer.doc/map_compile_diagnostic.md")]
fn map_compile_diagnostic(diagnostic: &CompileDiagnostic) -> (u32, String, LspSeverity) {
    match diagnostic {
        CompileDiagnostic::Resolution(error) => {
            (error.offset(), error.to_string(), LspSeverity::Error)
        }
        CompileDiagnostic::Type(error) => (error.offset(), error.to_string(), LspSeverity::Error),
        CompileDiagnostic::Validation(validation) => (
            validation.offset(),
            validation.message().to_owned(),
            LspSeverity::Error,
        ),
        CompileDiagnostic::Lowering(error) => {
            // A missing `<For>` key is non-fatal (spec §2.11) — report it as a
            // warning; structural lowering errors stay errors.
            let (offset, severity) = match error {
                LoweringError::MissingTagName(offset) => (*offset, LspSeverity::Error),
                LoweringError::EmptyDocument => (0, LspSeverity::Error),
                LoweringError::MissingLoopKey(offset) => (*offset, LspSeverity::Warning),
            };
            (offset, error.to_string(), severity)
        }
    }
}

/// Builds a one-character [`Range`] anchored at `offset` (an empty range at the
/// end of the document), mapping byte offsets through `index`.
fn point_range(index: &LineIndex, offset: u32) -> Range {
    let start = index.offset_to_position(offset);
    let end = index.offset_to_position(offset.saturating_add(1));
    Range::new(start, end)
}

/// Builds a [`Range`] covering the full source span of `node`.
fn node_range(index: &LineIndex, node: &SyntaxNode) -> Range {
    let start_offset = node.offset().unwrap_or(0);
    let end_offset = start_offset.saturating_add(to_u32(node.text().len()));
    Range::new(
        index.offset_to_position(start_offset),
        index.offset_to_position(end_offset),
    )
}

/// Builds a [`DocumentSymbol`] for `element` and its element-shaped children,
/// or `None` for a nameless element.
fn symbol_of(index: &LineIndex, element: &Element) -> Option<DocumentSymbol> {
    let name = element.tag_name()?;
    let kind = SymbolKind::of_tag(&name);
    let range = node_range(index, element.syntax());
    let children = element
        .children()
        .filter_map(Element::cast)
        .filter_map(|child| symbol_of(index, &child))
        .collect();
    Some(DocumentSymbol::new(name, kind, range).with_children(children))
}

/// Finds the deepest symbol whose range contains `position`.
fn deepest_containing(symbols: &[DocumentSymbol], position: Position) -> Option<&DocumentSymbol> {
    for symbol in symbols {
        if symbol.range.contains(position) {
            return Some(deepest_containing(&symbol.children, position).unwrap_or(symbol));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_okui_compiler::validation::ComponentContract;
    use obscur_view::contract::ComponentSurface;

    const URI: &str = "file:///app.okui";

    fn button_registry() -> ContractRegistry {
        ContractRegistry::new().with(
            ComponentContract::new("Button")
                .requiring_prop("label")
                .allowing_event("click"),
        )
    }

    fn analyzer_with(text: &str) -> Analyzer {
        let mut analyzer = Analyzer::new().with_registry(button_registry());
        analyzer.store_mut().open(URI, 1, text);
        analyzer
    }

    #[test]
    fn well_formed_document_has_no_diagnostics() {
        let analyzer = analyzer_with("<Button label=\"Save\" on:click=submit/>");
        assert!(
            analyzer.analyze(URI).is_empty(),
            "diagnostics: {:?}",
            analyzer.analyze(URI)
        );
    }

    /// Parses `source` into the AST the compiler takes.
    fn ast(source: &str) -> Document {
        Document::cast(Parser::new().parse(source).tree().root().clone()).expect("a document")
    }

    #[test]
    fn a_default_analyzer_enforces_the_builtin_contracts() {
        // The registry was `ContractRegistry::default()` — EMPTY — so this
        // server, and its `--check` CI mode, enforced §2.13 for NOTHING: not one
        // user component, and not one of the built-in tags either.
        // `echo '<Button laebl="Save"/>' | obscur-okui-lsp --check` printed
        // `diagnostics: none` and exited zero on a document `Compiler::new()`
        // rejects and the `okui!` macro refuses at the author's own token.
        for (source, expected) in [
            (
                r#"<Button label="Save" laebl="Save"/>"#,
                "unknown attribute 'laebl' on component 'Button'",
            ),
            (
                r#"<Image src="logo.png" widht="10"/>"#,
                "unknown attribute 'widht' on component 'Image'",
            ),
            (
                r#"<Image alt="logo"/>"#,
                "component 'Image' requires prop 'src'",
            ),
        ] {
            // The premise, asserted rather than assumed: the build path really
            // does reject this document. If `Compiler::new` ever stops seeding
            // the built-ins, the agreement below would hold vacuously — both
            // sides silent — and this test would pass while enforcing nothing.
            assert!(
                Compiler::new(URI.to_owned())
                    .compile(&ast(source))
                    .has_errors(),
                "`{source}`: a default compile must reject this, or this test is vacuous"
            );

            let mut analyzer = Analyzer::new();
            analyzer.store_mut().open(URI, 1, source);
            let diagnostics = analyzer.analyze(URI);
            assert!(
                diagnostics
                    .iter()
                    .any(|d| d.source == "okui-compile" && &*d.message == expected),
                "`{source}`: the editor must report what the build rejects; got {diagnostics:?}"
            );
        }
    }

    #[test]
    fn a_default_analyzer_leaves_correct_documents_clean() {
        // The disjoint half. Seeding the built-ins must make the WRONG document
        // loud and change nothing else — a correct `<Button>`/`<Image>` stays
        // silent, and an unregistered tag stays permissive, which is the
        // deliberate §2.13 policy (`Compiler` does not refuse a tag whose
        // contract it cannot see; see `builtin_contracts`). If this file had
        // instead made unknown tags an error, the test above would still pass
        // and this one would fail.
        for source in [
            r#"<Button label="Save" disabled="true"/>"#,
            r#"<Image src="logo.png" alt="logo"/>"#,
            r#"<CustomThing whatever="1"/>"#,
        ] {
            let mut analyzer = Analyzer::new();
            analyzer.store_mut().open(URI, 1, source);
            let diagnostics = analyzer.analyze(URI);
            assert!(
                diagnostics.is_empty(),
                "`{source}` must analyze clean; got {diagnostics:?}"
            );
        }
    }

    #[test]
    fn a_user_components_registered_surface_is_enforced_by_the_editor() {
        // §2.14's "static registration is the default", on the only front end
        // that can honour it for a USER component. `CardProps::contract_surface()`
        // is a function that runs when the built crate runs — a proc-macro
        // expanding inside that crate cannot call it, and neither can that
        // crate's own `build.rs` (cargo rejects a self build-dependency as a
        // cyclic package dependency). A language server is an ordinary binary
        // that links the component crate and runs after it, so it is the one
        // place `register_surface` is reachable. This is that wire, and the
        // built-ins survive it.
        let surface = ComponentSurface::new("Card")
            .requiring_prop("title")
            .allowing_event("open");
        let mut analyzer =
            Analyzer::new().with_registry(ContractRegistry::builtins().with_surface(&surface));
        analyzer
            .store_mut()
            .open(URI, 1, r#"<div><Card titel="Hi"/><Image alt="x"/></div>"#);
        let messages: Vec<_> = analyzer
            .analyze(URI)
            .into_iter()
            .map(|d| d.message)
            .collect();

        assert!(
            messages
                .iter()
                .any(|m| &**m == "unknown attribute 'titel' on component 'Card'"),
            "the user component's registered surface must bite: {messages:?}"
        );
        assert!(
            messages
                .iter()
                .any(|m| &**m == "component 'Card' requires prop 'title'"),
            "and its required prop: {messages:?}"
        );
        // `with_surface` on top of `builtins()` must ADD, not replace: the
        // built-in `Image` contract still bites in the same document. Registering
        // a user component by handing over `ContractRegistry::new()` is what
        // silently drops these.
        assert!(
            messages
                .iter()
                .any(|m| &**m == "component 'Image' requires prop 'src'"),
            "the built-ins must survive a user registration: {messages:?}"
        );
    }

    #[test]
    fn missing_document_analyzes_to_nothing() {
        let analyzer = Analyzer::new();
        assert!(analyzer.analyze("file:///nope.okui").is_empty());
        assert!(analyzer.document_symbols("file:///nope.okui").is_empty());
    }

    #[test]
    fn parse_error_yields_an_error_diagnostic_with_a_range() {
        // A missing close tag is a recoverable parse error.
        let analyzer = analyzer_with("<Button>oops");
        let diagnostics = analyzer.analyze(URI);
        assert!(!diagnostics.is_empty());
        let parse = diagnostics
            .iter()
            .find(|d| d.source == "okui-parse")
            .expect("a parse diagnostic");
        assert_eq!(parse.severity, LspSeverity::Error);
        // The error is anchored on the first line at/after the tag.
        assert_eq!(parse.range.start.line, 0);
        assert!(parse.range.end >= parse.range.start);
    }

    #[test]
    fn semantic_error_yields_an_error_at_the_attribute() {
        // Parses cleanly but violates the Button contract (unknown attribute).
        //
        // This test asserted `LspSeverity::Warning`, and it is inverted here
        // rather than deleted. That assertion was the bug in a stable form:
        // `--check`'s gate reads error severity only, so a §2.13 violation
        // printed and then exited zero, while the same document is a hard
        // `syn::Error` in `okui!` and a `build.rs` panic on the file path.
        let analyzer = analyzer_with("<Button label=\"Save\" colr=\"red\"/>");
        let diagnostics = analyzer.analyze(URI);
        let semantic = diagnostics
            .iter()
            .find(|d| d.source == "okui-compile")
            .expect("a compile diagnostic");
        assert_eq!(semantic.severity, LspSeverity::Error);
        assert!(
            semantic.message.contains("unknown attribute 'colr'"),
            "message: {}",
            semantic.message
        );
        // `colr` starts at byte 21 on line 0.
        assert_eq!(semantic.range.start, Position::new(0, 21));
    }

    #[test]
    fn the_check_gate_fails_on_a_contract_violation_alone() {
        // Rule: a check whose verdict nothing reads is not a check. `--check`
        // exits non-zero only on an ERROR-severity diagnostic, so this is the
        // property that makes the §2.13 enforcement above worth anything in CI.
        //
        // The source is chosen to isolate it: `<Button laebl="Save"/>` raises
        // contract diagnostics and NOTHING else — no parse error, no unresolved
        // name. Probing with `<VirtualFor each={items}>` instead would have been
        // useless, because its undeclared `{items}` raises a resolution error
        // that has always been fatal, and the test would pass with this whole
        // mapping reverted.
        let mut analyzer = Analyzer::new();
        analyzer
            .store_mut()
            .open(URI, 1, r#"<Button laebl="Save"/>"#);
        let diagnostics = analyzer.analyze(URI);

        assert!(
            diagnostics
                .iter()
                .all(|d| d.source == "okui-compile" && !d.message.contains("unknown name")),
            "the probe must isolate the contract rule; got {diagnostics:?}"
        );
        assert!(
            diagnostics.iter().any(|d| d.severity == LspSeverity::Error),
            "a §2.13 violation must reach the `--check` gate; got {diagnostics:?}"
        );
    }

    #[test]
    fn resolution_error_is_reported_as_error() {
        // `{missing}` is an unknown reference — a resolution error.
        let mut analyzer = Analyzer::new();
        analyzer.store_mut().open(URI, 1, "<Text value={missing}/>");
        let diagnostics = analyzer.analyze(URI);
        assert!(
            diagnostics
                .iter()
                .any(|d| d.source == "okui-compile" && d.severity == LspSeverity::Error),
            "diagnostics: {diagnostics:?}"
        );
    }

    #[test]
    fn document_symbols_list_top_level_and_nested_elements() {
        let mut analyzer = Analyzer::new();
        analyzer
            .store_mut()
            .open(URI, 1, "<Col>\n  <Row/>\n  <Icon/>\n</Col>");
        let symbols = analyzer.document_symbols(URI);
        assert_eq!(symbols.len(), 1);
        assert_eq!(&*symbols[0].name, "Col");
        assert_eq!(symbols[0].kind, SymbolKind::Component);
        assert_eq!(symbols[0].children.len(), 2);
        let names: Vec<_> = symbols[0]
            .children
            .iter()
            .map(|s| s.name.to_string())
            .collect();
        assert_eq!(names, ["Row", "Icon"]);
    }

    #[test]
    fn document_symbol_range_covers_the_element() {
        let mut analyzer = Analyzer::new();
        analyzer.store_mut().open(URI, 1, "<App/>");
        let symbols = analyzer.document_symbols(URI);
        assert_eq!(symbols[0].range.start, Position::new(0, 0));
        assert_eq!(symbols[0].range.end, Position::new(0, 6));
    }

    #[test]
    fn hover_returns_the_innermost_element_name() {
        let mut analyzer = Analyzer::new();
        analyzer.store_mut().open(URI, 1, "<Col><Row/></Col>");
        // Byte 6 is inside `<Row/>` (which spans bytes 5..11).
        let hover = analyzer
            .hover(URI, Position::new(0, 6))
            .expect("a hover result");
        assert!(
            hover.contents.contains("Row"),
            "contents: {}",
            hover.contents
        );
    }

    #[test]
    fn hover_outside_any_element_is_none() {
        let mut analyzer = Analyzer::new();
        analyzer.store_mut().open(URI, 1, "  <A/>");
        assert!(analyzer.hover(URI, Position::new(0, 0)).is_none());
    }
}
